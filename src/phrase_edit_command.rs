use std::fmt::{Debug, Display};

use crate::{Phrase, PhraseEffect};

/// An enum for commands emitted by a phrase editor
#[derive(Debug, Clone)]
pub enum PhraseEditCommand {
    SetEffect(PhraseSetEffect),
    ClearEffects(PhraseClearEffects),
    RestoreEffects(PhraseRestoreEffects),
}

impl std::fmt::Display for PhraseEditCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhraseEditCommand::SetEffect(cmd) => std::fmt::Display::fmt(cmd, f),
            PhraseEditCommand::ClearEffects(cmd) => std::fmt::Display::fmt(cmd, f),
            PhraseEditCommand::RestoreEffects(cmd) => std::fmt::Display::fmt(cmd, f),
        }
    }
}

impl PhraseEditCommand {
    /// execute the command, and return a command to undo the execution if a change was made
    pub fn execute(&self, phrase: &mut Phrase) -> Option<Self> {
        match self {
            PhraseEditCommand::SetEffect(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            PhraseEditCommand::ClearEffects(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
            PhraseEditCommand::RestoreEffects(cmd) => cmd.execute(phrase).map(PhraseEditCommand::from),
        }
    }
}

impl From<PhraseSetEffect> for PhraseEditCommand {
    fn from(cmd: PhraseSetEffect) -> Self {
        PhraseEditCommand::SetEffect(cmd)
    }
}
impl From<PhraseClearEffects> for PhraseEditCommand {
    fn from(cmd: PhraseClearEffects) -> Self {
        PhraseEditCommand::ClearEffects(cmd)
    }
}
impl From<PhraseRestoreEffects> for PhraseEditCommand {
    fn from(cmd: PhraseRestoreEffects) -> Self {
        PhraseEditCommand::RestoreEffects(cmd)
    }
}

// Removed: impl Clone for Box<dyn PhraseEditCommand>

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
