use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex, mpsc::Receiver}};

use crate::{NoteId, Phrase, PhraseCommand, PhraseCommandIterator, Synthesizer, arrangement::{Arrangement, InstrumentId, PhraseId}};

/// global playback data
#[derive(Debug)]
pub struct GlobalPlaybackState {
    pub wholes_per_second: f64,
    pub whole_note_delta: f64,
    pub sample_rate: u32,
    pub sample_index: u32,
}

#[derive(Debug)]
pub struct TrackPlaybackState {
    pub phrase: PhraseId,
    pub phrase_whole_note: f64,
}

/// a buffer of phrase commands, derived from a phrase
pub type PhraseCommandBuffer = VecDeque<(f64, Vec<PhraseCommand>)>;

/// constructs a new phrase command buffer from a phrase
pub fn new_command_buffer(phrase: &Phrase) -> PhraseCommandBuffer {
    PhraseCommandIterator::from_phrase(phrase).collect()
}

/// gets a sample based on a phrase command buffer
/// `cmd_buf` is the command buffer of the phrase
/// `synth` is the synthesizer to play the phrase on
/// `whole_note` is the elapsed whole notes since the start of the phrase, before the sample is played
/// `whole_note_delta` is the time that will be added to whole note once the sample is generated
/// returns: (sample, whether this phrase should keep playing)
pub fn get_phrase_sample(
    cmd_buf: &mut PhraseCommandBuffer,
    synth: &mut dyn Synthesizer,
    whole_note: f64,
    whole_note_delta: f64,
) -> (f32, bool) {
    if cmd_buf.is_empty() {
        return (0.0, false);
    }

    let mut keep_playing = true;

    // handle commands
    while let Some((time, _)) = cmd_buf.front() {
        if *time > whole_note {
            break;
        }

        let (_, commands) = cmd_buf.pop_front().unwrap();

        for command in commands.iter() {
            type C = PhraseCommand;
            match command {
                C::StartNote { voice, value } => {
                    synth.start_playing_note(
                        NoteId::from(*voice as u32),
                        value.frequency(440.0)
                    );
                },
                C::LerpNote { voice, duration, end } => {
                    synth.lerp_note(
                        NoteId::from(*voice as u32),
                        end.frequency(440.0),
                        *duration
                    );
                }
                C::StopNote { voice } => {
                    synth.stop_playing_note(NoteId::from(*voice as u32));
                }
                C::Silence => {
                    synth.stop_all();
                }
                C::EndPhrase => {
                    synth.stop_all();
                    keep_playing = false;
                    break;
                }
            }
        }
    }

    // generate sample
    (synth.generate_sample(whole_note_delta), keep_playing)
}

/// Plays a phrase, writing samples to the given buffer.
/// If the phrase commands are exhausted before the buffer is filled, returns Some(index)
/// where index is the index of the first unmodified sample.
/// If the buffer is exhausted before the phrase ends, returns None.
pub fn play_phrase(
    phrase_buffer: &mut PhraseCommandBuffer,
    synth: &mut dyn Synthesizer,
    whole_note: &mut f64,
    whole_note_delta: f64,
    buffer: &mut [f32]
) -> Option<usize> {
    let len = buffer.len();
    let mut buffer_iter = buffer.iter_mut().enumerate();

    let mut logical_whole_note = *whole_note;

    while let (Some((index, left)), Some((_, right))) = (buffer_iter.next(), buffer_iter.next()) {
        let (sample, keep_playing) = get_phrase_sample(
            phrase_buffer,
            synth,
            logical_whole_note,
            whole_note_delta
        );

        logical_whole_note += whole_note_delta;
        *left = sample;
        *right = sample;

        if !keep_playing && index + 2 > len {
            *whole_note += whole_note_delta * (index / 2 + 1) as f64;
            return Some(index + 2);
        }
    }

    *whole_note += whole_note_delta * (len / 2) as f64;

    None
}

pub struct PlaybackState {
    mode: PlaybackMode,
    arrangent: Arc<Mutex<Arrangement>>,
    instruments: Arc<Mutex<HashMap<InstrumentId, Box<dyn Synthesizer>>>>,
    command_receiver: Receiver<PlaybackCommand>,
    sample_rate: u32,
}

impl PlaybackState {
    pub fn new(
        arrangent: Arc<Mutex<Arrangement>>,
        instruments: Arc<Mutex<HashMap<InstrumentId, Box<dyn Synthesizer>>>>,
        command_receiver: Receiver<PlaybackCommand>,
        sample_rate: u32,
    ) -> Self {
        Self {
            mode: PlaybackMode::Off,
            arrangent,
            instruments,
            command_receiver,
            sample_rate,
        }
    }

    fn end_current_mode(&mut self) {
        type M = PlaybackMode;
        match &self.mode {
            M::LoopPhrase(phrase_state) => {
                let mut instrument_lock = self.instruments.lock()
                    .expect("cannot handle poisoned lock on instrument");

                if let Some(instrument) = instrument_lock.get_mut(&phrase_state.instrument_id) {
                    instrument.stop_all();
                }
            },
            M::Off => (),
        }
    }

    fn handle_commands(&mut self) {
        while let Ok(cmd) = self.command_receiver.try_recv() {
            match cmd {
                PlaybackCommand::StopPlayback => {
                    self.end_current_mode();
                    self.mode = PlaybackMode::Off
                },

                PlaybackCommand::LoopPhrase { phrase_id, instrument_id, wholes_per_second } => {
                    self.end_current_mode();

                    let arrangement_lock = self.arrangent.lock()
                        .expect("cannot handle poisoned lock on arrangent");

                    if let Some(phrase) = arrangement_lock.get_phrase(phrase_id) {
                        self.mode = LoopPhraseState {
                            phrase_id,
                            phrase_buffer: new_command_buffer(phrase),
                            instrument_id,
                            whole_note_delta: wholes_per_second / self.sample_rate as f64,
                            whole_note: 0.0,
                        }.into();
                    } else {
                        self.mode = PlaybackMode::Off;
                    }
                }

            }
        }
    }

    pub fn play(&mut self, buffer: &mut [f32]) {
        self.handle_commands();

        if let PlaybackMode::Off = self.mode {
            buffer.fill(0.0);
            return;
        }

        let arrangement_lock = self.arrangent.lock()
            .expect("cannot handle poisoned lock on arrangement");
        let mut instruments_lock = self.instruments.lock()
            .expect("cannot handle poisoned lock on instruments");

        match &mut self.mode {
            PlaybackMode::Off => unreachable!("case already handled above"),

            PlaybackMode::LoopPhrase(state) => {
                state.play(&arrangement_lock, &mut instruments_lock, buffer);
            }
        }
    }
}

#[derive(Debug)]
enum PlaybackMode {
    Off,
    LoopPhrase(LoopPhraseState),
}

/// playback a single phrase on loop
#[derive(Debug)]
struct LoopPhraseState {
    phrase_id: PhraseId,
    phrase_buffer: PhraseCommandBuffer,
    instrument_id: InstrumentId,
    whole_note: f64,
    whole_note_delta: f64,
}

impl From<LoopPhraseState> for PlaybackMode {
    fn from(value: LoopPhraseState) -> Self {
        Self::LoopPhrase(value)
    }
}

impl LoopPhraseState {
    pub fn play(
        &mut self,
        arrangent: &Arrangement,
        instruments: &mut HashMap<InstrumentId, Box<dyn Synthesizer>>,
        buffer: &mut [f32],
    ) {
        if let Some(instrument) = instruments.get_mut(&self.instrument_id) {
            let mut range_start = 0;

            if self.phrase_buffer.len() <= 0 {
                self.whole_note = 0.0;
                if let Some(phrase) = arrangent.get_phrase(self.phrase_id) {
                    self.phrase_buffer = new_command_buffer(phrase);
                } else {
                    buffer[range_start..].fill(0.0);
                    return;
                }
            }

            while let Some(index) = play_phrase(
                &mut self.phrase_buffer,
                instrument.as_mut(),
                &mut self.whole_note,
                self.whole_note_delta,
                &mut buffer[range_start..]
            ) {
                range_start = index;
                self.whole_note = 0.0;

                if let Some(phrase) = arrangent.get_phrase(self.phrase_id) {
                    self.phrase_buffer = new_command_buffer(phrase);
                } else {
                    buffer[range_start..].fill(0.0);
                    return;
                }

            }

        } else {
            buffer.fill(0.0);
        }

    }
}

pub enum PlaybackCommand {
    StopPlayback,
    LoopPhrase{ phrase_id: PhraseId, instrument_id: InstrumentId, wholes_per_second: f64 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackKind {
    Off,
    PhrasePlayback
}
