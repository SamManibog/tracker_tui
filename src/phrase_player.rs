use crate::{NoteId, Phrase, PhraseCommand, PhraseCommandIterator, Synthesizer};

/// plays a phrase on a synthesizer
/// implemented as an iterator over samples (mono)
pub struct PhrasePlayer<'a> {
    /// the iterator over the phrase
    phrase_iterator: PhraseCommandIterator<'a>,

    /// the synthesizer used by the phrase player
    synth: &'a mut dyn Synthesizer,

    /// the number of whole notes per second
    wholes_per_second: f64,

    /// the sample number we are on
    sample_index: u32,

    /// the sample rate
    sample_rate: u32
}

unsafe impl Send for PhrasePlayer<'_> {}

impl<'a> PhrasePlayer<'a> {
    /// creates a new phrase
    pub fn new(
        phrase: &'a Phrase,
        synth: &'a mut dyn Synthesizer,
        sample_rate: u32,
        wholes_per_second: f64,
    ) -> Self {
        assert!(wholes_per_second >= 1.0 / 60.0);
        Self {
            phrase_iterator: phrase.iter(),
            synth,
            sample_index: 0,
            sample_rate,
            wholes_per_second,
        }
    }
}

impl<'a> Iterator for PhrasePlayer<'a> {
    type Item = f32;

    /// generates the next sample in the phrase
    /// this function never returns none
    fn next(&mut self) -> Option<Self::Item> {
        // the next time step of commands to handle in whole notes
        let next_time = self.sample_index as f64 / self.sample_rate as f64 * self.wholes_per_second;

        // handle commands
        while let Some((time, commands)) = self.phrase_iterator.peek() {
            if time > next_time {
                break;
            }

            /*
            println!(
            "cmd_time: {}, next_time: {}, second: {}, sample: {}",
            time,
            next_time,
            self.sample_index as f64 / self.sample_rate as f64,
            self.sample_index
            );
            */

            for command in commands.iter() {
                type C = PhraseCommand;
                match command {
                    C::StartNote { voice, value } => {
                        self.synth.start_playing_note(
                            NoteId::from(*voice as u32),
                            value.frequency(440.0)
                        );
                    },
                    C::LerpNote { voice, duration, end } => {
                        self.synth.lerp_note(
                            NoteId::from(*voice as u32),
                            end.frequency(440.0),
                            *duration as f64 / self.wholes_per_second,
                        );
                    }
                    C::StopNote { voice } => {
                        self.synth.stop_playing_note(NoteId::from(*voice as u32));
                    }
                    C::Silence => {
                        self.synth.stop_all();
                    }
                }
            }
            self.phrase_iterator.next();
        }

        self.sample_index += 1;

        // generate sample
        Some(self.synth.generate_sample())
    }
}
