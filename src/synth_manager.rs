use crate::{Note, Synthesizer};

pub enum SynthManagerCommand {
    /// add a synthesizer to the manager
    AddSynth(Box<dyn Synthesizer>),

    /// play a note on the synthesizers
    PlayNote(Note),

    /// stop a note on the synthesizers
    StopNote(Note),

    /// stop playing notes
    StopAll,
}

/// tells synths which notes to play and generates samples
pub struct SynthManager {
    synths: Vec<Box<dyn Synthesizer>>,
}

impl SynthManager {
    pub fn new() -> Self {
        Self {
            synths: Vec::new(),
        }
    }

    pub fn add_synth(&mut self, synth: Box<dyn Synthesizer>) {
        self.synths.push(synth);
    }

    pub fn start_playing_note(&mut self, note: Note) {
        for synth in self.synths.iter_mut() {
            synth.start_playing_note(note);
        }
    }

    pub fn stop_playing_note(&mut self, note: Note) {
        for synth in self.synths.iter_mut() {
            synth.stop_playing_note(note);
        }
    }

    pub fn stop_all(&mut self) {
        for synth in self.synths.iter_mut() {
            synth.stop_all();
        }
    }

    pub fn handle_command(&mut self, command: SynthManagerCommand) {
        type S = SynthManagerCommand;
        match command {
            S::AddSynth(synth)	=> self.add_synth(synth),
            S::PlayNote(note)	=> self.start_playing_note(note),
            S::StopNote(note)	=> self.stop_playing_note(note),
            S::StopAll			=> self.stop_all(),
        }
    }

    pub fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for sample in buffer.iter_mut() {
            *sample = 0.0;
        }
        for synth in self.synths.iter_mut() {
            synth.generate_samples(buffer, sample_rate);
        }
    }
}
