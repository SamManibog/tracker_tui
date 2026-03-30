use std::{collections::{BTreeMap, HashMap}, sync::{Arc, mpsc::Sender}};

use crate::{Note, Synthesizer, SynthesizerSpecification};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynthId(pub u32);

/// a command sent to a synth manager to handle
pub enum SynthManagerCommand {
    /// add a synthesizer to the manager
    AddSynth{synth_type_id: u32},

    /// play a note on the synthesizers
    PlayNote{synth_id: SynthId, note: Note},

    /// stop a note on the synthesizers
    StopNote{synth_id: SynthId, note: Note},

    /// stop playing notes
    StopAll,
}

/// a message outgoing from a synth manager resulting from a command
pub enum SynthManagerMessage {
    SynthAdded(SynthId),
}

/// tells synths which notes to play and generates samples
pub struct SynthManager {
    synths: BTreeMap<SynthId, Box<dyn Synthesizer>>,
    synth_specs: Arc<HashMap<u32, Box<SynthesizerSpecification>>>,
}

unsafe impl Send for SynthManager {}

impl SynthManager {
    pub fn new(synth_specs: Arc<HashMap<u32, Box<SynthesizerSpecification>>>) -> Self {
        Self {
            synths: BTreeMap::new(),
            synth_specs,
        }
    }

    /// adds a synth, returning the id of the added synth
    pub fn add_synth(&mut self, synth_type_id: u32) -> Option<SynthId> {
        if let Some(specification) = self.synth_specs.get(&synth_type_id) {
            // search for the first unused id
            let mut new_id = 0;
            for id in self.synths.keys() {
                if id.0 == new_id {
                    new_id += 1;
                } else {
                    break;
                }
            }

            // insert with the first unused id
            self.synths.insert(SynthId(new_id), (specification.generate_synth)());
            Some(SynthId(new_id))
        } else {
            None
        }
    }

    pub fn start_playing_note(&mut self, synth_id: SynthId, note: Note) {
        if let Some(synth) = self.synths.get_mut(&synth_id) {
            synth.start_playing_note(note);
        }
    }

    pub fn stop_playing_note(&mut self, synth_id: SynthId, note: Note) {
        if let Some(synth) = self.synths.get_mut(&synth_id) {
            synth.stop_playing_note(note);
        }
    }

    pub fn stop_all(&mut self) {
        for synth in self.synths.values_mut() {
            synth.stop_all();
        }
    }

    pub fn handle_command(&mut self, command: SynthManagerCommand, sender: &mut Sender<SynthManagerMessage>) {
        type C = SynthManagerCommand;
        type M = SynthManagerMessage;
        match command {
            C::AddSynth { synth_type_id }	=> {
                if let Some(id) = self.add_synth(synth_type_id) {
                    let _ = sender.send(M::SynthAdded(id));
                }
            },
            C::PlayNote { synth_id, note }	=> self.start_playing_note(synth_id, note),
            C::StopNote { synth_id, note }	=> self.stop_playing_note(synth_id, note),
            C::StopAll						=> self.stop_all(),
        }
    }

    pub fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for sample in buffer.iter_mut() {
            *sample = 0.0;
            for synth in self.synths.values_mut() {
                *sample += synth.generate_sample(sample_rate);
            }
        }
    }
}
