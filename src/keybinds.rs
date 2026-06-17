use std::{fmt::{Display, Write}, ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign}, str::FromStr};

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::controls::ControlCommandKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        };

        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyMods(u8);

impl KeyMods {
    pub const EMPTY: Self = Self(0);
    pub const SHIFT: Self = Self(1 << 0);
    pub const CONTROL: Self = Self(1 << 1);
    pub const ALT: Self = Self(1 << 2);

    fn try_from_char(value: char) -> Option<Self> {
        match value.to_ascii_lowercase() {
            'c' => Some(KeyMods::CONTROL),
            'a' => Some(KeyMods::ALT),
            's' => Some(KeyMods::SHIFT),
            _ => None,
        }
    }
}

impl Display for KeyMods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::with_capacity(3);
        let mut s = String::with_capacity(5);

        if *self ^ Self::CONTROL == Self::EMPTY {
            v.push("C");
        }

        if *self ^ Self::ALT == Self::EMPTY {
            v.push("A");
        }

        if *self ^ Self::SHIFT == Self::EMPTY {
            v.push("S");
        }

        let mut v_iter = v.iter();
        if let Some(c) = v_iter.next() {
            s.push_str(c);
        }

        for c in v_iter {
            s.push_str("-");
            s.push_str(c);
        }

        f.write_str(&s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyModParseError {
    DuplicateModifier(char),
    UnknownModifier(char),
    ExpectedDash(char),
    UnexpectedDash,
}

impl Display for KeyModParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateModifier(c) => write!(
                f,
                "Duplicate modifier '{c}'."
            ),
            Self::UnknownModifier(c) => write!(
                f,
                "Unknown modifier '{c}'. Expected one of 'C' (Control), 'A' (Alt), or 'S' (Shift)."
            ),
            Self::ExpectedDash(c) => write!(
                f,
                "Improper formatting. Expected '-', got '{c}'."
            ),
            Self::UnexpectedDash => write!(
                f,
                "Improper formatting. Unexpected dash."
            ),
        }
    }
}

impl FromStr for KeyMods {
    type Err = KeyModParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(KeyMods::EMPTY);
        }

        let mut s_iter = s.chars();
        let mut mods = Self::EMPTY;

        while let Some(c) = s_iter.next() {
            if let Some(modifier) = KeyMods::try_from_char(c) {
                if mods & modifier != KeyMods::EMPTY {
                    return Err(KeyModParseError::DuplicateModifier(c));
                }
                mods |= modifier;
            } else {
                return Err(KeyModParseError::UnknownModifier(c));
            }

            if let Some(dash) = s_iter.next() {
                if dash != '-' {
                    return Err(KeyModParseError::ExpectedDash(dash));
                }
            } else {
                return Ok(mods);
            }
        }

        Err(KeyModParseError::UnexpectedDash)
    }
}

impl Default for KeyMods {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl BitOr for KeyMods {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0.bitor(rhs.0))
    }
}

impl BitXor for KeyMods {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0.bitxor(rhs.0))
    }
}

impl BitAnd for KeyMods {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0.bitand(rhs.0))
    }
}

impl BitOrAssign for KeyMods {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitXorAssign for KeyMods {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl BitAndAssign for KeyMods {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Char(char),
    Direction(Direction),
    Backspace,
    Delete,
    Enter,
    Escape,
    Tab,
}

impl Key {
    pub fn try_from_key_code(code: ratatui::crossterm::event::KeyCode) -> Option<Self> {
        let key = match event.code {
            KeyCode::Char(c) => Key::Char(c),
            KeyCode::Up => Key::Direction(Direction::Up),
            KeyCode::Down => Key::Direction(Direction::Down),
            KeyCode::Left => Key::Direction(Direction::Left),
            KeyCode::Right => Key::Direction(Direction::Right),
            KeyCode::Backspace => Key::Backspace,
            KeyCode::Delete => Key::Delete,
            KeyCode::Enter => Key::Enter,
            KeyCode::Esc => Key::Escape,
            KeyCode::Tab => Key::Tab,
            _ => return None,
        };
        Some(key)
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char(c) => f.write_char(*c),
            Self::Direction(d) => write!(f, "{}", d),
            Self::Backspace => f.write_str("BS"),
            Self::Delete => f.write_str("Del"),
            Self::Enter => f.write_str("CR"),
            Self::Escape => f.write_str("Esc"),
            Self::Tab => f.write_str("Tab"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyPress {
    mods: KeyMods,
    key: Key,
}

impl KeyPress {
    pub fn new(key: Key, mods: KeyMods) -> Self {
        if let Key::Char(c) = key && c.is_ascii_uppercase() {
            Self {
                key: Key::Char(c.to_ascii_lowercase()),
                mods: mods | KeyMods::SHIFT
            }
        } else {
            Self { key, mods }
        }
    }

    pub fn key(&self) -> Key {
        self.key
    }

    pub fn mods(&self) -> KeyMods {
        self.mods
    }

    pub fn try_from_key_event(event: &KeyEvent) -> Option<Self> {
        if !event.is_press() {
            return None;
        }

        let mut mods = KeyMods::EMPTY;
        if event.modifiers & KeyModifiers::SHIFT == KeyModifiers::SHIFT {
            mods |= KeyMods::SHIFT;
        }
        if event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
            mods |= KeyMods::CONTROL;
        }
        if event.modifiers & KeyModifiers::CONTROL == KeyModifiers::CONTROL {
            mods |= KeyMods::CONTROL;
        }

        let key = Key::try_from_key_code(event.code)?;

        Some(Self::new(key, mods))
    }

    pub fn keystroke_to_string(keystroke: &[Self]) -> String {
        let mut s = String::new();
        for k in keystroke {
            s.push_str(&k.to_string());
        }
        s
    }
}

impl From<Key> for KeyPress {
    fn from(value: Key) -> Self {
        Self { mods: KeyMods::EMPTY, key: value }
    }
}

impl Display for KeyPress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.mods == KeyMods::EMPTY {
            write!(f, "{}", self.key())
        } else {
            write!(f, "<{}-{}>", self.mods, self.key())
        }
    }
}

pub trait KeybindEntry {
    fn keystroke(&self) -> Vec<KeyPress>;
    fn action(&self) -> ControlCommandKind;
}

#[derive(Debug, Clone)]
pub enum KeybindMapBuildError {
    DuplicateKeybind(String),
    ShadowingKeybind(String),
}

impl KeybindMapBuildError {
    fn extend(self, key_press: KeyPress) -> Self {
        match self {
            Self::DuplicateKeybind(s) => {
                Self::DuplicateKeybind(format!("{}{}", key_press, s))
            },
            Self::ShadowingKeybind(s) => {
                Self::ShadowingKeybind(format!("{}{}", key_press, s))
            },
        }
    }
}

/// huffman tree-based keybind map
#[derive(Debug, Clone)]
pub struct KeybindMap {
    root: KeybindNode,
}

impl KeybindMap {
    /// creates an empty keybind map
    pub fn new() -> Self {
        Self {
            root: KeybindNode {
                key_press: Key::Backspace.into(),
                action: None,
                children: Vec::new(),
            }
        }
    }

    /// gets the action based on the corresponding keystroke
    pub fn get_action(&self, keystroke: &[KeyPress]) -> Option<ControlCommandKind> {
        self.root.get_action(keystroke)
    }

    /// attempts to create a keybind map from an iterator over keybind entries
    /// fails if:
    ///     1) there equivalent keystrokes are mapped to different actions
    ///     2) a keystroke exactly matches an initial substring of another keybind (keybind shadows
    ///        other ones)
    pub fn try_from_iterator<S: KeybindEntry, T: Iterator<Item = S>>(iter: T) -> Option<Self> {
        let mut map = Self::new();

    }
}

#[derive(Debug, Clone)]
struct KeybindNode {
    key_press: KeyPress,
    action: Option<ControlCommandKind>,
    children: Vec<Box<KeybindNode>>,
}

impl KeybindNode {
    /// attempts to get the action based on the given keystroke
    fn get_action(&self, keystroke: &[KeyPress]) -> Option<ControlCommandKind> {
        if keystroke.is_empty() {
            return self.action;
        }

        let bs_result = self.children.binary_search_by_key(&keystroke[0], |child| child.key_press);
        match bs_result {
            Ok(i) => self.children[i].get_action(keystroke.split_at(1).1),
            Err(_) => None,
        }
    }

    /// attempts to insert a keystroke into the node, returning true on success
    fn insert_keystroke(
        &mut self,
        keystroke: &[KeyPress],
        action: ControlCommandKind
    ) -> Result<(), KeybindMapBuildError> {
        type ERR = KeybindMapBuildError;

        if self.action.is_some() {
            if keystroke.is_empty() {
                return Err(ERR::DuplicateKeybind(self.key_press.to_string()));
            } else {
                return Err(ERR::ShadowingKeybind(self.key_press.to_string()));
            }
        } else if keystroke.is_empty() && !self.children.is_empty() {
            return Err(ERR::ShadowingKeybind(KeyPress::keystroke_to_string(keystroke)));
        }

        let bs_result = self.children.binary_search_by_key(&keystroke[0], |child| child.key_press);
        let idx = match bs_result {
            Ok(i) => i,
            Err(i) => {
                let node = Box::new(KeybindNode {
                    key_press: keystroke[0],
                    action: None,
                    children: Vec::new()
                });
                self.children.insert(i, node);
                i
            }
        };
        let node = &mut self.children[idx];

        match node.insert_keystroke(keystroke.split_at(1).1, action) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.extend(self.key_press)),
        }
    }
}
