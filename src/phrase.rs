use std::{collections::{BTreeMap, btree_map}, iter::Peekable, u32};

use ratatui::{layout::{Position, Rect}, style::{Color, Style}, text::Text, widgets::Widget};

use crate::Note;

/// a vector of voice units, sorted by time
/// voice units cannot occupy the same timeslot
#[derive(Debug, Default, Clone)]
struct Voice(BTreeMap<u32, Note>);

impl Voice {
    // gets the note at the given time
    fn get_note(&self, time_step: u32) -> Option<Note> {
        self.0.get(&time_step).cloned()
    }

    // sets the given note (or nothing) to play at the given time
    // returns the note originally at that time
    fn set_note(&mut self, time_step: u32, note_opt: Option<Note>) -> Option<Note> {
        if let Some(note) = note_opt {
            self.0.insert(time_step, note)
        } else {
            self.0.remove(&time_step)
        }
    }
}

#[derive(Debug, Default, Clone)]
struct PhraseEffectsBuffer(BTreeMap<u32, Box<[Option<PhraseEffect>; Phrase::FX_COLUMNS]>>);

impl PhraseEffectsBuffer {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    // gets the list of effects at the given time
    fn get_effect_list(&self, time_step: u32) -> Option<&[Option<PhraseEffect>]> {
        if let Some(boxed) = self.0.get(&time_step) { 
            Some(boxed.as_slice())
        } else {
            None
        }
    }

    // sets the given note (or nothing) to play at the given time
    // returns the note originally at that time
    fn set_effect(
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

#[derive(Debug, Clone, Copy)]
pub enum PhraseEffect {
    /// set the transition type for all notes created while active
    SetTransition(PhraseTransitionMode),

    /// stop all voices
    Silence
}

impl PhraseEffect {
    /// returns the 3-character abbreviation of the effect
    pub fn abbreviate(&self) -> &str {
        match self {
            Self::SetTransition(transition) => transition.abbreviate(),
            Self::Silence => 	"shh",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseTransitionMode {
    /// enable release-right-before note transitions
    Release,

    /// enable lerp note transitions
    Lerp,
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
    voices: Vec<Voice>,

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
    
    /// the maximum number of voices in a phrase
    pub const MAX_VOICES: usize = 16;

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
            voices: vec![Voice::default()],
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

    /// adds a voice to the phrase, returning true on success
    pub fn add_voice(&mut self) -> bool {
        if self.voices.len() < Self::MAX_VOICES {
            self.voices.push(Voice::default());
            true
        } else {
            false
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
            if time_step < self.subdivisions {
                voice.set_note(time_step, note_opt)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// gets an iterator over the commands in the phrase
    pub fn iter(&self) -> PhraseCommandIterator {
        PhraseCommandIterator::new(self)
    }
}

/// a temporary Ratatui widget used to display a phrase
pub struct PhraseWidget<'a> {
    /// the phrase to draw
    pub phrase: &'a Phrase,

    /// the position of the camera
    pub cam_pos: &'a mut Position,

    /// the position of the selected cell
    pub cell_pos: &'a mut Position,
}

impl<'a> PhraseWidget<'a> {
    pub const VERTICAL_PADDING: usize = 0;
    pub const HORIZONTAL_PADDING: usize = 0;

    pub const NOTE_WIDTH: u16 = 3;
    pub const FX_WIDTH: u16 = 6;

    pub const LINE_NUMBER_COLOR: Color = Color::Yellow;
	pub const EMPTY_COLOR: Color = Color::DarkGray;
    pub const FILLED_COLOR: Color = Color::White;
}

impl Widget for PhraseWidget<'_> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer
    ) where Self: Sized {
        // clamp cell position
        if self.cell_pos.x >= Phrase::FX_COLUMNS as u16{
            if self.cell_pos.x as usize - Phrase::FX_COLUMNS >= self.phrase.voice_count() {
                self.cell_pos.x = (Phrase::FX_COLUMNS + self.phrase.voice_count()) as u16 - 1;
            }
            if self.cell_pos.y >= self.phrase.subdivisions as u16 {
                self.cell_pos.y = self.phrase.subdivisions as u16 - 1;
            }
        } else if self.cell_pos.y > self.phrase.subdivisions as u16 {
            self.cell_pos.y = self.phrase.subdivisions as u16;
        }

        if area.height <= 0 {
            return;
        }

        // clamp camera y position to the correct bounds
        // must have 1+ visible row and contain the current cell
        self.cam_pos.y = self.cam_pos.y.clamp((self.cell_pos.y + 1).saturating_sub(area.height), self.cell_pos.y);

        // the first line number
        let start_line_number = self.cam_pos.y;

        // the last line number (inclusive)
        let end_fx_line_number = (start_line_number + area.height - 1)
            .min(self.phrase.subdivisions as u16);
        let end_note_line_number = (start_line_number + area.height - 1)
            .min(self.phrase.subdivisions as u16 - 1);

        // the number of characters needed for the line number
        let fx_line_number_digits = ((self.phrase.subdivisions + 1).ilog2() as u16) / 4 + 1;
        let note_line_number_digits = ((self.phrase.subdivisions).ilog2() as u16) / 4 + 1;

        // clamp camera x position to the correct bounds (must have 1+ visible column)
        self.cam_pos.x = self.cam_pos.x.min(
            // line numbers and spacing
            fx_line_number_digits + 1

            // fx_columns + spacing
            + Phrase::FX_COLUMNS as u16 * (Self::FX_WIDTH + 1)

            // voices + spacing
            + (self.phrase.voice_count() as u16 - 1) * (Self::NOTE_WIDTH + 1)
        );
        let x_offset = self.cam_pos.x;

        // x_cursor must be greater than this in order to render fully
        let x_threshold = x_offset + fx_line_number_digits + 1;

        // render begining line numbers (camera-stable, no need threshold checking)
        if area.width < fx_line_number_digits {
            return;
        }
        for i in start_line_number..=end_fx_line_number {
            let number_text = Text::from(format!("{:X}", i))
                .style(Style::new().fg(Self::LINE_NUMBER_COLOR))
                .right_aligned();
            number_text.render(Rect::new(area.x, i - start_line_number, fx_line_number_digits, 1), buf);
        }

        // the cursor to draw the current column of notes or effects
        let mut cursor_x = area.x + fx_line_number_digits + 1;

        // render effects
        let empty_fx_text = Text::from("------")
                    .style(Style::new().fg(Self::EMPTY_COLOR));
        let empty_fx_list = &[{None}; Phrase::FX_COLUMNS];
        if cursor_x + Self::FX_WIDTH > area.x + area.width {
            return;
        }
        // iteration over effect_lists before columns has O(nlogn) time complexity
        // iteration over columns then effect lists has O(mnlogn) time complexity
        for i in start_line_number..=end_fx_line_number {
            let mut fx_cursor_x = cursor_x;
            let effect_list = self.phrase.effects.get_effect_list(i as u32).unwrap_or(empty_fx_list);

            'draw_fx: for (column, effect_opt) in effect_list.iter().enumerate() {
                if fx_cursor_x + Self::FX_WIDTH > area.x + area.width {
                    break 'draw_fx;
                }

                // the number of cells we have to render in
                if fx_cursor_x >= x_threshold {
                    let text = if let Some(effect) = effect_opt {
                        &Text::from(format!("{}:ff", effect.abbreviate()))
                            .style(Style::new().fg(Self::FILLED_COLOR))
                    } else {
                        &empty_fx_text
                    };
                    if i == self.cell_pos.y && column as u16 == self.cell_pos.x {
                        text.clone()
                            .patch_style(Style::new().reversed())
                            .render(Rect::new(
                                fx_cursor_x - x_offset,
                                i - start_line_number,
                                Self::FX_WIDTH,
                                1), buf);
                    } else {
                        text.render(Rect::new(
                            fx_cursor_x - x_offset,
                            i - start_line_number,
                            Self::FX_WIDTH,
                            1), buf);
                    };
                }

                fx_cursor_x += Self::FX_WIDTH + 1;
            }
        }

        // second line number
        cursor_x += (Self::FX_WIDTH + 1) * Phrase::FX_COLUMNS as u16;
        if cursor_x + fx_line_number_digits > area.x + area.width {
            return;
        }
        if cursor_x >= x_threshold {
            for i in start_line_number..=end_fx_line_number {
                let number_text = Text::from(format!("{:X}", i))
                    .style(Style::new().fg(Self::LINE_NUMBER_COLOR))
                    .right_aligned();
                number_text.render(Rect::new(
                    cursor_x - x_offset,
                    i - start_line_number,
                    fx_line_number_digits, 1), buf);
            }
        }

        // render voices
        let empty_note_text = Text::from("---")
            .style(Style::new().fg(Self::EMPTY_COLOR));
        cursor_x += fx_line_number_digits + 1;
        for voice in &self.phrase.voices {
            if cursor_x + Self::NOTE_WIDTH > area.x + area.width {
                return;
            }

            if cursor_x >= x_threshold {
                for i in start_line_number..=end_note_line_number {
                    let text = if let Some(note) = voice.get_note(i as u32) {
                        &Text::from(note.to_padded_string_sharps())
                            .style(Style::new().fg(Self::FILLED_COLOR))
                    } else {
                        &empty_note_text
                    };
                    text.render(Rect::new(
                            cursor_x - x_offset,
                            i - start_line_number,
                            Self::NOTE_WIDTH,
                            1), buf);
                }
            }

            cursor_x += Self::NOTE_WIDTH + 1;
        }

        if cursor_x + note_line_number_digits > area.x + area.width {
            return;
        }

        // render line numbers again
        // we do not need to check for x_threshold bounds because of camera positioning rules
        for i in start_line_number..=end_note_line_number {
            let number_text = Text::from(format!("{:X}", i))
                .style(Style::new().fg(Self::LINE_NUMBER_COLOR));
            number_text.render(Rect::new(
                cursor_x - x_offset,
                i - start_line_number,
                note_line_number_digits,
                1), buf);
        }

    }
}

#[derive(Debug, Clone)]
pub enum PhraseCommand {
    // start a note in the given voice
    StartNote{voice: u8, value: Note},

    // linear interpolate over the duration in whole notes
    LerpNote{voice: u8, duration: f64, end: Note},

    // stop a note
    StopNote{voice: u8},

    // silence all notes
    Silence,
}

/// an iterator over the PhraseOutputCommands in a phrase
/// if storing this, it is recommended to keep this in a box; its quite large
#[derive(Debug)]
pub struct PhraseCommandIterator<'a> {
    /// the a vector of (original transition when creating previous note, iterator over notes)
    voice_iters: Vec<(PhraseTransitionMode, Peekable<btree_map::Iter<'a, u32, Note>>)>,
    
    /// an iterator over the effects on each step
    fx_iter: Peekable<btree_map::Iter<'a, u32, Box<[Option<PhraseEffect>; Phrase::FX_COLUMNS]>>>,

    /// the current transition mode
    transition_mode: PhraseTransitionMode,

    /// the next output of the iterator
    /// if empty, the iterator should return none
    next: Vec<PhraseCommand>,

    /// the number of the next step
    next_step: u32,

    /// the time that the next step occurs
    next_time: f64,

    /// the total number of subdivisions in the phrase
    subdivisions: u32,

    /// the duration of the phrase
    duration: u32,
}

impl<'a> PhraseCommandIterator<'a> {
    pub fn new(phrase: &'a Phrase) -> Self {
        let mut voice_iters = Vec::new();
        for voice in &phrase.voices {
            voice_iters.push((
                PhraseTransitionMode::Release,
                voice.0.iter().peekable()
            ));
        }
        let fx_iter = phrase.effects.0.iter().peekable();

        let mut output = Self {
            voice_iters,
            fx_iter,
            transition_mode: PhraseTransitionMode::Release,
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

        type C = PhraseCommand;

        // calculate next_step field
        self.next_step = u32::MAX;
        if let Some((time, _)) = self.fx_iter.peek() {
            self.next_step = self.next_step.min(**time);
        }
        for (_, voice_iter) in &mut self.voice_iters {
            if let Some((time, _)) = voice_iter.peek() {
                self.next_step = self.next_step.min(**time);
            }
        }

        // calculate next field
        if self.next_step == u32::MAX {
            return;
        }

        // handle commands
        if let Some((time, fx_list)) = self.fx_iter.peek() {
            if **time == self.next_step {
                for fx in fx_list.iter().filter_map(|item| *item) {
                    type E = PhraseEffect;
                    match fx {
                        E::Silence => {
                            self.next.push(C::Silence)
                        },
                        E::SetTransition(transition) => {
                            self.transition_mode = transition;
                        }
                    };
                }
                self.fx_iter.next();
            }
        }

        // handle voices
        let mut voice = 0;
        for (og_transition, voice_iter) in &mut self.voice_iters {
            if let Some((time, _)) = voice_iter.peek() {
                if **time == self.next_step {
                    let (time, note) = voice_iter.next().unwrap();

                    // handle original transition mode: release
                    if *og_transition == PhraseTransitionMode::Release {
                    	self.next.push(C::StopNote { voice: voice });
                    	self.next.push(C::StartNote { voice: voice, value: *note });
                    }

                    // save the note's transition
                    *og_transition = self.transition_mode;

                    // handle current lerp transition mode
                    if self.transition_mode == PhraseTransitionMode::Lerp &&
                    	let Some((next_time, next_note)) = voice_iter.peek() {

                        // the duration of the transition in ticks
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

        // calculate next_time in whole notes
        self.next_time = (self.next_step * self.duration) as f64
            / self.subdivisions as f64 / Phrase::TICKS_PER_WHOLE_NOTE as f64;
    }

    /// peeks the next item without iterating
    pub fn peek(&self) -> Option<(f64, &Vec<PhraseCommand>)> {
        if self.next.is_empty() {
            return None;
        }

        Some((self.next_time, &self.next))
    }
}

impl<'a> Iterator for PhraseCommandIterator<'a> {
    type Item = (f64, Vec<PhraseCommand>);

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
