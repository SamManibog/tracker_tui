use std::collections::HashMap;

use ratatui::{style::Color, layout::{Position, Rect}, style::Style, text::Text, widgets::Widget};

use crate::{Synthesizer, arrangement::InstrumentId};

/// the page that provides an overview of all created instruments
pub struct InstrumentOverview {
    /// the position of the focused cell
    cell_pos: Position,

    /// the position of the camera
    cam_pos: Position,

    /// the number control currently being input
    number_control: u32,
}

impl Default for InstrumentOverview {
    fn default() -> Self {
        Self {
            cell_pos: Position::ORIGIN,
            cam_pos: Position::ORIGIN,
            number_control: 0,
        }
    }
}

struct OverviewWidget<'a> {
    cell_pos: &'a mut Position,
    cam_pos: &'a mut Position,
    cell_rect: &'a mut Option<Rect>,
    instruments: &'a HashMap<InstrumentId, Box<dyn Synthesizer>>,
}

impl<'a> OverviewWidget<'a> {
    pub const LINE_NUMBER_COLOR: Color = Color::Yellow;
	pub const EMPTY_COLOR: Color = Color::DarkGray;
    pub const FILLED_COLOR: Color = Color::White;
    pub const H_GAP: u16 = 1;
    pub const V_GAP: u16 = 0;
    pub const ROWS: u16 = 16;
    pub const COLUMNS: u16 = 16;
}

impl Widget for OverviewWidget<'_> {
    fn render(self,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer
    ) where Self: Sized {
        *self.cell_rect = None;

        // clamp cell position
        self.cell_pos.x = self.cell_pos.x.min(Self::COLUMNS - 1);
        self.cell_pos.y = self.cell_pos.y.min(Self::ROWS - 1);

        if area.height <= 0 {
            return;
        }
        if area.width < 2 {
            return;
        }

        for r in 0..Self::ROWS {
            let num_rect = Rect {
                x: area.x,
                y: area.y + (r + 1) * (1 + Self::V_GAP),
                width: 2,
                height: 1,
            };

            if area.intersection(num_rect) == num_rect {
                let number_text = Text::from(format!("{:X}X", r))
                    .style(Style::new().fg(Self::LINE_NUMBER_COLOR));
                number_text.render(num_rect, buf);
            } else {
                break;
            }
        }

        for c in 0..Self::COLUMNS {
            let num_rect = Rect {
                x: area.x + (c + 1) * (2 + Self::H_GAP),
                y: area.y,
                width: 2,
                height: 1,
            };

            if area.intersection(num_rect) == num_rect {
                let number_text = Text::from(format!("X{:X}", c))
                    .style(Style::new().fg(Self::LINE_NUMBER_COLOR));
                number_text.render(num_rect, buf);
            } else {
                break;
            }
        }

        'row_iter: for r in 0..Self::ROWS {
            let y = area.y + (r + 1) * (1 + Self::V_GAP);

            if area.height <= y {
                break 'row_iter;
            }

            'col_iter: for c in 0..Self::COLUMNS {
                let num_rect = Rect {
                    x: area.x + (c + 1) * (2 + Self::H_GAP),
                    y,
                    width: 2,
                    height: 1,
                };

                if area.intersection(num_rect) == num_rect {
                    let key = InstrumentId((r * 16 + c) as u8);
                    let num_text = if self.instruments.contains_key(&key) {
                        Text::from(format!("{:X}{:X}", r, c))
                            .style(Style::new().fg(Self::FILLED_COLOR))
                    } else {
                        Text::from(format!("--"))
                            .style(Style::new().fg(Self::EMPTY_COLOR))
                    };
                    num_text.render(num_rect, buf);
                } else {
                    break 'col_iter;
                }
            }
        }

    }
}

impl InstrumentOverview {
    pub fn new() -> Self {
        Self::default()
    }

    /// render the editor, returning the text element used for the statusline
    pub fn render(
        &mut self,
        instruments: &HashMap<InstrumentId, Box<dyn Synthesizer>>,
        focused: bool,
        area: Rect,
        buf: &mut ratatui::prelude::Buffer
    ) -> Text<'_> {
        // render the phrase
        let mut cell_rect = None;
        OverviewWidget {
            cam_pos: &mut self.cam_pos,
            cell_pos: &mut self.cell_pos,
            cell_rect: &mut cell_rect,
            instruments,
        }.render(area, buf);

        if let Some(cell_rect) = cell_rect {
            // if focused, highlight the cell rect
            if focused {
                buf.set_style(cell_rect, Style::new().black().on_white());
            }

        }

        Text::from("test")
    }
}

