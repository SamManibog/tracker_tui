use crate::{NoteId, Phrase, PhraseCommand, PhraseCommandIterator, PhraseTransitionMode, Synthesizer};

/// global playback data
#[derive(Debug)]
pub struct GlobalPlaybackState {
    pub wholes_per_second: f64,
    pub sample_rate: u32,
    pub sample_index: u32,
}

/// the state of a voice
#[derive(Debug, Default)]
pub struct VoicePlaybackState {
    /// the transition mode for the currently-playing note
    pub current_transition_mode: PhraseTransitionMode,

    /// the transition mode for the next note to play
    pub next_transition_mode: PhraseTransitionMode,
}


/// the playback state for a single phrase
#[derive(Debug, Default)]
pub struct PhrasePlaybackState {
    /// the next timestep to handle
    pub time_step: u32,

    /// the current sample index
    pub sample_index: u64,

    /// the states of each voice
    pub voice_states: Box<[VoicePlaybackState; Phrase::VOICE_COLUMNS]>,
}

/// Plays a phrase, writing samples to the given buffer.
/// If the phrase commands are exhausted before the buffer is filed, returns Some(index)
/// where index is the index of the first unmodified sample.
/// If the buffer is exhausted before the phrase ends, returns None.
pub fn play_phrase(
    phrase: &Phrase,
    synth: &mut dyn Synthesizer,
    phrase_state: &mut Option<PhrasePlaybackState>,
    global_state: &mut GlobalPlaybackState,
    buffer: &mut [f32]
) -> Option<usize> {
    if phrase_state.is_none() {
        *phrase_state = Some(Default::default());
    }
    let phrase_state = phrase_state.as_mut().unwrap();
    let mut command_iter = PhraseCommandIterator::from_playback_state(phrase, phrase_state);

    let mut buffer_iter = buffer.iter_mut().enumerate();

    while let (Some((index, left)), Some((_, right))) = (buffer_iter.next(), buffer_iter.next()) {
        if let Some(sample) = get_phrase_sample(&mut command_iter, synth, phrase_state, global_state) {
            *left = sample;
            *right = sample;
        } else {
            return Some(index);
        }
    }

    None
}

/// gets the next sample for the phrase
/// if the phrase ends, returns None
fn get_phrase_sample(
    command_iter: &mut PhraseCommandIterator,
    synth: &mut dyn Synthesizer,
    phrase_state: &mut PhrasePlaybackState,
    global_state: &mut GlobalPlaybackState,
) -> Option<f32> {
    // the next time step of commands to handle in whole notes
    let next_time = phrase_state.sample_index as f64 / global_state.sample_rate as f64 * global_state.wholes_per_second;

    // handle commands
    while let Some((time, commands)) = command_iter.peek() {
        if time > next_time {
            break;
        }

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
                        *duration as f64 / global_state.wholes_per_second,
                    );
                }
                C::StopNote { voice } => {
                    synth.stop_playing_note(NoteId::from(*voice as u32));
                }
                C::Silence => {
                    synth.stop_all();
                }
                C::EndPhrase => return None,
            }
        }
        command_iter.next(phrase_state);
    }

    phrase_state.sample_index += 1;

    // generate sample
    Some(synth.generate_sample())
}

