use std::collections::{HashMap, VecDeque};

use crate::{NoteId, OutputStreamState, SynthParamId, SynthParamSpecification, Synthesizer, SynthesizerSpecification};

/// a synthesizer that generates samples using multiple fixed oscillators
pub struct PolyphonicOscSynth {
    sample_rate: u32,

    osc_generator: Box<dyn Fn(f64) -> PhaseIndexOscillator + Send>,
    declick_duration: f64,
    declick_samples: u32,
    max_voices: usize,

    /// the note being played, its oscillator, and the declick level
	oscillators: VecDeque<(NoteId, PhaseIndexOscillator, u32)>,

    /// the notes being stopped and their declick level
    /// should be sorted by declick level
    stopping_oscillators: VecDeque<(PhaseIndexOscillator, u32)>,
}

impl PolyphonicOscSynth {
    pub const AMPLITUDE_MULTIPLIER: f32 = 0.05;

    pub fn new(
        declick_samples: u32,
        max_voices: usize,
        osc_generator: Box<dyn Fn(f64) -> PhaseIndexOscillator + Send>,
    ) -> Self {
        Self {
            sample_rate: 48000,
            osc_generator,
            declick_samples,
            declick_duration: declick_samples as f64 / 48000.0,
            max_voices,
            oscillators: VecDeque::new(),
            stopping_oscillators: VecDeque::new(),
        }
    }

    pub fn set_declick_duration(&mut self, declick_duration: f64) {
        assert!(declick_duration >= 0.0);
        self.declick_duration = declick_duration;
        self.recalculate_declick_samples();
    }

    pub fn set_sample_rate(&mut self, sample_rate: u32) {
        self.sample_rate = sample_rate;
        self.recalculate_declick_samples();
    }

    fn recalculate_declick_samples(&mut self) {
        self.declick_samples = (self.sample_rate as f64 * self.declick_duration).ceil() as u32;
    }

    /// generates the specification for a sine osc synth
    pub fn sine_specification() -> SynthesizerSpecification {
        SynthesizerSpecification {
            name: "Sinewave Synth".to_string(),
            parameters: HashMap::from([
                (
                    SynthParamId(0),
                    SynthParamSpecification::new("declick secs")
                        .min_max(0.0, 0.5)
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
                    Box::new(|freq|
                        PhaseIndexOscillator::new_sine(freq as f32)
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
                    SynthParamSpecification::new("declick secs")
                        .min_max(0.0, 0.5)
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
                    Box::new(|freq|
                        PhaseIndexOscillator::new_saw(freq as f32)
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
                    SynthParamSpecification::new("declick secs")
                        .min_max(0.0, 0.5)
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
                    Box::new(|freq|
                        PhaseIndexOscillator::new_square(freq as f32)
                    )
                ))
            })
        }
    }
}

impl Synthesizer for PolyphonicOscSynth {
    fn lerp_note(&mut self, note_id: NoteId, freq: f64, duration: f64) {
        println!("unimplemented: lerp note {:?} at {} for {}", note_id, freq, duration);
    }

    fn set_stream_state(&mut self, state: &OutputStreamState) {
        self.sample_rate = state.sample_rate;
    }
    
    fn start_playing_note(&mut self, note_id: NoteId, freq: f64) {
        // ensure that note is not already playing
        for (osc_note_id, _, _) in self.oscillators.iter() {
            if *osc_note_id == note_id {
                return;
            }
        }

        // ensure that voice_count is in the valid range,
        // by possibly removing the oldest oscillator
        if self.oscillators.len() >= self.max_voices {
            if let Some((_, osc, declick)) = self.oscillators.pop_front() {
                self.stopping_oscillators.push_back((osc, declick));
            }
        }

        // add note
        self.oscillators.push_back((
            note_id,
            (self.osc_generator)(freq),
            0,
        ));
    }

    fn set_note_frequency(&mut self, note_id: NoteId, freq: f64) {
        if let Some((_, oscillator, _)) = self.oscillators.iter_mut()
            .find(|(osc_note_id, _, _)| {
                *osc_note_id == note_id
            }) {
            oscillator.set_frequency(freq as f32);
        }
    }

    fn stop_playing_note(&mut self, note_id: NoteId) {
        for (index, (osc_note_id, _, _)) in self.oscillators.iter().enumerate() {
            if *osc_note_id == note_id {
                let (_, osc, declick) = self.oscillators.remove(index).unwrap();
                self.stopping_oscillators.push_back((osc, declick));
                return;
            }
        }
    }

    fn generate_sample(&mut self, whole_delta: f64) -> f32{
        let mut sample = 0.0;

        // note: iterate through stopping_oscillators first to avoid duplicate samples
        // when removing from oscillators
        self.stopping_oscillators.retain_mut(|(osc, declick)| {
            let mut oscillator_sample = osc.generate_sample(self.sample_rate);

            *declick = declick.saturating_sub(1);
            oscillator_sample *= *declick as f32 / self.declick_samples as f32;

            sample += oscillator_sample;

            *declick > 0
        });

        let mut index = 0;
        while index < self.oscillators.len() {
            let (_, osc, declick) = &mut self.oscillators[index];

            let mut oscillator_sample = osc.generate_sample(self.sample_rate);

            *declick = (*declick + 1).min(self.declick_samples);
            oscillator_sample *= *declick as f32 / self.declick_samples as f32;

            sample += oscillator_sample;

            if *declick <= 0 {
                let (_, osc, declick) = self.oscillators.swap_remove_back(index).unwrap();
                self.stopping_oscillators.push_back((osc, declick));
            } else {
                index += 1;
            }
        }

        sample * Self::AMPLITUDE_MULTIPLIER
    }

    fn stop_all(&mut self) {
        while let Some((_, osc, declick)) = self.oscillators.pop_back() {
            self.stopping_oscillators.push_back((osc, declick));
        }
    }

    fn set_parameter(&mut self, param_id: SynthParamId, value: f64) {
        match param_id.0 {
            0 => self.declick_samples = value.round() as u32,
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
