use std::{collections::BTreeMap, ops::{Add, AddAssign, Mul, MulAssign, RangeBounds, Sub, SubAssign}};

use crate::Note;

/// a timestamp in a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternTime(u64);

impl PatternTime {
	/// the number of breakdowns of the internal representation of a whole_note
    pub const STEPS_PER_WHOLE_NOTE: u64 = 20160;

    pub const ZERO: Self = PatternTime(0);

    /// creates a note time from the given count of the given subdivision
    /// note that not all subdivisions divide cleanly
    pub fn from_subdivision_count(subdivision: u64, count: u64) -> Option<Self> {
        Some(Self(
            Self::STEPS_PER_WHOLE_NOTE.checked_mul(count)? / subdivision
        ))
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_add(rhs.0)?))
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_sub(rhs.0)?))
    }

    /// gets the number of whole notes passed from time 0
    pub fn elapsed_whole_notes(&self) -> f64 {
        1.0 / Self::STEPS_PER_WHOLE_NOTE as f64 * self.0 as f64
    }
}

impl Add<PatternDuration> for PatternTime {
    type Output = Self;

    fn add(self, rhs: PatternDuration) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign<PatternDuration> for PatternTime {
    fn add_assign(&mut self, rhs: PatternDuration) {
        self.0 += rhs.0;
    }
}

impl Sub<PatternDuration> for PatternTime {
    type Output = Self;

    fn sub(self, rhs: PatternDuration) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign<PatternDuration> for PatternTime {
    fn sub_assign(&mut self, rhs: PatternDuration) {
        self.0 -= rhs.0
    }
}

/// a difference in timestamps in a pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PatternDuration(u64);

impl PatternDuration {
	/// the number of breakdowns of the internal representation of a whole_note
    pub const STEPS_PER_WHOLE_NOTE: u64 = 20160;

    pub const ZERO: Self = PatternDuration(0);

    /// a single duration unit
    pub const UNIT: Self = PatternDuration(1);

    /// creats a note time from the given count of the given subdivision
    /// note that not all subdivisions divide cleanly
    pub fn from_subdivision_count(subdivision: u64, count: u64) -> Option<Self> {
        Some(Self(
            Self::STEPS_PER_WHOLE_NOTE.checked_mul(count)? / subdivision
        ))
    }

    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_add(rhs.0)?))
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_sub(rhs.0)?))
    }

    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        Some(Self(self.0.checked_mul(rhs.0)?))
    }

    /// gets the number of whole notes in this duration
    pub fn whole_notes(&self) -> f64 {
        1.0 / Self::STEPS_PER_WHOLE_NOTE as f64 * self.0 as f64
    }
}

impl Add for PatternDuration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for PatternDuration {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0
    }
}

impl Sub for PatternDuration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.abs_diff(rhs.0))
    }
}

impl SubAssign for PatternDuration {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.abs_diff(rhs.0)
    }
}

macro_rules! p_duration_mul_def {
    ( $($x:ty),* ) => {
        $(
        impl Mul<$x> for PatternDuration {
        	type Output = Self;
        	fn mul(self, rhs: $x) -> Self::Output {
        		Self(self.0 * rhs as u64)
        	}
        }

        impl MulAssign<$x> for PatternTime {
        	fn mul_assign(&mut self, rhs: $x) {
        		self.0 *= rhs as u64
        	}
        }
        )*
    };
}
p_duration_mul_def!(u8, u16, u32, u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteId(u32);

impl From<u32> for NoteId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternCommandData {
    Start(Note),
    Stop,
}

impl PatternCommandData {
    pub fn kind(&self) -> PatternCommandKind {
        type K = PatternCommandKind;
        match self {
            Self::Start(_) => K::Start,
            Self::Stop => K::Stop,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternCommandKind {
    Start,
    Stop
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PatternCommand {
    pub note_id: NoteId,
    pub data: PatternCommandData,
}

#[derive(Debug, Clone)]
pub struct PatternNoteData {
    pub start_time: PatternTime,
    pub start_note: Note,
    pub stop_time: PatternTime,
}

/// a pattern of notes
#[derive(Debug, Default)]
pub struct Pattern {
    /// a map from used_ids to the time the note starts and stops
    used_ids: BTreeMap<NoteId, PatternNoteData>,

    /// a map from the time a note starts to the commands at that time
    commands: BTreeMap<PatternTime, Vec<PatternCommand>>,
}

impl Pattern {
    /// creates a new empty pattern
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// adds a command to the command set, preventing duplicates in the same time slot
    fn add_command(&mut self, time: PatternTime, command: PatternCommand) {
        let command_vec = if let Some(command_vec) = self.commands.get_mut(&time) {
            command_vec
        } else {
            self.commands.insert(time, Vec::new());
            self.commands.get_mut(&time).expect("element was just inserted")
        };

        if !command_vec.contains(&command) {
            command_vec.push(command);
        }
    }

    /// removes the specific command type with a note from the timeslot
    /// this matches over NoteCommandType, not checking for exact equality
    #[allow(dead_code)]
    fn remove_command_kind(
        &mut self,
        time: PatternTime,
        note_id: NoteId,
        command_kind: PatternCommandKind
    ) {
        if let Some(command_vec) = self.commands.get_mut(&time) {
            command_vec.retain(|command| {
                command.note_id != note_id || command.data.kind() != command_kind
            });
        }
    }

    /// removes the commands associated with a note from the time slot
    fn remove_associated_commands(&mut self, time: PatternTime, note_id: NoteId) {
        if let Some(command_vec) = self.commands.get_mut(&time) {
            command_vec.retain(|command| {
                command.note_id != note_id
            });
        }
    }

    /// creates a new note with the given start time, duration, and frequency
    /// returns the id of the note if it could be created
    pub fn add_note(
        &mut self,
        note: Note,
        start_time: PatternTime,
        stop_time: PatternTime,
    ) -> Option<NoteId> {
        if PatternTime::ZERO <= start_time  && start_time < stop_time {
            // determine the id to use
            let mut new_id_internal = 0;
            for id in self.used_ids.keys() {
                if id.0 == new_id_internal {
                    new_id_internal += 1;
                } else {
                    break;
                }
            }
            let new_id = NoteId(new_id_internal);

            // insert the commands into the pattern
            self.used_ids.insert(new_id, PatternNoteData {
                start_time,
                start_note: note,
                stop_time
            });
            self.add_command(start_time, PatternCommand {
                note_id: new_id,
                data: PatternCommandData::Start(note),
            });
            self.add_command(stop_time, PatternCommand {
                note_id: new_id,
                data: PatternCommandData::Stop,
            });

            Some(new_id)
        } else {
            None
        }
    }

    /// removes the note from the pattern, returning true if it was removed
    /// and false if the note was not in the pattern
    pub fn remove_note(&mut self, note_id: NoteId) -> bool {
        if let Some(note_data) = self.used_ids.remove(&note_id) {
            self.remove_associated_commands(note_data.start_time, note_id);
            self.remove_associated_commands(note_data.stop_time, note_id);
            true
        } else {
            false
        }
    }

    /// returns an iterator over the commands in the pattern
    pub fn command_iter(&self) -> impl Iterator<Item = (&PatternTime, &Vec<PatternCommand>)> {
        self.commands.iter()
    }

    /// returns an iterator over the commands in the given time range
    pub fn command_range(
        &self,
        range: impl RangeBounds<PatternTime>,
    ) -> impl Iterator<Item = (&PatternTime, &Vec<PatternCommand>)> {
        self.commands.range(range)
    }

    /// checks if the pattern contains notes of the given id
    pub fn contains_note(&self, note_id: NoteId) -> bool {
        self.used_ids.contains_key(&note_id)
    }

    /// gets the data associated with the given note
    pub fn get_note_data(&self, note_id: NoteId) -> Option<&PatternNoteData> {
        self.used_ids.get(&note_id)
    }
}
