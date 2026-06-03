use std::collections::{HashMap, VecDeque};

use crate::{NoteId, OutputStreamState, SynthParamId, SynthParamSpecification, Synthesizer, SynthesizerSpecification, note};

/// a wavetable
pub trait Wavetable {
    fn get_sample(&self, index: f32) -> f32;
}

pub struct SineWave();
impl Wavetable for SineWave {
    fn get_sample(&self, index: f32) -> f32 {
        f32::sin(index * std::f32::consts::TAU)
    }
}

pub struct SquareWave();
impl Wavetable for SquareWave {
    fn get_sample(&self, index: f32) -> f32 {
        if index < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

pub struct SawWave();
impl Wavetable for SawWave {
    fn get_sample(&self, index: f32) -> f32 {
        index
    }
}

/// a synthesizer that generates samples using multiple fixed oscillators
pub struct PolyphonicOscSynth {
    sample_rate: u32,

    wavetable: Box<dyn Wavetable + Send>,
    declick_duration: f64,
    declick_samples: u32,
    max_voices: usize,

    /// the note being played, its oscillator, and the declick level
	oscillators: VecDeque<(NoteId, Box<VoiceData>)>,

    /// the notes being stopped and their declick level
    /// should be sorted by declick level
    stopping_oscillators: VecDeque<Box<VoiceData>>,
}

struct VoiceData {
    index: f32,
    start_freq: f32,
    delta_freq: f32,
    target_freq: f32,
    wholes_remaining: f64,
    lerp_duration: f64,
    declick: u32,
}

impl VoiceData {
    fn new(delta_freq: f32) -> Self {
        Self {
            index: 0.0,
            delta_freq,
            start_freq: delta_freq,
            target_freq: delta_freq,
            wholes_remaining: 0.0,
            lerp_duration: 1.0,
            declick: 0,
        }
    }

    fn set_delta_freq(&mut self, delta_freq: f32) {
        self.delta_freq = delta_freq;
        self.start_freq = delta_freq;
        self.target_freq = delta_freq;

        self.lerp_duration = 1.0;
        self.wholes_remaining = 1.0;
    }

    fn set_lerp(&mut self, delta_freq: f32, duration: f64) {
        self.wholes_remaining = duration;
        self.lerp_duration = duration;

        self.start_freq = self.delta_freq;
        self.target_freq = delta_freq;
    }

    fn generate_sample_no_lerp(
        &mut self,
        wavetable: &dyn Wavetable,
        sample_rate: u32,
    ) -> f32 {
        // generate sample
        let sample = wavetable.get_sample(self.index);

        // update index for next sample
        self.index += (1.0 / sample_rate as f32) * note::frequency_from_delta_freq(self.delta_freq);
        self.index %= 1.0;

        sample
    }

    fn generate_sample(
        &mut self,
        wavetable: &dyn Wavetable,
        sample_rate: u32,
        whole_note_delta: f64,
    ) -> f32 {
        // generate sample
        let sample = wavetable.get_sample(self.index);

        // update index for next sample
        self.index += (1.0 / sample_rate as f32) * note::frequency_from_delta_freq(self.delta_freq);
        self.index %= 1.0;

        // update frequency for next sample
        let t = 1.0 - (self.wholes_remaining / self.lerp_duration);
        self.delta_freq = self.start_freq + t as f32 * (self.target_freq - self.start_freq);

        // update whole notes remaining
        self.wholes_remaining = (self.wholes_remaining - whole_note_delta).max(0.0);

        sample
    }
}

impl PolyphonicOscSynth {
    pub const AMPLITUDE_MULTIPLIER: f32 = 0.05;

    pub fn new(
        sample_rate: u32,
        declick_samples: u32,
        max_voices: usize,
        wavetable: Box<dyn Wavetable + Send>
    ) -> Self {
        Self {
            wavetable,
            sample_rate,
            declick_samples,
            declick_duration: declick_samples as f64 / sample_rate as f64,
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
            short_name: "Sine".to_string(),
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
            generate_synth: Box::new(|sample_rate| {
                Box::new(Self::new(
                    sample_rate,
                    240,
                    12,
                    Box::new(SineWave())
                ))
            })
        }
    }

    /// generates the specification for a saw osc synth
    pub fn saw_specification() -> SynthesizerSpecification {
        SynthesizerSpecification {
            name: "Saw Synth".to_string(),
            short_name: "Saw".to_string(),
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
            generate_synth: Box::new(|sample_rate| {
                Box::new(Self::new(
                    sample_rate,
                    240,
                    12,
                    Box::new(SawWave())
                ))
            })
        }
    }

    /// generates the specification for a square osc synth
    pub fn square_specification() -> SynthesizerSpecification {
        SynthesizerSpecification {
            name: "Squarewave Synth".to_string(),
            short_name: "Square".to_string(),
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
            generate_synth: Box::new(|sample_rate| {
                Box::new(Self::new(
                    sample_rate,
                    240,
                    12,
                    Box::new(SquareWave())
                ))
            })
        }
    }
}

impl Synthesizer for PolyphonicOscSynth {
    fn lerp_note(&mut self, note_id: NoteId, delta_freq: f64, duration: f64) {
        if let Some((_, voice)) = self.oscillators.iter_mut()
            .find(|(osc_note_id, _)| {
                *osc_note_id == note_id
            }) {

            let true_duration = if duration <= 0.0 {
                1.0
            } else {
                    duration
                };

            voice.set_lerp(delta_freq as f32, true_duration);
        }
    }

    fn set_stream_state(&mut self, state: &OutputStreamState) {
        self.sample_rate = state.sample_rate;
    }
    
    fn start_playing_note(&mut self, note_id: NoteId, freq: f64) {
        // ensure that note is not already playing
        for (osc_note_id, _) in self.oscillators.iter() {
            if *osc_note_id == note_id {
                return;
            }
        }

        // ensure that voice_count is in the valid range,
        // by possibly removing the oldest oscillator
        if self.oscillators.len() >= self.max_voices {
            if let Some((_, voice)) = self.oscillators.pop_front() {
                self.stopping_oscillators.push_back(voice);
            }
        }

        // add note
        self.oscillators.push_back((
            note_id,
            Box::new(VoiceData::new(freq as f32))
        ));
    }

    fn set_note_frequency(&mut self, note_id: NoteId, delta_freq: f64) {
        if let Some((_, voice)) = self.oscillators.iter_mut()
            .find(|(osc_note_id, _)| {
                *osc_note_id == note_id
            }) {
            voice.set_delta_freq(delta_freq as f32);
        }
    }

    fn stop_playing_note(&mut self, note_id: NoteId) {
        for (index, (osc_note_id, _)) in self.oscillators.iter().enumerate() {
            if *osc_note_id == note_id {
                let (_, mut voice) = self.oscillators.remove(index).unwrap();
                voice.set_delta_freq(voice.delta_freq);
                self.stopping_oscillators.push_back(voice);
                return;
            }
        }
    }

    fn generate_sample(&mut self, whole_note_delta: f64) -> f32{
        let mut sample = 0.0;

        // note: iterate through stopping_oscillators first to avoid duplicate samples
        // when removing from oscillators
        self.stopping_oscillators.retain_mut(|voice| {
            let mut oscillator_sample = voice.generate_sample_no_lerp(
                self.wavetable.as_ref(),
                self.sample_rate,
            );

            voice.declick = voice.declick.saturating_sub(1);
            oscillator_sample *= voice.declick as f32 / self.declick_samples as f32;

            sample += oscillator_sample;

            voice.declick > 0
        });

        let mut index = 0;
        while index < self.oscillators.len() {
            let (_, voice) = &mut self.oscillators[index];

            let mut oscillator_sample = voice.generate_sample(
                self.wavetable.as_ref(),
                self.sample_rate,
                whole_note_delta
            );

            voice.declick = (voice.declick + 1).min(self.declick_samples);
            oscillator_sample *= voice.declick as f32 / self.declick_samples as f32;

            sample += oscillator_sample;

            if voice.declick <= 0 {
                let (_, mut voice) = self.oscillators.swap_remove_back(index).unwrap();
                voice.set_delta_freq(voice.delta_freq);
                self.stopping_oscillators.push_back(voice);
            } else {
                index += 1;
            }
        }

        sample * Self::AMPLITUDE_MULTIPLIER
    }

    fn stop_all(&mut self) {
        while let Some((_, mut voice)) = self.oscillators.pop_back() {
            voice.set_delta_freq(voice.delta_freq);
            self.stopping_oscillators.push_back(voice);
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
