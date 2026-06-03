use std::fmt::{Debug, Display};

use crate::{Note, Phrase, PhraseEffect};

/// An enum for commands emitted by a phrase editor
#[derive(Debug, Clone)]
pub enum PhraseEditCommand {
    SetEffect(PhraseSetEffect),
    ClearEffects(PhraseClearEffects),
    RestoreEffects(PhraseRestoreEffects),
    SetNote(PhraseSetNote),
    ClearNotes(PhraseClearNotes),
    RestoreNotes(PhraseRestoreNotes),
}

impl std::fmt::Display for PhraseEditCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetEffect(cmd) => std::fmt::Display::fmt(cmd, f),
            Self::ClearEffects(cmd) => std::fmt::Display::fmt(cmd, f),
            Self::RestoreEffects(cmd) => std::fmt::Display::fmt(cmd, f),
            Self::SetNote(cmd) => std::fmt::Display::fmt(cmd, f),
            Self::ClearNotes(cmd) => std::fmt::Display::fmt(cmd, f),
            Self::RestoreNotes(cmd) => std::fmt::Display::fmt(cmd, f),
        }
    }
}

impl PhraseEditCommand {
    /// execute the command, and return a command to undo the execution if a change was made
    pub fn execute(&self, phrase: &mut Phrase) -> Option<Self> {
        match self {
            Self::SetEffect(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            Self::ClearEffects(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            Self::RestoreEffects(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            Self::SetNote(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            Self::ClearNotes(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            Self::RestoreNotes(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
        }
    }
}

impl From<PhraseSetEffect> for PhraseEditCommand {
    fn from(cmd: PhraseSetEffect) -> Self {
        Self::SetEffect(cmd)
    }
}
impl From<PhraseClearEffects> for PhraseEditCommand {
    fn from(cmd: PhraseClearEffects) -> Self {
        Self::ClearEffects(cmd)
    }
}
impl From<PhraseRestoreEffects> for PhraseEditCommand {
    fn from(cmd: PhraseRestoreEffects) -> Self {
        Self::RestoreEffects(cmd)
    }
}
impl From<PhraseSetNote> for PhraseEditCommand {
    fn from(cmd: PhraseSetNote) -> Self {
        Self::SetNote(cmd)
    }
}
impl From<PhraseClearNotes> for PhraseEditCommand {
    fn from(cmd: PhraseClearNotes) -> Self {
        Self::ClearNotes(cmd)
    }
}
impl From<PhraseRestoreNotes> for PhraseEditCommand {
    fn from(cmd: PhraseRestoreNotes) -> Self {
        Self::RestoreNotes(cmd)
    }
}

/// a phrase edit command that sets a single effect
#[derive(Debug, Clone)]
pub struct PhraseSetEffect {
    pub effect: PhraseEffect,
    pub row: u32,
    pub column: usize,
}

impl PhraseSetEffect {
    pub fn new(effect: PhraseEffect, row: u32, column: usize) -> Self {
        Self {
            effect,
            row,
            column
        }
    }
}

impl Display for PhraseSetEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Set r{}:c{} to {}", self.row, self.column, self.effect)
    }
}

impl PhraseSetEffect {
    pub fn execute(&self, phrase: &mut Phrase) -> Option<PhraseEditCommand> {
        // make sure we are working in the phrase
        if self.column >= Phrase::FX_COLUMNS as usize {
            return None;
        }
        if self.row > phrase.subdivisions() {
            return None;
        }

        // execute
        let old_effect_opt = phrase.set_effect(self.row, self.column, Some(self.effect));
        if let Some(old_effect) = old_effect_opt {
            if old_effect == self.effect {
                None
            } else {
                Some(Self::new(old_effect, self.row, self.column).into())
            }
        } else {
            Some(PhraseClearEffects::new_cell(self.row, self.column).into())
        }
    }
}

/// a phrase clear command clears effects in a range
#[derive(Debug, Clone)]
pub struct PhraseClearEffects {
    pub rows: (u32, u32),
    pub columns: (usize, usize),
}

impl PhraseClearEffects {
    /// creates a phrase clear effect for a single cell
    pub fn new_cell(row: u32, column: usize) -> Self {
        Self {
            rows: (row, row),
            columns: (column, column),
        }
    }

    /// creates a phrase clear effect for a rectangle
    pub fn new(rows: (u32, u32), columns: (usize, usize)) -> Self {
        Self {
            rows,
            columns
        }
    }
}

impl Display for PhraseClearEffects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rows.0 == self.rows.1 && self.columns.0 == self.columns.1 {
            write!(
                f,
                "Cleared r{}:c{}",
                self.rows.0,
                self.columns.0
            )
        } else {
            write!(
                f,
                "Cleared r{}-{}:c{}-{}",
                self.rows.0,
                self.rows.1,
                self.columns.0,
                self.columns.1
            )
        }
    }
}

impl PhraseClearEffects {
    pub fn execute(&self, phrase: &mut Phrase) -> Option<PhraseRestoreEffects> {
        let mut is_empty = true;
        let mut buffer = Vec::new();
        for r in self.rows.0..=self.rows.1 {
            for c in self.columns.0..=self.columns.1.min(Phrase::FX_COLUMNS - 1) {
                let old_effect = phrase.set_effect(r, c, None);
                if old_effect.is_some() {
                    is_empty = false;
                }
                buffer.push((r, c, old_effect));
            }
        }
        if is_empty {
            None
        } else {
            Some(PhraseRestoreEffects {
                cells: buffer,
            })
        }
    }
}

/// a phrase command to restore cleared effects, only creatable through executing PhraseClearEffects
#[derive(Debug, Clone)]
pub struct PhraseRestoreEffects {
    pub cells: Vec<(u32, usize, Option<PhraseEffect>)>,
}

impl Display for PhraseRestoreEffects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Restored {} effect(s)", self.cells.len())
    }
}

impl PhraseRestoreEffects {
    pub fn execute(&self, phrase: &mut Phrase) -> Option<PhraseRestoreEffects> {
        let mut redo = Vec::new();
        let mut is_empty = true;
        for &(r, c, effect_opt) in &self.cells {
            let prev = phrase.set_effect(r, c, effect_opt);
            redo.push((r, c, prev));
            if prev.is_some() {
                is_empty = false;
            }
        }
        if is_empty {
            None
        } else {
            Some(PhraseRestoreEffects { cells: redo })
        }
    }
}

/// a phrase edit command that sets a single effect
#[derive(Debug, Clone)]
pub struct PhraseSetNote {
    pub note: Note,
    pub row: u32,
    pub column: usize,
}

impl PhraseSetNote {
    pub fn new(note: Note, row: u32, column: usize) -> Self {
        Self {
            note,
            row,
            column
        }
    }
}

impl Display for PhraseSetNote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Set r{}:c{} to {}", self.row, self.column, self.note.to_string_sharps())
    }
}

impl PhraseSetNote {
    pub fn execute(&self, phrase: &mut Phrase) -> Option<PhraseEditCommand> {
        let column = self.column.checked_sub(Phrase::FX_COLUMNS)?;

        // make sure we are working in the phrase
        if column >= Phrase::VOICE_COLUMNS as usize {
            return None;
        }
        if self.row > phrase.subdivisions() {
            return None;
        }

        // execute
        let old_effect_opt = phrase.set_note(self.row, column, Some(self.note));
        if let Some(old_effect) = old_effect_opt {
            if old_effect == self.note {
                None
            } else {
                Some(Self::new(old_effect, self.row, self.column).into())
            }
        } else {
            Some(PhraseClearNotes::new_cell(self.row, self.column).into())
        }
    }
}

/// a phrase clear command clears effects in a range
#[derive(Debug, Clone)]
pub struct PhraseClearNotes {
    pub rows: (u32, u32),
    pub columns: (usize, usize),
}

impl PhraseClearNotes {
    /// creates a phrase clear effect for a single cell
    pub fn new_cell(row: u32, column: usize) -> Self {
        Self {
            rows: (row, row),
            columns: (column, column),
        }
    }

    /// creates a phrase clear effect for a rectangle
    pub fn new(rows: (u32, u32), columns: (usize, usize)) -> Self {
        Self {
            rows,
            columns
        }
    }
}

impl Display for PhraseClearNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rows.0 == self.rows.1 && self.columns.0 == self.columns.1 {
            write!(
                f,
                "Cleared r{}:c{}",
                self.rows.0,
                self.columns.0
            )
        } else {
            write!(
                f,
                "Cleared r{}-{}:c{}-{}",
                self.rows.0,
                self.rows.1,
                self.columns.0,
                self.columns.1
            )
        }
    }
}

impl PhraseClearNotes {
    pub fn execute(&self, phrase: &mut Phrase) -> Option<PhraseRestoreNotes> {
        let mut is_empty = true;
        let mut buffer = Vec::new();
        for r in self.rows.0..=self.rows.1 {
            if self.columns.0 < Phrase::FX_COLUMNS {
                break;
            }

            let range = self.columns.0
            ..=self.columns.1.min(Phrase::FX_COLUMNS + Phrase::VOICE_COLUMNS - 1);

            for c in range {
                let old_note = phrase.set_note(r, c - Phrase::FX_COLUMNS, None);
                if old_note.is_some() {
                    is_empty = false;
                }
                buffer.push((r, c, old_note));
            }
        }
        if is_empty {
            None
        } else {
            Some(PhraseRestoreNotes {
                cells: buffer,
            })
        }
    }
}

/// a phrase command to restore cleared notes, only creatable through executing PhraseRestoreNotes
#[derive(Debug, Clone)]
pub struct PhraseRestoreNotes {
    pub cells: Vec<(u32, usize, Option<Note>)>,
}

impl Display for PhraseRestoreNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Restored {} notes(s)", self.cells.len())
    }
}

impl PhraseRestoreNotes {
    pub fn execute(&self, phrase: &mut Phrase) -> Option<PhraseRestoreNotes> {
        let mut redo = Vec::new();
        let mut is_empty = true;
        for &(r, c, note_opt) in &self.cells {
            if c < Phrase::FX_COLUMNS {
                break;
            }

            let prev = phrase.set_note(r, c - Phrase::FX_COLUMNS, note_opt);
            redo.push((r, c, prev));
            if prev.is_some() {
                is_empty = false;
            }
        }
        if is_empty {
            None
        } else {
            Some(PhraseRestoreNotes { cells: redo })
        }
    }
}

