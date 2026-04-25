use std::{io, str::FromStr, sync::OnceLock};

use cpal::{StreamConfig, traits::{DeviceTrait, HostTrait, StreamTrait}};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, layout::{Position, Rect}, style::Stylize, symbols::border, text::{Line, Text}, widgets::{Block, Paragraph, Widget}};
use tracker_tui::{osc_synths::PolyphonicOscSynth, playback::{GlobalPlaybackState, PhrasePlaybackState}, *};

static TEST_PHRASE: OnceLock<&'static Phrase> = OnceLock::new();

fn main() -> io::Result<()> {
    TEST_PHRASE.set({
        let mut phrase = Box::new(Phrase::new(Phrase::TICKS_PER_WHOLE_NOTE, 4));
        let composition: &[(&[&str], u32)] = &[
            (
                &["d4", "f4", "g4", "c5"],
                0,
            ),
            (
                &["d4", "f4", "g4", "b4"],
                1,
            ),
            (
                &["c4", "e4", "a4", "b4", "d5"],
                2,
            ),
        ];
        for (notes, time) in composition {
            for (column, note_str) in notes.iter().enumerate() {
                if let Ok(note) = Note::from_str(note_str) {
                    phrase.set_note(*time, column, Some(note));
                }           
            }
        }
        Box::leak(phrase)
    }).expect("first thing in main function");

    //ratatui::run(|terminal| TuiTrackerApp::new().run(terminal))
    play_phrase();

    Ok(())
}

fn play_phrase() {

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");

    let sample_rate = 48000;
    let supported_config = supported_configs_range
        .find(|config| config.try_with_sample_rate(sample_rate).is_some())
        .expect("no supported config?!")
        .with_sample_rate(sample_rate);
    let config: StreamConfig = supported_config.into();

    let &phrase_ref = TEST_PHRASE.get().unwrap();
    let synth_box = (PolyphonicOscSynth::saw_specification().generate_synth)();
    let mut synth_box = unsafe { std::mem::transmute::<_, Box<dyn Synthesizer + Send>>(synth_box) };

    let mut global_state = GlobalPlaybackState {
        wholes_per_second: 10.0/60.0,
        sample_rate: sample_rate,
        sample_index: 0,
    };

    let mut phrase_state: Option<PhrasePlaybackState> = Some(Default::default());

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            if let Some(index) = playback::play_phrase(
                phrase_ref,
                synth_box.as_mut(),
                &mut phrase_state,
                &mut global_state, data
            ) {
                for sample in data.iter_mut().skip(index) {
                    *sample = 0.0;
                }
            }
        },
        move |_err| {
            println!("ERROR BRUH")
        },
        None // None=blocking, Some(Duration)=timeout
    ).expect("Could not build stream.");

    let _ = stream.play().expect("stream could not play");

    loop {}
}

/*
#[derive(Debug)]
pub struct RsynthTuiApp {
    phrase: Phrase,
    phrase_editor_data: PhraseEditorData,
    exit: bool
}

impl RsynthTuiApp {
    pub fn new() -> Self {
        let mut phrase = Phrase::new(16, 16);
        phrase.set_note(0, 0, Some(Note::C4.add_semitones(1)));
        phrase.set_note(3, 0, Some(Note::A4));
        phrase.set_note(7, 0, Some(Note::C0));
        phrase.set_note(11, 0, Some(Note::B9));
        phrase.set_note(2, 1, Some(Note::B9));
        phrase.set_note(6, 1, Some(Note::B9));
        phrase.set_note(10, 1, Some(Note::B9));
        phrase.set_note(14, 1, Some(Note::B9));
        phrase.set_effect(0, 0, Some(PhraseEffect::Silence));
        phrase.set_effect(0, 1, Some(PhraseEffect::Silence));
        phrase.set_effect(0, 2, Some(PhraseEffect::SetTransition(PhraseTransitionMode::Release)));
        phrase.set_effect(3, 0, Some(PhraseEffect::SetTransition(PhraseTransitionMode::Lerp)));
        phrase.set_effect(3, 1, Some(PhraseEffect::SetTransition(PhraseTransitionMode::Release)));
        phrase.set_effect(12, 0, Some(PhraseEffect::Silence));
        Self {
            phrase,
            phrase_editor_data: PhraseEditorData {
                cell_pos: Position::MIN,
                cam_pos: Position::MIN,
                text: Some("helloww".to_string()),
                focused: true,
            },
            exit: false
        }
    }

    /// runs the application's main loop until the user quits
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

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit = true,

            KeyCode::Char('j') => self.phrase_editor_data.cell_pos.y = self.phrase_editor_data.cell_pos.y.saturating_add(1),
            KeyCode::Char('k') => self.phrase_editor_data.cell_pos.y = self.phrase_editor_data.cell_pos.y.saturating_sub(1),

            KeyCode::Char('h') => self.phrase_editor_data.cell_pos.x = self.phrase_editor_data.cell_pos.x.saturating_sub(1),
            KeyCode::Char('l') => self.phrase_editor_data.cell_pos.x = self.phrase_editor_data.cell_pos.x.saturating_add(1),

            KeyCode::Char('J') => self.phrase_editor_data.cam_pos.y = self.phrase_editor_data.cam_pos.y.saturating_add(1),
            KeyCode::Char('K') => self.phrase_editor_data.cam_pos.y = self.phrase_editor_data.cam_pos.y.saturating_sub(1),
            KeyCode::Char('H') => self.phrase_editor_data.cam_pos.x = self.phrase_editor_data.cam_pos.x.saturating_sub(1),
            KeyCode::Char('L') => self.phrase_editor_data.cam_pos.x = self.phrase_editor_data.cam_pos.x.saturating_add(1),
            _ => {}
        }
    }
}

impl Widget for &mut RsynthTuiApp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let phrase_editor = PhraseEditor::new(&mut self.phrase_editor_data, &self.phrase);
        phrase_editor.render(area, buf);
        let title = Line::from(" Counter App Tutorial ".bold());
        let instructions = Line::from(vec![
            " Decrement ".into(),
            "<J>".blue().bold(),
            " Increment ".into(),
            "<K>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let note_string = if self.write_sharp {
            self.note.to_string_sharps()
        } else {
            self.note.to_string_flats()
        };
        let counter_text = Text::from(vec![
            Line::from(vec![
                "Value: ".into(),
                note_string.yellow(),
            ]),
        ]);

        Paragraph::new(counter_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

// plays a fun little pattern lol
fn play_pattern() {
    static PATTERN: OnceLock<&'static Pattern> = OnceLock::new();

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");

    let sample_rate = 48000;
    let supported_config = supported_configs_range
        .find(|config| config.try_with_sample_rate(sample_rate).is_some())
        .expect("no supported config?!")
        .with_sample_rate(sample_rate);
    let config: StreamConfig = supported_config.into();

    let synth = (PolyphonicOscSynth::sine_specification().generate_synth)();
    let &pattern_ref = PATTERN.get_or_init(|| {
        let mut pattern = Box::new(Pattern::new());
        let composition: &[(&[&str], PatternTime, PatternDuration)] = &[
            (
                &["d4", "f4", "g4", "c5"],
                PatternTime::from_subdivision_count(4, 0).unwrap(),
                PatternDuration::from_subdivision_count(4, 1).unwrap()
            ),
            (
                &["d4", "f4", "g4", "b4"],
                PatternTime::from_subdivision_count(4, 1).unwrap(),
                PatternDuration::from_subdivision_count(4, 1).unwrap()
            ),
            (
                &["c4", "e4", "a4", "b4", "d5"],
                PatternTime::from_subdivision_count(4, 2).unwrap(),
                PatternDuration::from_subdivision_count(4, 2).unwrap()
            ),
        ];
        for (notes, start, duration) in composition {
            let end = *start + *duration;
            for note in *notes {
                pattern.add_note(
                    Note::from_str(note).unwrap(),
                    *start,
                    end
                );
            }
        }
        Box::leak(pattern)
    });

    let mut pattern_player = PatternPlayer::new(
        (&pattern_ref).command_iter(),
        synth,
        0.2,
        sample_rate
    );

    let stream = device.build_output_stream(
        &config,
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            pattern_player.generate_samples(data);
        },
        move |_err| {
            println!("ERROR BRUH")
        },
        None // None=blocking, Some(Duration)=timeout
    ).expect("Could not build stream.");
    let _ = stream.play().expect("stream could not play");

    loop {}
}

fn run_synth_test {
    let specification_map = Arc::new(HashMap::from([
        (0, Box::new(PolyphonicOscSynth::sine_specification())),
        (1, Box::new(PolyphonicOscSynth::square_specification())),
        (2, Box::new(PolyphonicOscSynth::saw_specification())),
    ]));

    let native_options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "RSynth",
        native_options,
        Box::new(|cc| Ok(Box::new(RsynthApp::new(
            cc,
            specification_map
        ))))
    );
}

struct RsynthApp {
    pressed_notes: HashSet<Note>,
    sender: Sender<SynthManagerCommand>,
    reciever: Receiver<SynthManagerMessage>,

    #[allow(dead_code)]
    stream: Stream,
}

impl RsynthApp {
    pub fn new(
        _cc: &eframe::CreationContext,
        synth_specs: Arc<HashMap<u32, Box<SynthesizerSpecification>>>
    ) -> Self {
        let (command_sender, command_reciever) = mpsc::channel();
        let (mut message_sender, message_reciever) = mpsc::channel();
        let _ = command_sender.send(SynthManagerCommand::AddSynth{synth_type_id: 2});

        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available");
        let mut supported_configs_range = device.supported_output_configs()
            .expect("error while querying configs");

        let sample_rate = 48000;
        let supported_config = supported_configs_range
            .find(|config| config.try_with_sample_rate(sample_rate).is_some())
            .expect("no supported config?!")
            .with_sample_rate(sample_rate);
        let config: StreamConfig = supported_config.into();

        let mut manager = SynthManager::new(synth_specs.clone());

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                while let Ok(command) = command_reciever.try_recv() {
                    manager.handle_command(command, &mut message_sender);
                }
                manager.generate_samples(data, sample_rate);
            },
            move |_err| {
                println!("ERROR BRUH")
            },
            None // None=blocking, Some(Duration)=timeout
        ).expect("Could not build stream.");
        let _ = stream.play().expect("stream could not play");

        Self {
            pressed_notes: HashSet::new(),
            sender: command_sender,
            reciever: message_reciever,
            stream
        }
    }

    pub fn key_to_note(key: &str) -> Option<Note> {
        let multiplier = if key == "Comma" {
            12
        } else {
            if key.len() != 1 {
                return None;
            }
            match key.chars().next().expect("key.len == 1") {
                'Z' => 0,
                'S' => 1,
                'X' => 2,
                'D' => 3,
                'C' => 4,
                'V' => 5,
                'G' => 6,
                'B' => 7,
                'H' => 8,
                'N' => 9,
                'J' => 10,
                'M' => 11,
                ',' => 12,
                'Q' => 12,
                '2' => 13,
                'W' => 14,
                '3' => 15,
                'E' => 16,
                'R' => 17,
                '5' => 18,
                'T' => 19,
                '6' => 20,
                'Y' => 21,
                '7' => 22,
                'U' => 23,
                'I' => 24,
                _ => return None,
            }
        };
        Some(Note::from_semitone_delta_a4(multiplier - 9))
    }
}

impl eframe::App for RsynthApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.label("hi")
            });
    }

    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        for event in _raw_input.events.iter() {
            if let Event::Key {
                key, 
                physical_key: _,
                pressed,
                repeat: _,
                modifiers: _
            } = event {
                let name = key.name();
                if let Some(note) = Self::key_to_note(name) {
                    if *pressed {
                        if self.pressed_notes.insert(note) {
                            let _ = self.sender.send(SynthManagerCommand::PlayNote{synth_id: SynthId(0), note});
                        }
                    } else {
                        if self.pressed_notes.remove(&note) {
                            let _ = self.sender.send(SynthManagerCommand::StopNote{synth_id: SynthId(0), note});
                        }
                    }
                }
            }
        }
    }
}
*/
