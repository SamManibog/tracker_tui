use std::io;

use rand::{RngExt, distr::uniform::SampleRange};
use ratatui::{DefaultTerminal, Frame, crossterm::event::{self, Event, KeyCode, KeyEventKind}, layout::Rect, style::Style, text::Line, widgets::Widget};

/// a struct representing the stored state of a shfit grid
#[derive(Default)]
pub struct ShiftGridState {
    /// the row that is "selected", which must be on screen
    pub centered_row: u32,

    /// the column that is "selected", which must be on screen
    pub centered_col: u32,

    /// the area on the screen containing the centered cell
    pub centered_area: Rect,

    /// the x position of the camera
    pub cam_x: u32,

    /// the y position of the camera
    pub cam_y: u32,

    /// debug fields
    pub d_start_col: u32,
    pub d_end_col: u32,
    pub d_x_offset: u16,
    pub d_text: String,
}

/// a ratatui widget that can lay out small 1-height elements in a grid-like layout
/// the grid may be shifted by an offset determined by the state
/// on the grid, a single element is considered "selected" and will always be within view
pub struct ShiftGrid<'a> {
    /// the function to generate line elements for row numbers
    pub row_numbers: &'a dyn Fn(u32) -> Line<'a>,

    /// the max width of row numbers
    pub row_number_width: u16,

    /// the number of rows
    pub row_count: u32,

    /// the vertical gap between rows
    pub row_gap: u16,

    /// the horizontal gap between the row number and rows
    pub row_number_gap: u16,

    /// the function to generate column numbers
    pub col_numbers: &'a dyn Fn(u32) -> Line<'a>,

    /// the function that determines column widths
    pub col_widths: &'a dyn Fn(u32) -> u16,

    /// the number of columns
    pub col_count: u32,

    /// the horizontal gap between columns
    pub col_gap: u16,

    /// whether or not to show row numbers
    pub show_row_numbers: bool,

    /// whether or not to show column numbers
    pub show_col_numbers: bool,

    /// the vertical gap between the column number and columns
    pub col_number_gap: u16,

    /// the function to generate the content of a cell
    /// width should be consistent with col_widths
    pub cells: &'a dyn Fn(u32, u32) -> Line<'a>,

    /// the state
    pub state: &'a mut ShiftGridState,
}

impl<'a> ShiftGrid<'a> {
    pub const DEFAULT_ROW_COUNT: u32 = 0;
    pub const DEFAULT_COL_COUNT: u32 = 0;
    pub const DEFAULT_ROW_GAP: u16 = 0;
    pub const DEFAULT_COL_GAP: u16 = 1;

    pub const DEFAULT_ROW_NUMBER_GAP: u16 = 1;
    pub const DEFAULT_COL_NUMBER_GAP: u16 = 0;

    /// the number of blank cells between row numbers and content
    pub const ROW_NUMBER_SPACING: u16 = 1;

    /// the number of blank cells between column numbers and content
    pub const COL_NUMBER_SPACING: u16 = 0;

    pub fn default_row_numbers<'b>(index: u32) -> Line<'b> {
        Line::from(format!("{}", index)).right_aligned()
    }

    pub fn default_col_numbers<'b>(index: u32) -> Line<'b> {
        Line::from(format!("{}", index)).centered()
    }

    pub fn default_cells<'b>(_: u32, _: u32) -> Line<'b> {
        Line::from("")
    }

    pub fn base10_width(index: u32) -> u16 {
        if index == 0 {
            0
        } else {
            index.ilog10() as u16 + 1
        }
    }

    pub fn new(state: &'a mut ShiftGridState) -> Self {
        Self {
            row_numbers: &Self::default_row_numbers,
            row_number_width: 1,
            row_count: Self::DEFAULT_ROW_COUNT,
            row_gap: Self::DEFAULT_ROW_GAP,
            row_number_gap: Self::DEFAULT_ROW_NUMBER_GAP,
            show_row_numbers: true,

            col_numbers: &Self::default_col_numbers,
            col_widths: &Self::base10_width,
            col_count: Self::DEFAULT_COL_COUNT,
            col_gap: Self::DEFAULT_COL_GAP,
            col_number_gap: Self::DEFAULT_COL_NUMBER_GAP,
            show_col_numbers: true,

            cells: &Self::default_cells,
            state
        }
    }

    pub fn row_number_width(self, row_number_width: u16) -> Self {
        let mut out = self;
        out.row_number_width = row_number_width;
        out
    }

    pub fn row_numbers(self, row_numbers: &'a dyn Fn(u32) -> Line<'a>) -> Self {
        let mut out = self;
        out.row_numbers = row_numbers;
        out
    }

    pub fn row_count(self, row_count: u32) -> Self {
        let mut out = self;
        out.row_count = row_count;
        out
    }

    pub fn row_gap(self, row_gap: u16) -> Self {
        let mut out = self;
        out.row_gap = row_gap;
        out
    }

    pub fn row_number_gap(self, row_number_gap: u16) -> Self {
        let mut out = self;
        out.row_number_gap = row_number_gap;
        out
    }

    pub fn col_numbers(self, col_numbers: &'a dyn Fn(u32) -> Line<'a>) -> Self {
        let mut out = self;
        out.col_numbers = col_numbers;
        out
    }

    pub fn col_widths(self, col_widths: &'a dyn Fn(u32) -> u16) -> Self {
        let mut out = self;
        out.col_widths = col_widths;
        out
    }

    pub fn col_count(self, col_count: u32) -> Self {
        let mut out = self;
        out.col_count = col_count;
        out
    }

    pub fn col_gap(self, col_gap: u16) -> Self {
        let mut out = self;
        out.col_gap = col_gap;
        out
    }

    pub fn col_number_gap(self, col_number_gap: u16) -> Self {
        let mut out = self;
        out.col_number_gap = col_number_gap;
        out
    }

    pub fn hide_row_numbers(self) -> Self {
        let mut out = self;
        out.show_row_numbers = false;
        out
    }

    pub fn hide_col_numbers(self) -> Self {
        let mut out = self;
        out.show_col_numbers = false;
        out
    }

    pub fn hide_numbers(self) -> Self {
        let mut out = self;
        out.show_row_numbers = false;
        out.show_col_numbers = false;
        out
    }

    pub fn show_row_numbers(self) -> Self {
        let mut out = self;
        out.show_row_numbers = true;
        out
    }

    pub fn show_col_numbers(self) -> Self {
        let mut out = self;
        out.show_col_numbers = true;
        out
    }

    pub fn show_numbers(self) -> Self {
        let mut out = self;
        out.show_col_numbers = true;
        out.show_row_numbers = true;
        out
    }

    pub fn cells(self, cells: &'a dyn Fn(u32, u32) -> Line<'a>) -> Self {
        let mut out = self;
        out.cells = cells;
        out
    }

    /// gets the first and last rows to render and their offset
    /// assumes that the centered row is valid w.r.t. the row count
    fn get_row_start_end_offset(
        state: &'a mut ShiftGridState,
        height: u16,
        row_count: u32,
        row_gap: u16
    ) -> (u32, u32, u16) {
        let factor = 1 + row_gap as u32;
        let mut y_offset = (state.cam_y % factor) as u16;

        // essentially calulates start_row as ceiling(cam_y / factor)
        let correction = if y_offset == 0 {
            0
        } else {
            1
        };
        let mut start_row = state.cam_y / factor + correction;

        // ensure that centered row is after start row
        if state.centered_row < start_row {
            y_offset = 0;
            start_row = state.centered_row;
            state.cam_y = factor * start_row;
        }

        // determine the last row to render
        let mut end_row = {
            // essentially calulates ceiling(last_visual_row / factor)
            let end_row = (state.cam_y + height as u32 - 1) / factor;

            row_count.saturating_sub(1).min(end_row)
        };

        if state.centered_row > end_row {
            // calculate end row and camera position
            end_row = state.centered_row;
            state.cam_y = (factor * end_row + 1).saturating_sub(height as u32);

            // recalculate start row, reusing code above
            y_offset = (state.cam_y % factor) as u16;
            let correction = if y_offset == 0 {
                0
            } else {
                1
            };
            start_row = state.cam_y / factor + correction;
        }

        (start_row, end_row, y_offset)
    }

    /// gets the first and last column to render, and the offset from the first position
    /// assumes that the centered column is valid w.r.t. the column count
    fn get_col_start_end_offset(
        state: &'a mut ShiftGridState,
        width: u16,
        col_widths: &'a dyn Fn(u32) -> u16,
        col_count: u32,
        col_gap: u16,
    ) -> (u32, u32, u16) {
        // the x position of the next column to add
        let mut cursor: u32 = 0;

        // cached start positions
        let mut col_start_xs = Vec::with_capacity(col_count as usize);

        // fill the cache of column start positions up to the centered column
        for i in 0..=state.centered_col {
            col_start_xs.push(cursor);
            cursor += col_widths(i) as u32 + col_gap as u32;
        }

        let center_start_x = *col_start_xs.last().unwrap();
        let center_end_x = center_start_x + col_widths(col_start_xs.len() as u32 - 1).saturating_sub(1) as u32;

        if state.cam_x >= center_start_x {
            state.d_text = String::from("case1");
            // fix camera
            state.cam_x = center_start_x;

            // determine the end column
            let excluded_x = state.cam_x + width as u32;
            for i in col_start_xs.len() as u32..col_count {
                cursor += col_widths(i) as u32;
                if cursor > excluded_x {
                    return (state.centered_col, i - 1, 0);
                }
                cursor += col_gap as u32;
            }

            (state.centered_col, col_count - 1, 0)
        } else if state.cam_x + width as u32 - 1 <= center_end_x {
            state.d_text = String::from("case2");
            
            // fix camera
            state.cam_x = center_end_x + 1 - width as u32;

            // work backwards to determine the start column
            for (i, col_start_x) in col_start_xs.iter().enumerate().rev() {
                if *col_start_x < state.cam_x {
                    let x_offset = col_start_xs[i + 1] - state.cam_x;
                    return (i as u32 + 1, state.centered_col, x_offset as u16);
                }
            }

            (0, state.centered_col, 0)
        } else {
            state.d_text = String::from("case3");

            let start_col = match col_start_xs.binary_search(&state.cam_x) {
                Ok(i) => i as u32,
                Err(i) => i as u32,
            };
            let x_offset = col_start_xs.get(start_col as usize)
                .unwrap_or(&cursor)
                .saturating_sub(state.cam_x) as u16;

            let excluded_x = state.cam_x + width as u32;
            for i in col_start_xs.len() as u32..col_count {
                cursor += col_widths(i) as u32;
                if cursor > excluded_x {
                    return (start_col, i - 1, x_offset);
                }
                cursor += col_gap as u32;
            }

            (start_col, col_count - 1, x_offset)
        }
    }
}

impl Widget for &mut ShiftGrid<'_> {
    fn render(
        self,
        mut area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer
    ) where Self: Sized {
        if area.height <= 1 {
            return;
        }

        // do nothing if there are no rows or columns
        if self.row_count <= 0 || self.col_count <= 0 {
            return;
        }

        // fix centered row and column to be within valid range
        self.state.centered_row = self.state.centered_row.min(self.row_count - 1);
        self.state.centered_col = self.state.centered_col.min(self.col_count - 1);

        // the height allocated for cells
        let cells_height = if self.show_col_numbers {
            area.height.saturating_sub(1 + self.col_number_gap)
        } else {
            area.height
        };

        // determine the first and last row to render and its offset from the first y position
        let (start_row, end_row, y_offset) = ShiftGrid::get_row_start_end_offset(
            self.state,
            cells_height,
            self.row_count,
            self.row_gap,
        );

        // check that line numbers fit
        if area.width < self.row_number_width {
            return;
        }

        if self.show_row_numbers {
            // render row numbers
            for i in start_row..=end_row {
                let rect = Rect {
                    x: area.x,
                    y: area.y + (i - start_row) as u16 * (1 + self.row_gap) + y_offset + 1 + self.col_number_gap,
                    width: self.row_number_width,
                    height: 1,
                };

                let line = (*self.row_numbers)(i);
                line.render(rect, buf);
            }

            // calculate new area
            area = Rect {
                x: area.x + self.row_number_width + self.row_number_gap,
                width: area.width.saturating_sub(self.row_number_width + self.row_number_gap),
                ..area
            };
        }

        // ensure that the centered column fits
        if (self.col_widths)(self.state.centered_col) > area.width {
            return;
        }

        // determine first and last column to render
        let (start_col, end_col, x_offset) = ShiftGrid::get_col_start_end_offset(
            self.state,
            area.width,
            self.col_widths,
            self.col_count,
            self.col_gap,
        );
        self.state.d_start_col = start_col;
        self.state.d_end_col = end_col;
        self.state.d_x_offset = x_offset;

        if self.show_col_numbers {
            // render columns numbers
            let mut cursor = area.x + x_offset;

            for col in start_col..=end_col {
                let width = (self.col_widths)(col);
                let num_rect = Rect {
                    x: cursor,
                    y: area.y,
                    width,
                    height: 1,
                };
                (self.col_numbers)(col).render(num_rect, buf);
                cursor += width + self.col_gap;
            }

            area = Rect {
                y: area.y + 1 + self.col_number_gap,
                height: area.height.saturating_sub(1 + self.col_number_gap),
                ..area
            }
        }
        
        // render cells
        let factor = 1 + self.row_gap;
        let mut cursor = area.x + x_offset;
        for col in start_col..=end_col {
            // render column numbers
            let width = (self.col_widths)(col);

            // render cells
            for (relative_row, row) in (start_row..=end_row).enumerate() {
                let cell_rect = Rect {
                    x: cursor,
                    y: area.y + ShiftGrid::COL_NUMBER_SPACING + y_offset + factor * relative_row as u16,
                    width,
                    height: 1,
                };

                if row == self.state.centered_row && col == self.state.centered_col {
                    self.state.centered_area = cell_rect;
                }

                (self.cells)(row, col).render(cell_rect, buf);
            }

            cursor += width + self.col_gap;
        }

    }
}

/// a struct that tests a shift grid with randomly-generated column widths
#[allow(unused)]
pub struct ShiftGridTest<'a> {
    state: ShiftGridState,

    /// cells of the test, indexed by cells[row][col]
    cells: Vec<Vec<Line<'a>>>,

    /// the widths of the columns
    col_widths: Vec<u16>,

    /// whether to exit
    exit: bool,
}

#[allow(unused)]
impl<'a> ShiftGridTest<'a> {
    pub fn new<T>(
        width: usize,
        height: usize,
        col_width_range: T
    ) -> Self where T: SampleRange<u16> + Clone {
        let mut cells = Vec::with_capacity(height);

        let mut rng = rand::rng();

        // generate column widths
        let mut col_widths = Vec::with_capacity(width);
        for _ in 0..width {
            col_widths.push(rng.random_range(col_width_range.clone()));
        }

        // generate column heights
        for r in 0..height {
            let mut col = Vec::with_capacity(width);

            for c in 0..width {
                let col_width = col_widths[c] as usize;

                let base_text = format!("{}:{}", r, c);
                if base_text.len() < col_width {
                    col.push(Line::from(
                        format!("{}{}", base_text, "x".repeat(col_width - base_text.len()))
                    ));
                } else {
                    col.push(Line::from(
                        base_text[0..col_width].to_owned()
                    ));
                }
            }

            cells.push(col);
        }

        Self {
            state: Default::default(),
            cells,
            col_widths,
            exit: false,
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                type K = KeyCode;
                match key_event.code {
                    K::Char('h') => self.state.centered_col = self.state.centered_col.saturating_sub(1),
                    K::Char('l') => self.state.centered_col = self.state.centered_col.saturating_add(1)
                        .min(self.cells.get(0).unwrap_or(&Vec::new()).len() as u32),
                    K::Char('k') => self.state.centered_row = self.state.centered_row.saturating_sub(1),
                    K::Char('j') => self.state.centered_row = self.state.centered_row.saturating_add(1)
                        .min(self.cells.len().saturating_sub(1) as u32),
                    K::Char('q') => self.exit = true,
                    _ => (),
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }
}

impl<'a> Widget for &mut ShiftGridTest<'a> {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer
    ) where Self: Sized {
        let col_widths = |index: u32| -> u16 {
            self.col_widths[index as usize]
        };

        let cells = |row: u32, col: u32| -> Line<'_> {
            self.cells[row as usize][col as usize].clone()
        };

        let grid_area = Rect {
            height: area.height - 1,
            ..area
        };
        let info_area = Rect {
            height: 1,
            y: area.height - 1,
            ..area
        };

        let mut grid = ShiftGrid::new(&mut self.state)
            .row_number_width(ShiftGrid::base10_width((self.cells.len() - 1) as u32))
            .row_count(self.cells.len() as u32)
            .row_gap(0)
            .col_count(self.cells.get(0).unwrap_or(&Vec::new()).len() as u32)
            .col_gap(1)
            .col_number_gap(0)
            .col_widths(&col_widths)
            .cells(&cells);

        grid.render(grid_area, buf);

        buf.set_style(self.state.centered_area, Style::new().reversed());

        let info_line = Line::from(
            format!(
                "Dim: {}R:{}C, Cam {}R:{}C; Cel {}R:{}C; SC: {}; EC: {}; XO: {}; {}",
                self.cells.len(),
                self.cells.get(0).unwrap_or(&Vec::new()).len(),
                self.state.cam_y,
                self.state.cam_x,
                self.state.centered_row,
                self.state.centered_col,
                self.state.d_start_col,
                self.state.d_end_col,
                self.state.d_x_offset,
                self.state.d_text,
            )
        );

        info_line.render(info_area, buf);

    }
}
