use std::collections::{HashMap, VecDeque};

use crate::{Note, NoteId, SynthParamId, SynthParamSpecification, Synthesizer, SynthesizerSpecification};

/// a synthesizer that generates samples using multiple fixed oscillators
pub struct PolyphonicOscSynth {
    osc_generator: Box<dyn Fn(Note) -> PhaseIndexOscillator + Send>,
    declick_samples: i32,
	voice_count: usize,
    max_voices: usize,

    /// the note being played, its oscillator, and the declick level
	oscillators: VecDeque<(NoteId, PhaseIndexOscillator, i32)>
}

impl PolyphonicOscSynth {
    pub const AMPLITUDE_MULTIPLIER: f32 = 0.05;

    pub fn new(
        declick_samples: i32,
        max_voices: usize,
        osc_generator: Box<dyn Fn(Note) -> PhaseIndexOscillator + Send>,
    ) -> Self {
        assert!(declick_samples >= 0, "declick_samples must be non-negative");
        Self {
            osc_generator,
            declick_samples,
            voice_count: 0,
            max_voices,
            oscillators: VecDeque::new(),
        }
    }

    /// generates the specification for a sine osc synth
    pub fn sine_specification() -> SynthesizerSpecification {
        SynthesizerSpecification {
            name: "Sinewave Synth".to_string(),
            parameters: HashMap::from([
                (
                    SynthParamId(0),
                    SynthParamSpecification::new("declick")
                        .int_like(0, 48000),
                ),
                (
                    SynthParamId(1),
                    SynthParamSpecification::new("max voices")
                        .int_like(1, 24),
                )
            ]),
            generate_synth: Box::new(|| {
                Box::new(Self::new(
                    240,
                    12,
                    Box::new(|note|
                        PhaseIndexOscillator::new_sine(note.frequency(440.0) as f32)
                    )
                ))
            })
        }
    }

    /// generates the specification for a saw osc synth
    pub fn saw_specification() -> SynthesizerSpecification {
        SynthesizerSpecification {
            name: "Saw Synth".to_string(),
            parameters: HashMap::from([
                (
                    SynthParamId(0),
                    SynthParamSpecification::new("declick")
                        .int_like(0, 48000),
                ),
                (
                    SynthParamId(1),
                    SynthParamSpecification::new("max voices")
                        .int_like(1, 24),
                )
            ]),
            generate_synth: Box::new(|| {
                Box::new(Self::new(
                    240,
                    12,
                    Box::new(|note|
                        PhaseIndexOscillator::new_saw(note.frequency(440.0) as f32)
                    )
                ))
            })
        }
    }

    /// generates the specification for a square osc synth
    pub fn square_specification() -> SynthesizerSpecification {
        SynthesizerSpecification {
            name: "Squarewave Synth".to_string(),
            parameters: HashMap::from([
                (
                    SynthParamId(0),
                    SynthParamSpecification::new("declick")
                        .int_like(0, 48000),
                ),
                (
                    SynthParamId(1),
                    SynthParamSpecification::new("max voices")
                        .int_like(1, 24),
                )
            ]),
            generate_synth: Box::new(|| {
                Box::new(Self::new(
                    240,
                    12,
                    Box::new(|note|
                        PhaseIndexOscillator::new_square(note.frequency(440.0) as f32)
                    )
                ))
            })
        }
    }
}

impl Synthesizer for PolyphonicOscSynth {
    fn start_playing_note(&mut self, note_id: NoteId, note: Note) {
        // ensure that note is not already playing
        for (osc_note_id, _, _) in self.oscillators.iter() {
            if *osc_note_id == note_id {
                return;
            }
        }

        // ensure that voice_count is in the valid range,
        // by possibly removing an oscillator
        if self.voice_count >= self.max_voices {
            if let Some((_, _, declick)) = self.oscillators.iter_mut()
                .find(|(_, _, declick)| {
                    *declick > 0
                }) {
                *declick *= -1;
                self.voice_count -= 1;
            }
        }

        // add note
        self.oscillators.push_back((
            note_id,
            (self.osc_generator)(note),
            0,
        ));
        self.voice_count += 1;
    }

    fn stop_playing_note(&mut self, note_id: NoteId) {
        for (osc_note_id, _, declick) in self.oscillators.iter_mut() {
            if *osc_note_id == note_id {
                *declick *= -1;
                self.voice_count -= 1;
                return;
            }
        }
    }

    fn generate_sample(&mut self, sample_rate: u32) -> f32{
        let mut sample = 0.0;

        self.oscillators.retain_mut(|(_, oscillator, declick)| {
            // generate initial sample for oscillator
            let mut oscillator_sample = oscillator.generate_sample(sample_rate);
            
            // multiply by declick level
            *declick = (*declick + 1).min(self.declick_samples as i32);
            let declick_level = if *declick > 0 {
                *declick
            } else {
                -*declick
            }; 
            oscillator_sample *= declick_level as f32 / self.declick_samples as f32;

            // add to overall sample
            sample += oscillator_sample;

            // remove oscillator if necessary
            *declick != 0
        });

        sample * Self::AMPLITUDE_MULTIPLIER
    }

    fn stop_all(&mut self) {
        self.oscillators.clear();
        for (_, _, declick) in self.oscillators.iter_mut() {
            *declick *= -1;
        }
        self.voice_count = 0;
    }

    fn set_parameter(&mut self, param_id: SynthParamId, value: f64) {
        match param_id.0 {
            0 => self.declick_samples = value.round() as i32,
            1 => self.max_voices = value.round() as usize,
            _ => (),
        }
    }

    fn get_parameter(&self, param_id: SynthParamId) -> Option<f64> {
        match param_id.0 {
            0 => Some(self.declick_samples as f64),
            1 => Some(self.max_voices as f64),
            _ => None,
        }
    }
}

/// an oscillator that generates samples for a given note
pub struct PhaseIndexOscillator {
    callback: Box<dyn Fn(f32) -> f32 + Send>,
    frequency: f32,
    index: f32
}

impl PhaseIndexOscillator {
    /// creates a oscillator with the given frequency and callback
    /// where callback takes a number in [0.0, 1.0] and outputs the amplitude
    /// basically we are getting a sample for an index of the waveform
    pub fn new(frequency: f32, callback: Box<dyn Fn(f32) -> f32 + Send>) -> Self {
        Self {
            callback,
            frequency,
            index: 0.0
        }
    }

    /// creates a sinewave oscillator with the given frequency
    pub fn new_sine(frequency: f32) -> Self {
        Self::new(
            frequency,
            Box::new(|phase_index| {
                f32::sin(phase_index * std::f32::consts::TAU)
            })
        )
    }

    /// creates a squarewave oscillator with the given frequency
    pub fn new_square(frequency: f32) -> Self {
        Self::new(
            frequency,
            Box::new(|phase_index| {
                if phase_index < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            })
        )
    }

    /// creates a saw oscillator with the given frequency
    pub fn new_saw(frequency: f32) -> Self {
        Self::new(
            frequency,
            Box::new(|phase_index| {
                phase_index
            })
        )
    }

    pub fn generate_sample(&mut self, sample_rate: u32) -> f32 {
        let sample = (self.callback)(self.index);

        self.index += (1.0 / sample_rate as f32) * self.frequency;
        self.index %= 1.0;

        sample
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
}
