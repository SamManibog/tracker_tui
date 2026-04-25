use std::collections::HashMap;

use crate::NoteId;

/// the id of a parameter on a synthesizer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SynthParamId(pub u32);

/// the specification of a synthesizer parameter, uses the builderLite pattern
#[derive(Debug, Clone)]
pub struct SynthParamSpecification {
    /// the readable name
    pub name: String,

    /// the minimum allowed value
    /// min must be less than max
    pub min: f64,

    /// the maximum allowed value
    /// max must be greater than min
    pub max: f64,

    /// the number of steps taken from the minimum and maximum values
    /// if 0 or 1, signifies a discrete parameter without steps
    pub steps: u32,
}

impl SynthParamSpecification {
    /// creates a new specification with the given name
    /// remember uses the builderlite pattern
    /// by default min = 0.0, max = 1.0, steps = 0 (discrete)
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            min: 0.0,
            max: 0.0,
            steps: 0
        }
    }

    /// builderlite pattern for making specification like an integer,
    /// having integer values
    pub fn int_like(self, min: i64, max: i64) -> Self {
        Self {
            min: min as f64,
            max: max as f64,
            steps: (max - min + 1) as u32,
            ..self
        }
    }

    /// builderlite pattern for setting min and max
    pub fn min_max(self, min: f64, max: f64) -> Self {
        Self {
            min,
            max,
            ..self
        }
    }

    /// builderlite pattern for setting steps
    pub fn steps(self, steps: u32) -> Self {
        Self {
            steps,
            ..self
        }
    }

    /// returns if this is a discrete specification or not
    pub fn is_discrete(&self) -> bool {
        self.steps <= 1
    }

    /// returns if this is a binary specificaiton or not
    pub fn is_binary(&self) -> bool {
        self.steps == 2
    }

    /// adjusts the given value to fit into the allowed range
    /// Nan is treated as 0.0
    pub fn quantize(&self, value: f64) -> f64 {
        // reinterpret Nan as 0.0
        let value = if value.is_nan() {
            0.0
        } else {
            value
        };

        let clamped = value.clamp(self.min, self.max);
        if self.is_discrete() {
            return clamped;
        }

        // the change in value from advacing a single step
        let step_delta = (self.max - self.min) / (self.steps - 1) as f64;

        // the number of steps we have taken
        let step_count = ((clamped - self.min) / step_delta).round();

        self.min + step_delta * step_count
    }

    /// gets the value based on the given step
    pub fn value_by_step(&self, step: u32) -> f64 {
        assert!(
            !self.is_discrete(),
            "cannot get value by step on a discrete parameter"
        );
        let step_delta = (self.max - self.min) / (self.steps - 1) as f64;
        self.min + step_delta * step as f64
    }
}

/// the specification for a synthesizer
pub struct SynthesizerSpecification {
    pub name: String,
    pub generate_synth: Box<dyn Fn() -> Box<dyn Synthesizer> + Send>,
    pub parameters: HashMap<SynthParamId, SynthParamSpecification>,
}

/// a synthesizer
pub trait Synthesizer {
    /// update the syntheziser with the output stream state
    fn set_stream_state(&mut self, state: &OutputStreamState);

    /// get the value of a parameter
    fn get_parameter(&self, param_id: SynthParamId) -> Option<f64>;

    /// tell the synthesizer to set a parameter to a specific value
    fn set_parameter(&mut self, param_id: SynthParamId, value: f64);

    /// tell the syntheziser to begin playing a frequency
    fn start_playing_note(&mut self, note_id: NoteId, freq: f64);

    /// set the frequency of the given note
    fn set_note_frequency(&mut self, note_id: NoteId, freq: f64);

    /// the given note the the frequency over the given duration in whole notes
    fn lerp_note(&mut self, note_id: NoteId, freq: f64, duration: f64);

    /// tell the synthesizer to stop playing a note.
    /// immediately after this is called, the note id it was called for must
    /// note reference the stopped note. this function should ignore calls
    /// to stop notes that are not playing
    fn stop_playing_note(&mut self, note_id: NoteId);

    /// tell the synthesizer to stop playing all notes
    fn stop_all(&mut self);

    /// generate a single sample assuming the given sample rate and whole-note delta
    fn generate_sample(&mut self, whole_delta: f64) -> f32;
}

/// describes the state of the output stream
#[derive(Debug)]
pub struct OutputStreamState {
    /// the number of samples per second
    pub sample_rate: u32,
}

