use ratatui::{crossterm::event::{KeyCode, KeyEvent, KeyModifiers}, layout::{Position, Rect}, prelude::Color, style::Style, text::Text, widgets::{Clear, Widget}};

use crate::{Phrase, PhraseEffect, phrase_edit_command::{PhraseClearEffects, PhraseEditCommand, PhraseSetEffect}, utils::PageCommand};

#[derive(Debug, Clone)]
pub enum PhraseEditorCommand {
    Edit(PhraseEditCommand),
    PlayPhrase,
}

/// the mode of the phrase editor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhraseEditorMode {
    Normal,
    Insert,
}

/// a temporary Ratatui widget used to display a phrase
pub struct PhraseWidget<'a> {
    /// the phrase to draw
    pub phrase: &'a Phrase,

    /// the position of the camera
    /// when rendering, this position is automatically updated so the widget contains the cell
    pub cam_pos: &'a mut Position,

    /// the position of the selected cell
    /// when rendering, this is updated to be within the allowed bounds
    pub cell_pos: &'a mut Position,

    /// the rectangle containing the selected cell
    /// setting this field beforehand does nothing, but this
    /// can be used for a second rendering pass like to highlight the cell
    pub cell_rect: &'a mut Option<Rect>,
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
        *self.cell_rect = None;

        // clamp cell position
        if self.cell_pos.x >= Phrase::FX_COLUMNS as u16{
            if self.cell_pos.x as usize - Phrase::FX_COLUMNS >= self.phrase.voice_count() {
                self.cell_pos.x = (Phrase::FX_COLUMNS + self.phrase.voice_count()) as u16 - 1;
            }
            if self.cell_pos.y >= self.phrase.subdivisions() as u16 {
                self.cell_pos.y = self.phrase.subdivisions() as u16 - 1;
            }
        } else if self.cell_pos.y > self.phrase.subdivisions() as u16 {
            self.cell_pos.y = self.phrase.subdivisions() as u16;
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
            .min(self.phrase.subdivisions() as u16);
        let end_note_line_number = (start_line_number + area.height - 1)
            .min(self.phrase.subdivisions() as u16 - 1);

        // the number of characters needed for the line number
        let fx_line_number_digits = (self.phrase.subdivisions()).ilog10() as u16 + 1;
        let note_line_number_digits = (self.phrase.subdivisions() - 1).ilog10() as u16 + 1;

        // clamp camera x position to the correct bounds (must have 1+ visible column)
        {
            let cell_min_x = if self.cell_pos.x < Phrase::FX_COLUMNS as u16 {
                self.cell_pos.x * (Self::FX_WIDTH + 1)
            } else {
                // midle section of line numbers
                fx_line_number_digits + 1

                // fx
                + Phrase::FX_COLUMNS as u16 * (Self::FX_WIDTH + 1)

                // voices + spacing
                + (self.cell_pos.x - Phrase::FX_COLUMNS as u16) * (Self::NOTE_WIDTH + 1)
            };

            let cell_max_x = cell_min_x + if self.cell_pos.x < Phrase::FX_COLUMNS as u16 {
                Self::FX_WIDTH
            } else {
                Self::NOTE_WIDTH
            } + fx_line_number_digits + 1

            // so we can show line numbers on right side
            + note_line_number_digits + 1;

            self.cam_pos.x = self.cam_pos.x.clamp(
                cell_max_x.saturating_sub(area.width),
                cell_min_x
            );
        }
        let x_offset = self.cam_pos.x;

        // x_cursor must be greater than this in order to render fully
        let x_threshold = x_offset + fx_line_number_digits + 1;

        // stop drawing if x_cursor is greater than this
        let x_cutoff = area.x + area.width + x_offset;

        // render begining line numbers (camera-stable, no need threshold checking)
        if area.width < fx_line_number_digits {
            return;
        }
        for i in start_line_number..=end_fx_line_number {
            let number_text = Text::from(format!("{}", i))
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
        if cursor_x + Self::FX_WIDTH > x_cutoff {
            return;
        }
        // iteration over effect_lists before columns has O(nlogn) time complexity
        // iteration over columns then effect lists has O(mnlogn) time complexity
        for i in start_line_number..=end_fx_line_number {
            let mut fx_cursor_x = cursor_x;
            let effect_list = self.phrase.effects().get_effect_list(i as u32).unwrap_or(empty_fx_list);

            'draw_fx: for (column, effect_opt) in effect_list.iter().enumerate() {
                if (fx_cursor_x + Self::FX_WIDTH) > x_cutoff {
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
                    let rect = Rect::new(
                        fx_cursor_x - x_offset,
                        i - start_line_number,
                        Self::FX_WIDTH,
                        1);
                    if i == self.cell_pos.y && column as u16 == self.cell_pos.x {
                        *self.cell_rect = Some(rect);
                    }
                    text.render(rect, buf);
                }

                fx_cursor_x += Self::FX_WIDTH + 1;
            }
        }

        // second line number
        cursor_x += (Self::FX_WIDTH + 1) * Phrase::FX_COLUMNS as u16;
        if cursor_x + fx_line_number_digits > x_cutoff {
            return;
        }
        if cursor_x >= x_threshold {
            for i in start_line_number..=end_fx_line_number {
                let number_text = Text::from(format!("{}", i))
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
        for (column, voice) in self.phrase.voices().iter().enumerate() {
            if cursor_x + Self::NOTE_WIDTH > x_cutoff {
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
                    let rect = Rect::new(
                        cursor_x - x_offset,
                        i - start_line_number,
                        Self::NOTE_WIDTH,
                        1);
                    if i == self.cell_pos.y && column as u16 + 8 == self.cell_pos.x {
                        *self.cell_rect = Some(rect);
                    }
                    text.render(rect, buf);
                }
            }

            cursor_x += Self::NOTE_WIDTH + 1;
        }

        if cursor_x + note_line_number_digits > x_cutoff {
            return;
        }

        // render line numbers again
        // we do not need to check for x_threshold bounds because of camera positioning rules
        for i in start_line_number..=end_note_line_number {
            let number_text = Text::from(format!("{}", i))
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
pub struct PhraseEditor {
    /// the mode of the editor
    mode: PhraseEditorMode,

    /// the position of the focused cell
    cell_pos: Position,

    /// the position of the camera
    cam_pos: Position,

    /// the text the modifying in the current cell
    text: String,

    /// the number control currently being input
    number_control: u32,
}

impl PhraseEditor {
    pub fn new() -> Self {
        Self {
            mode: PhraseEditorMode::Normal,
            number_control: 0,
            cell_pos: Position::MIN,
            cam_pos: Position::MIN,
            text: String::new(),
        }
    }

    /// gets the effect column the phrase editor is on
    pub fn effect_column(&self) -> Option<usize> {
        if self.cell_pos.x < 8 {
            Some(self.cell_pos.x as usize)
        } else {
            None
        }
    }

    /// gets the column of the voice the phrase editor is on
    pub fn voice_column(&self) -> Option<usize> {
        (self.cell_pos.x as usize).checked_sub(8)
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
            self.cell_pos.y.saturating_sub(-y_offset as u16)
        } else {
                self.cell_pos.y.saturating_add(y_offset as u16)
            };

        // clamp cell to bounds of the phrase
        self.set_cursor_y(phrase, y as u32);
    }

    fn clamp_cursor_y(&mut self, phrase: &Phrase, y: u32) -> u16 {
        if self.cell_pos.x < Phrase::FX_COLUMNS as u16 {
            y.min(phrase.subdivisions()) as u16
        } else {
            y.min(phrase.subdivisions().saturating_sub(1)) as u16
        }
    }

    pub fn set_cursor_y(&mut self, phrase: &Phrase, row: u32) {
        self.cell_pos.y = self.clamp_cursor_y(phrase, row);
    }

    /// attempts to move the cursor by the given offset in the x direction
    fn move_cursor_x(&mut self, phrase: &Phrase, offset: i32) {
        const OFFSET_MAX: i32 = (Phrase::FX_COLUMNS * Phrase::VOICE_COLUMNS) as i32;
        // determine real offset
        let x_offset = offset
            .saturating_mul(self.number_control.max(1) as i32)
            .clamp(-OFFSET_MAX, OFFSET_MAX);

        // handle movement
        let x = if x_offset.is_negative() {
            self.cell_pos.x.saturating_sub(-x_offset as u16)
        } else {
                self.cell_pos.x.saturating_add(x_offset as u16)
            };

        self.set_cursor_x(phrase, x as u32);
    }

    fn clamp_cursor_x(&mut self, phrase: &Phrase, x: u32) -> u16 {
        if self.cell_pos.y == phrase.subdivisions() as u16 {
            x.min(Phrase::FX_COLUMNS as u32 - 1) as u16
        } else {
            x.min((Phrase::VOICE_COLUMNS + Phrase::FX_COLUMNS) as u32 - 1) as u16
        }
    }

    /// attempts to set the cursor to the given x value
    pub fn set_cursor_x(&mut self, phrase: &Phrase, column: u32) {
        self.cell_pos.x = self.clamp_cursor_x(phrase, column);
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
                    'h' => self.move_cursor_x(&phrase, -1),
                    'l' => self.move_cursor_x(&phrase, 1),
                    'k' => self.move_cursor_y(&phrase, -1),
                    'j' => self.move_cursor_y(&phrase, 1),

                    // absolute movement
                    'c' => self.set_cursor_x(&phrase, self.number_control),
                    'r' => self.set_cursor_y(&phrase, self.number_control),

                    // change modes
                    'i' => self.mode = PhraseEditorMode::Insert,

                    // edit commands
                    'x' => {
                        output = PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseClearEffects::new_cell(
                                self.cell_pos.y.into(),
                                self.cell_pos.x.into()
                            ).into())
                        );
                    },

                    'q' => { output = PageCommand::Quit; },

                    _ => {}
                }
            }

            C::Left => self.move_cursor_x(&phrase, -1),
            C::Right => self.move_cursor_x(&phrase, 1),
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
        if event.modifiers == KeyModifiers::empty() {
            match event.code {
                KeyCode::Char(char) => {
                    if char.is_ascii_graphic() {
                        self.text.push(char);
                    }
                },

                KeyCode::Enter => {
                    self.mode = PhraseEditorMode::Normal;
                    if self.text.is_empty() {
                        return PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseClearEffects::new_cell(
                                self.cell_pos.y.into(),
                                self.cell_pos.x.into()
                            ).into())
                        );
                    } else if let Ok(fx) = str::parse::<PhraseEffect>(&self.text) {
                        return PageCommand::Command(
                            PhraseEditorCommand::Edit(PhraseSetEffect::new(
                                fx,
                                self.cell_pos.y.into(),
                                self.cell_pos.x.into()
                            ).into())
                        );
                    }
                },

                KeyCode::Backspace => {
                    self.text.pop();
                },

                _ => {}
            }
        }

        // handle exit to normal mode
        else if event.modifiers == KeyModifiers::CONTROL && let KeyCode::Char('c') = event.code {
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
        match self.mode {
            PhraseEditorMode::Normal => self.normal_handle_key_event(phrase, event),
            PhraseEditorMode::Insert => self.insert_handle_key_event(event),
        }
    }

    /// render the editor, returning the text element used for the statusline
    pub fn render(
        &mut self,
        phrase: &Phrase,
        focused: bool,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer
    ) -> Text {
        // render the phrase
        let mut cell_rect = None;
        PhraseWidget {
            cam_pos: &mut self.cam_pos,
            cell_pos: &mut self.cell_pos,
            cell_rect: &mut cell_rect,
            phrase: phrase,
        }.render(area, buf);

        if self.mode != PhraseEditorMode::Insert {
            self.text.clear();
        }

        if let Some(cell_rect) = cell_rect {
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

        }

        match self.mode {
            PhraseEditorMode::Normal => Text::from("Normal"),
            PhraseEditorMode::Insert => Text::from("Insert"),
        }
    }

}

