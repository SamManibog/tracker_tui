use std::iter::Peekable;

use crate::{PatternCommand, PatternCommandData, PatternTime, Synthesizer};

pub struct PatternPlayer<'a, T: Iterator<Item = (&'a PatternTime, &'a Vec<PatternCommand>)>> {
    /// the number of samples generated per second
    sample_rate: u32,

    /// the index of the NEXT sample to generate for
    sample_index: u32,

    /// the number of whole notes per second
    whole_notes_per_second: f64,

    /// the synthesizer producing samples
    synth: Box<dyn Synthesizer>,

    /// an iterator over the pattern
    command_iter: Peekable<T>,
}

unsafe impl<'a, T: Iterator<Item = (&'a PatternTime, &'a Vec<PatternCommand>)>> Send for PatternPlayer<'a, T> {}

impl<'a, T: Iterator<Item = (&'a PatternTime, &'a Vec<PatternCommand>)>> PatternPlayer<'a, T> {
    pub fn new(
        pattern_iter: T,
        synth: Box<dyn Synthesizer>,
        whole_notes_per_second: f64,
        sample_rate: u32
    ) -> Self {
        Self {
            command_iter: pattern_iter.peekable(),
            synth,
            whole_notes_per_second,
            sample_rate,
            sample_index: 0
        }
    }

    fn handle_pattern_command(
        synth: &mut Box<dyn Synthesizer>,
        command: &PatternCommand
    ) {
        type D = PatternCommandData;
        let note_id = command.note_id;
        match command.data {
            D::Start(note)	=> synth.start_playing_note(note_id, note),
            D::Stop			=> synth.stop_playing_note(note_id),
        }
    }

    pub fn generate_samples(&mut self, buffer: &mut[f32]) {
        for buffer_sample in buffer.iter_mut() {
            // the next timestamp in elapsed whole notes
            let next_time = self.sample_index as f64 * self.whole_notes_per_second / self.sample_rate as f64;

            // process commands
            'command_loop: while let Some((pattern_time, buffer)) = self.command_iter.peek() {
                let float_time = pattern_time.elapsed_whole_notes();
                if next_time >= float_time {
                    for command in buffer.iter() {
                        Self::handle_pattern_command(
                            &mut self.synth, 
                            &command
                        );
                    }
                    self.command_iter.next();
                } else {
                    break 'command_loop
                }
            }

            // create sample
            *buffer_sample = self.synth.generate_sample(self.sample_rate);

            // advance to next sample
            self.sample_index += 1;
        }
    }
}
