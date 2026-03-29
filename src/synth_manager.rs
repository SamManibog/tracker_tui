use std::collections::HashMap;

use crate::{Note, Synthesizer, SynthesizerSpecification};

pub enum SynthManagerCommand {
    /// add a synthesizer to the manager
    AddSynth(u32),

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
    synth_specs: HashMap<u32, Box<SynthesizerSpecification>>,
}

unsafe impl Send for SynthManager {}

impl SynthManager {
    pub fn new(synth_specs: HashMap<u32, Box<SynthesizerSpecification>>) -> Self {
        Self {
            synths: Vec::new(),
            synth_specs,
        }
    }

    pub fn add_synth(&mut self, synth_id: u32) {
        if let Some(specification) = self.synth_specs.get(&synth_id) {
            self.synths.push((specification.generate_synth)());
        }
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
            for synth in self.synths.iter_mut() {
                *sample += synth.generate_sample(sample_rate);
            }
        }
    }
}
