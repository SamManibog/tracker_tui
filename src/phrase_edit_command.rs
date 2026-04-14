use crate::{Phrase, PhraseEffect};

/// a trait for a command emitted by a phrase editor
pub trait PhraseEditCommand {
    /// get a short summary of the command possibly displayed when command occurs
    fn summary(&self) -> String;
    
    /// execute the command, and return a command to undo the execution if a change was made
    fn execute(&self, phrase: &mut Phrase) -> Option<Box<dyn PhraseEditCommand>>;
}

/// a phrase edit command that sets a single effect
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

impl PhraseEditCommand for PhraseSetEffect {
    fn summary(&self) -> String {
        format!("Set r{}:c{} to {}", self.row, self.column, self.effect)
    }

    fn execute(&self, phrase: &mut Phrase) -> Option<Box<dyn PhraseEditCommand>> {
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
                Some(Box::new(Self::new(old_effect, self.row, self.column)))
            }
        } else {
            Some(Box::new(PhraseClearEffects::new_cell(self.row, self.column)))
        }
    }
}

/// a phrase clear command clears effects in a range
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

impl PhraseEditCommand for PhraseClearEffects {
    fn summary(&self) -> String {
        if self.rows.0 == self.rows.1 && self.columns.0 == self.columns.1 {
            format!(
                "Cleared r{}:c{}",
                self.rows.0,
                self.columns.0
            )
        } else {
            format!(
                "Cleared r{}-{}:c{}-{}",
                self.rows.0,
                self.rows.1,
                self.columns.0,
                self.columns.1
            )
        }
    }

    fn execute(&self, phrase: &mut Phrase) -> Option<Box<dyn PhraseEditCommand>> {
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
            Some(Box::new(PhraseRestoreEffects {
                cells: buffer,
            }))
        }
    }
}

/// a phrase command to restore cleared effects, only creatable through executing PhraseClearEffects
pub struct PhraseRestoreEffects {
    pub cells: Vec<(u32, usize, Option<PhraseEffect>)>,
}

impl PhraseEditCommand for PhraseRestoreEffects {
    fn summary(&self) -> String {
        format!("Restored {} effect(s)", self.cells.len())
    }

    fn execute(&self, phrase: &mut Phrase) -> Option<Box<dyn PhraseEditCommand>> {
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
            Some(Box::new(PhraseRestoreEffects { cells: redo }))
        }
    }
}
