use std::{collections::HashMap, io, str::FromStr, sync::{Arc, Mutex, mpsc::{self, Sender}}};

use cpal::{Stream, StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ratatui::{DefaultTerminal, Frame, crossterm::event::{self, Event, KeyEvent, KeyEventKind}, layout::Rect, widgets::Widget};

use crate::{Note, Phrase, PhraseEditor, PhraseEditorCommand, Synthesizer, arrangement::{Arrangement, InstrumentId, PhraseId}, osc_synths::PolyphonicOscSynth, playback::{PlaybackCommand, PlaybackKind, PlaybackState}, utils::PageCommand};

/// the page this app is currently on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Phrase
}

pub struct TuiTrackerApp {
    /// the arrangement data
    arrangement: Arc<Mutex<Arrangement>>,

    /// the instruments
    instruments: Arc<Mutex<HashMap<InstrumentId, Box<dyn Synthesizer>>>>,

    /// checks if playback is occuring
    playback_kind: PlaybackKind,

    /// the current page we are on
    page: AppPage,

    /// the editor for the current phrase
    phrase_editor: PhraseEditor,

    /// the current phrase
    current_phrase: PhraseId,

    /// the index of the current instrument
    current_instrument: InstrumentId,

    /// a sender of commands to the playback thread
    playback_sender: Sender<PlaybackCommand>,

    /// the stream
    stream_opt: Option<Stream>,

    /// whether the app should exit
    exit: bool,
}

impl TuiTrackerApp {
    const BASE_WHOLES_PER_SECOND: f64 = 24.0 / 60.0;

    /// create a new instance of the app
    /// note: this is currently a test function
    pub fn new() -> Self {
        let mut phrase = Box::new(Phrase::new(16, 16));
        phrase.set_note(0, 0, Some(Note::from_str("C4").unwrap()));
        phrase.set_note(4, 0, Some(Note::from_str("D4").unwrap()));
        phrase.set_note(8, 0, Some(Note::from_str("E4").unwrap()));
        phrase.set_note(12, 0, Some(Note::from_str("F4").unwrap()));

        phrase.set_note(0, 1, Some(Note::from_str("G4").unwrap()));
        phrase.set_note(4, 1, Some(Note::from_str("A4").unwrap()));
        phrase.set_note(8, 1, Some(Note::from_str("B4").unwrap()));
        phrase.set_note(12, 1, Some(Note::from_str("C5").unwrap()));

        let instruments = HashMap::from([
            (
                InstrumentId(0),
                (PolyphonicOscSynth::sine_specification().generate_synth)()
            )
        ]);
        let instruments = Arc::new(Mutex::new(instruments));

        let mut arrangement = Arrangement::default();
        arrangement.set_phrase(PhraseId(0), phrase);
        let arrangement = Arc::new(Mutex::new(arrangement));

        let (playback_sender, stream) = Self::init_playback(arrangement.clone(), instruments.clone());

        Self {
            stream_opt: Some(stream),
            playback_sender,
            instruments,
            arrangement,
            playback_kind: PlaybackKind::Off,
            page: AppPage::Phrase,
            phrase_editor: PhraseEditor::new(),
            current_phrase: PhraseId(0),
            current_instrument: InstrumentId(0),
            exit: false,
        }
    }

    /// constructs the playback state and thread, returning the sender to the new thread
    fn init_playback(
        arrangent: Arc<Mutex<Arrangement>>,
        instruments: Arc<Mutex<HashMap<InstrumentId, Box<dyn Synthesizer>>>>,
    ) -> (Sender<PlaybackCommand>, Stream) {
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        let mut supported_configs_range = device.supported_output_configs()
            .expect("error while querying configs");

        let sample_rate = 48000;
        let supported_config = supported_configs_range
            .find(|config| config.try_with_sample_rate(sample_rate).is_some())
            .expect("no supported config?!")
            .with_sample_rate(sample_rate);
        let mut config: StreamConfig = supported_config.into();
        config.buffer_size = cpal::BufferSize::Fixed(sample_rate * 1 / 1000);

        let (sender, receiver) = mpsc::channel();

        let mut playback_state = PlaybackState::new(
            arrangent,
            instruments,
            receiver,
            sample_rate
        );

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                playback_state.play(data);
            },
            move |_err| {
                println!("ERROR BRUH")
            },
            None // None=blocking, Some(Duration)=timeout
        ).expect("Could not build stream.");

        let _ = stream.play().expect("stream could not play");

        (sender, stream)
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn phrase_handle_key_event(&mut self, key_event: KeyEvent) {
        let mut arrangement_lock = self.arrangement.lock()
            .expect("cannot handle poisoned lock");

        let phrase = arrangement_lock.get_phrase_mut(self.current_phrase)
            .expect("ensure initialize for debug only, remove this later");

        let page_cmd = self.phrase_editor.handle_key_event(&phrase, key_event);

        type E = PhraseEditorCommand;
        type P = PageCommand<E>;

        match page_cmd {
            P::Quit => self.exit = true,
            P::Nop => (),

            P::Command(edit_cmd) => match edit_cmd {
                E::Edit(cmd) => {
                    cmd.execute(phrase);
                },

                E::TogglePhrase => {
                    if self.playback_kind == PlaybackKind::PhrasePlayback {
                        let _ = self.playback_sender.send(PlaybackCommand::StopPlayback);
                        self.playback_kind = PlaybackKind::Off
                    } else {
                        let playback_cmd = PlaybackCommand::LoopPhrase {
                            phrase_id: self.current_phrase,
                            instrument_id: self.current_instrument,
                            wholes_per_second: Self::BASE_WHOLES_PER_SECOND,
                        };
                        let _ = self.playback_sender.send(playback_cmd);
                        self.playback_kind = PlaybackKind::PhrasePlayback
                    }
                },
                E::PlayPhrase => {
                    if self.playback_kind != PlaybackKind::PhrasePlayback {
                        let playback_cmd = PlaybackCommand::LoopPhrase {
                            phrase_id: self.current_phrase,
                            instrument_id: self.current_instrument,
                            wholes_per_second: Self::BASE_WHOLES_PER_SECOND,
                        };
                        let _ = self.playback_sender.send(playback_cmd);
                        self.playback_kind = PlaybackKind::PhrasePlayback
                    }
                },
                E::StopPhrase => {
                    if self.playback_kind != PlaybackKind::Off {
                        let _ = self.playback_sender.send(PlaybackCommand::StopPlayback);
                        self.playback_kind = PlaybackKind::Off;
                    }
                },
            },
        }

        std::mem::drop(arrangement_lock);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            match self.page {
                AppPage::Phrase => self.phrase_handle_key_event(key_event),
            }
        }
        _ => {}
        };

        Ok(())
    }

}

impl Widget for &mut TuiTrackerApp {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer
    ) where Self: Sized {
        if area.height < 3 {
            return;
        }

        let page_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height - 2
        };
        let cmd_line_area = Rect {
            x: area.x,
            y: area.height - 1,
            width: area.width,
            height: 1
        };
        let status_line_area = Rect {
            x: area.x,
            y: area.height - 2,
            width: area.width,
            height: 1
        };

        match self.page {
            AppPage::Phrase => {
                let arrangement_lock = self.arrangement.lock()
                    .expect("cannot handle poisoned lock");

                let phrase = arrangement_lock.get_phrase(self.current_phrase)
                    .expect("render should be called after phrase is valid");
                self.phrase_editor.render(phrase, true, page_area, buf);
            }
        }
    }

}
