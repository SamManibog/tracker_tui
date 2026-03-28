use std::collections::VecDeque;

use crate::Note;

/// a synthesizer
pub trait Synthesizer: Send {
    /// tell the syntheziser to begin playing a note
    fn start_playing_note(&mut self, note: Note);

    /// tell the synthesizer to stop playing a note
    fn stop_playing_note(&mut self, note: Note);

    /// tell the synthesizer to stop playing all notes
    fn stop_all(&mut self);

    /// generate samples for the buffer at the given sample rate
    /// when implementing, be sure to add the generated sample to the buffer
    /// do not set the buffer's value
    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32);
}

/// a synthesizer that generates samples using multiple fixed oscillators
pub struct PolyphonicOscSynth<T: FixedOscillator> {
    max_voices: usize,
    a4: f64,
	oscillators: VecDeque<(Note, T)>
}

impl<T: FixedOscillator> PolyphonicOscSynth<T> {
    pub const AMPLITUDE_MULTIPLIER: f32 = 0.05;

    pub fn new(max_voices: usize, a4: f64) -> Self {
        Self {
            max_voices,
            a4,
            oscillators: VecDeque::new()
        }
    }
}

impl<T: FixedOscillator> Synthesizer for PolyphonicOscSynth<T> {
    fn start_playing_note(&mut self, note: Note) {
        for (existing_note, _) in self.oscillators.iter() {
            if *existing_note == note {
                return;
            }
        }
        if self.oscillators.len() >= self.max_voices {
            self.oscillators.pop_front();
        }
        self.oscillators.push_back((
            note,
            T::new(note.frequency(self.a4) as f32)
        ));
    }

    fn stop_playing_note(&mut self, note: Note) {
        for (index, (osc_note, _)) in self.oscillators.iter().enumerate() {
            if *osc_note == note {
                self.oscillators.remove(index);
                return;
            }
        }
    }

    fn generate_samples(&mut self, buffer: &mut [f32], sample_rate: u32) {
        for oscillator in self.oscillators.iter_mut() {
            for sample in buffer.iter_mut() {
                *sample += oscillator.1.generate_sample(sample_rate);
            }
        }
        for sample in buffer.iter_mut() {
            *sample *= Self::AMPLITUDE_MULTIPLIER;
        }
    }

    fn stop_all(&mut self) {
        self.oscillators.clear();
    }
}

/// an oscillator that generates samples for a given note
pub trait FixedOscillator: Send {
    /// creates a new unit for the frequency
    fn new(frequency: f32) -> Self;

    /// generates a single sample for the given sample rate,
    /// assuming an amplitude of 1.0
    fn generate_sample(&mut self, sample_rate: u32) -> f32;
}

/// generates sinewaves
pub struct FixedSineOscillator {
    pub frequency: f32,
    pub index: f32
}

impl FixedOscillator for FixedSineOscillator {
    fn new(frequency: f32) -> Self {
        Self {
            frequency,
            index: 0.0
        }
    }

    fn generate_sample(&mut self, sample_rate: u32) -> f32 {
        let sample = f32::sin(self.index * std::f32::consts::TAU);

        self.index += (1.0 / sample_rate as f32) * self.frequency;
        self.index %= 1.0;

        sample
    }
}

