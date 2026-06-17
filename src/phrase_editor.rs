use ratatui::{crossterm::{event::{KeyCode, KeyEvent, KeyModifiers}, style}, layout::{Position, Rect}, prelude::Color, style::Style, text::{Line, Text}, widgets::{Clear, Widget}};

use crate::{Note, Phrase, PhraseEffect, phrase_edit_command::{PhraseClearEffects, PhraseClearNotes, PhraseEditCommand, PhraseSetEffect, PhraseSetNote}, shift_grid::{ShiftGrid, ShiftGridState}, utils::PageCommand};

#[derive(Debug, Clone)]
pub enum PhraseEditorCommand {
    Edit(PhraseEditCommand),
    PlayPhrase,
    StopPhrase,
    TogglePhrase,
}

/// the mode of the phrase editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseEditorMode {
    Normal,
    Insert,
}

#[derive(Debug, Clone)]
pub struct PhraseEditor {
    /// the mode of the editor
    mode: PhraseEditorMode,

    /// the state of the shift grid
    state: ShiftGridState,

    /// the text the modifying in the current cell
    text: String,

    /// the number control currently being input
    number_control: u32,
}

impl PhraseEditor {
    pub const VERTICAL_PADDING: usize = 0;
    pub const HORIZONTAL_PADDING: usize = 0;

    pub const NOTE_WIDTH: u16 = 3;
    pub const FX_WIDTH: u16 = 6;

    pub const COL_COUNT: u32 = (Phrase::FX_COLUMNS + Phrase::VOICE_COLUMNS) as u32;

    pub const LINE_NUMBER_COLOR: Color = Color::Yellow;
	pub const EMPTY_COLOR: Color = Color::DarkGray;
    pub const FILLED_COLOR: Color = Color::White;

    pub fn new() -> Self {
        Self {
            mode: PhraseEditorMode::Normal,
            number_control: 0,
            state: ShiftGridState::default(),
            text: String::new(),
        }
    }

    fn col_widths(col: u32) -> u16 {
        if col < Phrase::FX_COLUMNS as u32 {
            Self::FX_WIDTH
        } else {
            Self::NOTE_WIDTH
        }.into()
    }

    fn col_numbers<'b>(col: u32) -> Line<'b> {
        Line::from(format!("{:X}", col))
            .style(Style::new().fg(Self::LINE_NUMBER_COLOR))
            .centered()
    }

    fn row_numbers<'b>(row: u32) -> Line<'b> {
        Line::from(format!("{:X}", row))
            .style(Style::new().fg(Self::LINE_NUMBER_COLOR))
            .right_aligned()
    }

    fn row_number_width(rows: u32) -> u16 {
        rows.ilog10() as u16 + 1
    }

    /// handles a digit being added to the number control input
    fn add_control_digit(&mut self, digit: u32) {
        debug_assert!(digit < 10);
        self.number_control = self.number_control.saturating_mul(10).saturating_add(digit);
    }

    /// attempts to move the cursor by the given offset in the y direction
    fn move_cursor_y(&mut self, phrase: &Phrase, offset: i32) {
        const OFFSET_MAX: i32 = (Phrase::MAX_SUBDIVISION_MULTIPLIER * Phrase::MAX_DURATION) as i32;
        // determine real offset
        let y_offset = offset
            .saturating_mul(self.number_control.max(1) as i32)
            .clamp(-OFFSET_MAX, OFFSET_MAX);

        // handle movement
        let y = if y_offset.is_negative() {
            self.state.centered_row.saturating_sub(-y_offset as u32)
        } else {
                self.state.centered_row.saturating_add(y_offset as u32)
            };

        // clamp cell to bounds of the phrase
        self.set_cursor_y(phrase, y as u32);
    }

    fn clamp_cursor_y(&mut self, phrase: &Phrase, y: u32) -> u32 {
        y.min(phrase.subdivisions()) as u32
    }

    pub fn set_cursor_y(&mut self, phrase: &Phrase, row: u32) {
        self.state.centered_row = self.clamp_cursor_y(phrase, row);
    }

    /// attempts to move the cursor by the given offset in the x direction
    fn move_cursor_x(&mut self, offset: i32) {
        const OFFSET_MAX: i32 = (Phrase::FX_COLUMNS * Phrase::VOICE_COLUMNS) as i32;
        // determine real offset
        let x_offset = offset
            .saturating_mul(self.number_control.max(1) as i32)
            .clamp(-OFFSET_MAX, OFFSET_MAX);

        // handle movement
        let x = if x_offset.is_negative() {
            self.state.centered_col.saturating_sub(-x_offset as u32)
        } else {
                self.state.centered_col.saturating_add(x_offset as u32)
            };

        self.set_cursor_x(x as u32);
    }

    fn clamp_cursor_x(&mut self, x: u32) -> u32 {
        x.min((Phrase::VOICE_COLUMNS + Phrase::FX_COLUMNS) as u32 - 1)
    }

    /// attempts to set the cursor to the given x value
    pub fn set_cursor_x(&mut self, column: u32) {
        self.state.centered_col = self.clamp_cursor_x(column);
    }

    /// handle an event in normal mode
    fn normal_handle_key_event(
        &mut self,
        phrase: &Phrase,
        event: KeyEvent
    ) -> PageCommand<PhraseEditorCommand> {
        type C = KeyCode;

        let mut output: PageCommand<PhraseEditorCommand> = PageCommand::Nop;

        // handle number control
        if let C::Char(character) = event.code && let Some(digit) = character.to_digit(10) {
            self.add_control_digit(digit);

            // must return before resetting number control
            return PageCommand::Nop;
        }

        // handle everything else
        match event.code {
            C::Char(character) => {
                // handle everything else
                match character {
                    // relative movement
                    'h' => self.move_cursor_x(-1),
                    'l' => self.move_cursor_x(1),
                    'k' => self.move_cursor_y(&phrase, -1),
                    'j' => self.move_cursor_y(&phrase, 1),

                    // absolute movement
                    'c' => self.set_cursor_x(self.number_control),
                    'r' => self.set_cursor_y(&phrase, self.number_control),

                    // switch between note and fx column
                    'w' => self.set_cursor_x(
                        (self.state.centered_col as u32 + Phrase::FX_COLUMNS as u32)
                        % (Phrase::FX_COLUMNS + Phrase::VOICE_COLUMNS) as u32
                    ),

                    // change modes
                    'i' => self.mode = PhraseEditorMode::Insert,

                    // play
                    ' ' => output = PageCommand::Command(PhraseEditorCommand::TogglePhrase),

                    // edit commands
                    'x' => output = PageCommand::Command(
                        if (self.state.centered_col as usize) < Phrase::FX_COLUMNS {
                            PhraseEditorCommand::Edit(PhraseClearEffects::new_cell(
                                self.state.centered_row,
                                self.state.centered_col as usize
                            ).into())
                        } else {
                            PhraseEditorCommand::Edit(PhraseClearNotes::new_cell(
                                self.state.centered_row,
                                self.state.centered_col as usize
                            ).into())
                        }
                    ),

                    'q' => { output = PageCommand::Quit; },

                    _ => {}
                }
            }

            C::Left => self.move_cursor_x(-1),
            C::Right => self.move_cursor_x(1),
            C::Up => self.move_cursor_y(&phrase, -1),
            C::Down => self.move_cursor_y(&phrase, 1),

            _ => {},
        }
        self.number_control = 0;

        output
    }

    /// handle an event in insert mode
    fn insert_handle_key_event(
        &mut self,
        event: KeyEvent
    ) -> PageCommand<PhraseEditorCommand> {
        // handle regular typing
        match (event.code, event.modifiers.is_empty()) {
            (KeyCode::Char(char), _) => {
                if char.is_ascii_graphic() {
                    self.text.push(char);
                }
            },

            (KeyCode::Enter, true) => {
                self.mode = PhraseEditorMode::Normal;
                if self.state.centered_col < Phrase::FX_COLUMNS as u32 {
                    if self.text.is_empty() {
                        return PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseClearEffects::new_cell(
                                self.state.centered_row,
                                self.state.centered_col as usize
                            ).into())
                        );
                    } else if let Ok(fx) = str::parse::<PhraseEffect>(&self.text) {
                        return PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseSetEffect::new(
                                fx,
                                self.state.centered_row, 
                                self.state.centered_col as usize
                            ).into())
                        );
                    }
                } else {
                    if self.text.is_empty() {
                        return PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseClearNotes::new_cell(
                                self.state.centered_row,
                                self.state.centered_col as usize
                            ).into())
                        );
                    } else if let Some(note) = Note::parse_three_character(&self.text) {
                        return PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseSetNote::new(
                                note,
                                self.state.centered_row,
                                self.state.centered_col as usize
                            ).into())
                        );
                    }

                }
            },

            (KeyCode::Backspace, _) => {
                self.text.pop();
            },

            _ => {}
        }

        // handle exit to normal mode
        if event.modifiers == KeyModifiers::CONTROL && let KeyCode::Char('c') = event.code {
            self.mode = PhraseEditorMode::Normal;
        }


        PageCommand::Nop
    }

    pub fn handle_key_event(
        &mut self,
        phrase: &Phrase,
        event: KeyEvent,
    ) -> PageCommand<PhraseEditorCommand> {
        // note: upper level call for this event is already known to be a press
        let out = match self.mode {
            PhraseEditorMode::Normal => self.normal_handle_key_event(phrase, event),
            PhraseEditorMode::Insert => self.insert_handle_key_event(event),
        };

        if self.mode != PhraseEditorMode::Insert {
            self.text.clear();
        }

        out
    }

    /// render the editor, returning the text element used for the statusline
    pub fn render(
        &mut self,
        phrase: &Phrase,
        focused: bool,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer
    ) -> Text<'_> {
        // render the grid
        let cells = |row: u32, col: u32| -> Line<'_> {
            if col < Phrase::FX_COLUMNS as u32 {
                let effect_list_opt = phrase.effects()
                    .get_effect_list(row);

                let effect_opt = if let Some(effect_list) = effect_list_opt {
                    effect_list[col as usize]
                } else {
                    None
                };

                if let Some(effect) = effect_opt {
                    Line::from(format!("{}      ", effect.abbreviate()))
                        .style(Style::new().fg(Self::FILLED_COLOR))
                } else {
                    Line::from("------")
                        .style(Style::new().fg(Self::EMPTY_COLOR))
                }
            } else {
                let voice = &phrase.voices()[col as usize - Phrase::FX_COLUMNS];

                if let Some(note) = voice.get_note(row as u32) {
                    Line::from(note.to_padded_string_sharps())
                        .style(Style::new().fg(Self::FILLED_COLOR))
                } else {
                    Line::from("---")
                        .style(Style::new().fg(Self::EMPTY_COLOR))
                }
            }
        };

        let row_count = phrase.subdivisions() + 1;

        ShiftGrid::new(&mut self.state, row_count, Self::COL_COUNT)
            .col_numbers(&Self::col_numbers)
            .col_widths(&Self::col_widths)
            .row_numbers(&Self::row_numbers)
            .row_number_width(Self::row_number_width(row_count))
            .cells(&cells)
            .render(area, buf);

        // render special modifiers for the centered area
        let cell_rect = self.state.centered_area;

        // write text if in insert mode
        if self.mode == PhraseEditorMode::Insert {
            // truncate if necessary
            if let Some((char_index, _)) = self.text.char_indices().skip(cell_rect.width as usize).next() {
                self.text.truncate(char_index);
            }

            Clear.render(cell_rect, buf);

            // write
            Text::from(self.text.clone()).render(cell_rect, buf);
        }

        // if focused, highlight the cell rect
        if focused {
            buf.set_style(cell_rect, Style::new().black().on_white());
        }

        match self.mode {
            PhraseEditorMode::Normal => Text::from("Normal"),
            PhraseEditorMode::Insert => Text::from("Insert"),
        }
    }

}

