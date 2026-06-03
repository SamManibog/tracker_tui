use std::{collections::{BTreeMap, VecDeque, btree_map}, fmt::Display, iter::Peekable, str::FromStr, u32};

use crate::Note;

pub type PhraseCommandBuffer = VecDeque<(f64, Vec<PhraseCommand>)>;

/// a vector of voice units, sorted by time
/// voice units cannot occupy the same timeslot
#[derive(Debug, Default, Clone)]
pub struct Voice(BTreeMap<u32, Note>);

impl Voice {
    // gets the note at the given time
    pub fn get_note(&self, time_step: u32) -> Option<Note> {
        self.0.get(&time_step).cloned()
    }

    // sets the given note (or nothing) to play at the given time
    // returns the note originally at that time
    pub fn set_note(&mut self, time_step: u32, note_opt: Option<Note>) -> Option<Note> {
        if let Some(note) = note_opt {
            self.0.insert(time_step, note)
        } else {
            self.0.remove(&time_step)
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct PhraseEffectsBuffer(BTreeMap<u32, Box<[Option<PhraseEffect>; Phrase::FX_COLUMNS]>>);

impl PhraseEffectsBuffer {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    // gets the list of effects at the given time
    pub fn get_effect_list(&self, time_step: u32) -> Option<&[Option<PhraseEffect>]> {
        if let Some(boxed) = self.0.get(&time_step) { 
            Some(boxed.as_slice())
        } else {
            None
        }
    }

    // sets the given note (or nothing) to play at the given time
    // returns the note originally at that time
    pub fn set_effect(
        &mut self,
        time_step: u32,
        column: usize,
        fx_opt: Option<PhraseEffect>
    ) -> Option<PhraseEffect> {
        // get existing or create new slot
        let slot = if let Some(slot) = self.0.get_mut(&time_step) {
            slot
        } else {
            let slot = Box::new([{None}; 8]);
            self.0.insert(time_step, slot);
            self.0.get_mut(&time_step).unwrap()
        };

        // set effect
        if let Some(old_fx) = slot.get_mut(column) {
            let output = *old_fx;
            *old_fx = fx_opt;
            output
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseEffect {
    /// set the transition type for all notes created while active
    SetTransition(PhraseTransitionMode),

    /// detune the next note by the given amount in cents
    Detune(i32),

    /// stop all voices
    Silence
}

impl PhraseEffect {
    /// returns the 3-character abbreviation of the effect
    pub fn abbreviate(&self) -> String {
        match self {
            Self::SetTransition(transition) => transition.abbreviate().to_string(),
            Self::Silence => 	"shh".to_string(),
            Self::Detune(cents) => {
                let center = 0x80;
                format!(
                    "dtn:{:02X}",
                    center + cents
                )
            },
        }
    }
}

impl Display for PhraseEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            PhraseEffect::SetTransition(transition) => transition.to_string(),

            PhraseEffect::Silence => "Silence".to_string(),

            PhraseEffect::Detune(cents) => format!(
                "Detune {} ¢",
                cents
            ),
        };

        write!(f, "{}", text)
    }
}

impl FromStr for PhraseEffect {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 6 {
            let s_ascii = s.as_bytes();
            if s_ascii[3] == ':' as u8 {
                if s_ascii[4].is_ascii_hexdigit() && s_ascii[5].is_ascii_hexdigit() {
                    let digit1 = (s_ascii[4] as char).to_digit(16).unwrap() as u8;
                    let digit2 = (s_ascii[5] as char).to_digit(16).unwrap() as u8;

                    let out = match &s[0..3] {
                        "dtn" => PhraseEffect::Detune(digit1 as i32 * 16 + digit2 as i32 - 0x80),
                        _ => return Err(()),
                    };

                    return Ok(out);

                } else {
                    return Err(());
                }
            }
        }

        let out = match s {
            "rel" => PhraseEffect::SetTransition(PhraseTransitionMode::Release),
            "lrp" => PhraseEffect::SetTransition(PhraseTransitionMode::Lerp),
            "shh" => PhraseEffect::Silence,
            "dtn" => PhraseEffect::Detune(0),
            _ => return Err(()),
        };

        return Ok(out);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseTransitionMode {
    /// enable release-right-before note transitions
    Release,

    /// enable lerp note transitions
    Lerp,
}

impl Default for PhraseTransitionMode {
    fn default() -> Self {
        Self::Release
    }
}

impl Display for PhraseTransitionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            PhraseTransitionMode::Release => "Release",
            PhraseTransitionMode::Lerp => "Linear Interpolate",
        };

        write!(f, "{}", text)
    }
}

impl PhraseTransitionMode {
    /// returns the 3-character abbreviation of the transition
    pub fn abbreviate(&self) -> &str {
        match self {
            Self::Release =>	"rel",
            Self::Lerp =>		"lrp",
        }
    }
}

/// a phrase in a tracker
/// phrases can have a custom duration specified using an amount of sixteenth notes
/// phrases also have a variable subdivision amount
#[derive(Debug, Clone)]
pub struct Phrase {
    /// the duration of the phrase in ticks see TICKS_PER_WHOLE_NOTE
    duration: u32,

    /// the number of subdivisions in the phrase
    subdivisions: u32,

    /// the voices in the phrase
    voices: Box<[Voice; Self::VOICE_COLUMNS]>,

    /// the effects in the phrase
    effects: PhraseEffectsBuffer,
}

impl Phrase {
    /// the number of duration ticks per whole note
    /// if you have a duration equal to this value, you have a whole note duration
    pub const TICKS_PER_WHOLE_NOTE: u32 = 16;

    /// the maximum duration of a phrase in 16th notes
    pub const MAX_DURATION: u32 = 16 * 16;

    /// MAX_DURATION * MAX_SUBDIVISION_MULTIPLIER is the maximum number of subdivisions
    /// for a phrase
    pub const MAX_SUBDIVISION_MULTIPLIER: u32 = 1;
    
    /// the number of voices in a phrase
    pub const VOICE_COLUMNS: usize = 8;

    /// the maximum number of fx columns in a phrase
    pub const FX_COLUMNS: usize = 8;

    /// the maximum number of subdivisions given the duration
    pub const fn duration_max_subdivisions(duration: u32) -> u32 {
        duration * Self::MAX_SUBDIVISION_MULTIPLIER
	}

    /// gets the duration in ticks
    pub const fn duration(&self) -> u32 {
        self.duration
    }

    /// gets the number of subdivisions
    pub const fn subdivisions(&self) -> u32 {
        self.subdivisions
    }

    /// gets the voices in the phrase
    pub const fn voices(&self) -> &[Voice] {
        self.voices.as_slice()
    }

    /// gets the effects in the phrase
    pub const fn effects(&self) -> &PhraseEffectsBuffer {
        &self.effects
    }

    /// gets the number of voices in the phrase. the voices may be empty
    pub const fn voice_count(&self) -> usize {
        self.voices.len()
    }

    /// creates a new phrase with the given duration and subdivisions
    /// panics if duration and subdivisions are too large or 0
    /// see TICKS_PER_WHOLE_NOTE to understand how to calculate duration
    pub fn new(duration: u32, subdivisions: u32) -> Self {
        Self {
            duration,
            subdivisions,
            voices: Box::new(Default::default()),
            effects: PhraseEffectsBuffer::new(),
        }
    }

    /// sets the effect at the given time and column
    pub fn set_effect(
        &mut self,
        time_step: u32,
        column: usize,
        fx_opt: Option<PhraseEffect>
    ) -> Option<PhraseEffect> {
        debug_assert!(column <= Phrase::FX_COLUMNS);

        // using <= because we want end-of-phrase effects
        if time_step <= self.subdivisions {
            self.effects.set_effect(time_step, column, fx_opt)
        } else {
            None
        }
    }

    /// sets the note at the given time and column
    pub fn set_note(
        &mut self,
        time_step: u32,
        column: usize,
        note_opt: Option<Note>
    ) -> Option<Note> {
        debug_assert!(column <= Phrase::FX_COLUMNS);
        if let Some(voice) = self.voices.get_mut(column) {

            // using < because we only want as many time steps for
            // notes as there are subdivisions
            if time_step <= self.subdivisions {
                voice.set_note(time_step, note_opt)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn buffer(&self) -> PhraseCommandBuffer {
        PhraseCommandIterator::from_phrase(self).collect()
    }

}

#[derive(Debug, Clone)]
pub enum PhraseCommand {
    // start a note in the given voice
    StartNote{ voice: u8, value: Note },

    // linear interpolate over the duration in whole notes
    LerpNote{ voice: u8, duration: f64, end: Note },

    // stop a note
    StopNote{ voice: u8 },

    // silence all notes
    Silence,
    
    // end the current phrase
    EndPhrase,
}

/// the state of a voice
#[derive(Debug, Default)]
pub struct VoicePlaybackState {
    /// the transition mode for the currently-playing note
    /// none if no note is playing
    pub current_transition_mode: Option<PhraseTransitionMode>,

    /// the transition mode for the next note to play
    pub next_transition_mode: PhraseTransitionMode,

    /// the detune of the next note in cents
    pub detune_cents: i32,
}

/// an iterator over the PhraseOutputCommands in a phrase
/// if storing this, it is recommended to keep this in a box; its quite large
#[derive(Debug)]
struct PhraseCommandIterator<'a> {
    /// the a vector of (original transition when creating previous note, iterator over notes)
    voice_iters: Vec<Peekable<btree_map::Iter<'a, u32, Note>>>,

    /// the states of each voice
    voice_states: Box<[VoicePlaybackState; Phrase::VOICE_COLUMNS]>,
    
    /// an iterator over the effects on each step
    fx_iter: Peekable<btree_map::Iter<'a, u32, Box<[Option<PhraseEffect>; Phrase::FX_COLUMNS]>>>,

    /// the next output of the iterator
    /// if empty, the iterator should return none
    next: Vec<PhraseCommand>,

    /// the number of the next step
    /// u32::MAX serves as a senteniel value meaning iteration should stop
    next_step: u32,

    /// the time that the next step occurs
    next_time: f64,

    /// the total number of subdivisions in the phrase
    subdivisions: u32,

    /// the duration of the phrase
    duration: u32,
}

impl<'a> PhraseCommandIterator<'a> {
    /// creates a new command iterator over all commands in a phrase
    pub fn from_phrase(phrase: &'a Phrase) -> Self {
        let mut voice_iters = Vec::new();
        for voice in phrase.voices.iter() {
            voice_iters.push(
                voice.0.iter()
                    .peekable()
            );
        }
        let fx_iter = phrase.effects.0.iter()
            .peekable();

        let mut output = Self {
            voice_iters,
            fx_iter,
            voice_states: Default::default(),
            next: Vec::new(),
            next_step: 0,
            next_time: 0.0,
            subdivisions: phrase.subdivisions,
            duration: phrase.duration,
        };
        output.calculate_next();
        output
    }

    /// fills the next, next_subdivision, and next_time fields
    /// assumes that self.next is empty
    fn calculate_next(&mut self) {
        debug_assert!(self.next.is_empty(), "next should be empty before calculate_next call");

        // senteniel value exit
        if self.next_step == u32::MAX {
            return;
        }

        type C = PhraseCommand;

        // calculate next_step field
        self.next_step = self.subdivisions;
        if let Some((time, _)) = self.fx_iter.peek() {
            self.next_step = self.next_step.min(**time);
        }
        for voice_iter in &mut self.voice_iters {
            if let Some((time, _)) = voice_iter.peek() {
                self.next_step = self.next_step.min(**time);
            }
        }

        // calculate next_time in whole notes
        self.next_time = (self.next_step * self.duration) as f64
            / self.subdivisions as f64 / Phrase::TICKS_PER_WHOLE_NOTE as f64;

        // calculate next field

        // handle commands
        if let Some((time, fx_list)) = self.fx_iter.peek() {
            if **time == self.next_step {

                for (voice, fx_opt) in fx_list.iter().enumerate() {
                    if let Some(fx) = fx_opt {
                        type E = PhraseEffect;
                        match fx {
                            E::Silence => {
                                self.next.push(C::Silence)
                            },
                            E::SetTransition(transition) => {
                                self.voice_states[voice].next_transition_mode = *transition;
                            }
                            E::Detune(cents) => {
                                self.voice_states[voice].detune_cents = *cents;
                            },
                        };
                    }
                }
                self.fx_iter.next();
            }
        }

        // handle voices
        let mut voice: u8 = 0;
        if self.next_step != self.subdivisions {
            for voice_iter in &mut self.voice_iters {
                if let Some((time, _)) = voice_iter.peek() {
                    if **time == self.next_step {

                        let (time, note) = voice_iter.next().unwrap();
                        let current_transition = &mut self.voice_states[voice as usize].current_transition_mode;
                        let next_transition = self.voice_states[voice as usize].next_transition_mode;
                        let note = note.add_cents(std::mem::take(
                            &mut self.voice_states[voice as usize].detune_cents
                        ));

                        // handle original transition mode: release
                        if let Some(PhraseTransitionMode::Release) = current_transition {
                            self.next.push(C::StopNote { voice: voice });
                            self.next.push(C::StartNote { voice: voice, value: note });
                        } else if current_transition.is_none() {
                            self.next.push(C::StartNote { voice: voice, value: note });
                        }

                        // save the note's new transition
                        *current_transition = Some(next_transition);

                        // handle current lerp transition mode
                        if next_transition == PhraseTransitionMode::Lerp &&
                        let Some((next_time, next_note)) = voice_iter.peek() {

                            // the duration of the transition whole notes
                            let duration_ticks = **next_time - time;
                            let duration = (duration_ticks * self.duration) as f64
                            / self.subdivisions as f64 / Phrase::TICKS_PER_WHOLE_NOTE as f64;

                            self.next.push(C::LerpNote {
                                voice,
                                duration,
                                end: **next_note, 
                            })
                        }

                    }
                }
                voice += 1;
            }
        }

        // if we iterated through all commands, end the phrase
        if self.next_step == self.subdivisions {
            self.next_step = u32::MAX;
            self.next.push(C::EndPhrase);
        } else if self.next.is_empty() {
            // recalculate if empty (some rare circumstances can lead to this, but we don't want to
            // prematurely end; note that amortized iteration time is still O(n))
            self.calculate_next();
        }
    }

}

impl<'a> Iterator for PhraseCommandIterator<'a> {
    type Item = (f64, Vec<PhraseCommand>);

    /// gets the next item, updating state to match
    /// on the iteration immediately before ending, the vector will contain
    /// an endphrase command as the last element
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_empty() {
            None
        } else {
            let mut output = (self.next_time, Vec::new());
            std::mem::swap(&mut output.1, &mut self.next);
            self.calculate_next();
            Some(output)
        }
    }
}

