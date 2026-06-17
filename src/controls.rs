use std::fmt::Display;

use ratatui::{crossterm::event::{KeyEvent, KeyModifiers}, text::Line};

use crate::keybinds::{Direction, KeybindMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Normal,
    Insert,
    Modify,
    Select,
}

impl Default for EditorMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl Display for EditorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Self::Normal => "Normal",
            Self::Insert => "Insert",
            Self::Modify => "Modify",
            Self::Select => "Select",
        };

        write!(f, "{}", string)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlCommand<'a> {
    ChangeMode(EditorMode),
    InsertKey(char),
    ModifyDirection{direction: Direction, delta: u32},
    MoveDirection{direction: Direction, delta: u32},
    Teleport{row: u32, col: u32},
    CenterCamera,
    Unknown(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlCommandKind {
    ChangeMode(EditorMode),
    Modify,
    Move,
    Teleport,
    CenterCamera,
}

#[derive(Debug, Clone, Default)]
pub struct ControlResult<'a> {
    pub new_mode: Option<EditorMode>,
    pub status_message: Option<Line<'a>>,
    pub ruler: Option<Line<'a>>,
    pub message: Option<Line<'a>>,
}

#[derive(Debug, Clone)]
pub struct EditorController<'a> {
    mode: EditorMode,
    message: Line<'a>,
    input_string: String,
    number_control: u32,
    keybinds: KeybindMap,
}

impl<'a> EditorController<'a> {
    pub fn new(keybinds: KeybindMap) -> Self {
        Self {
            mode: EditorMode::default(),
            message: Line::from(""),
            input_string: String::default(),
            number_control: 0,
            keybinds
        }
    }
}

