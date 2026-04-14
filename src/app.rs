use std::{collections::HashMap, io};

use ratatui::{DefaultTerminal, Frame, crossterm::event::{self, Event, KeyEventKind}, layout::Rect, widgets::Widget};

use crate::{Note, Phrase, PhraseEditor, PhraseEffect, PhraseTransitionMode, utils::PageCommand};

/// the page this app is currently on
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AppPage {
    Phrase
}

#[derive(Debug)]
pub struct TuiTrackerApp {
    /// a map from a number to its phrase
    phrases: HashMap<u32, Box<Phrase>>,

    /// the current page we are on
    page: AppPage,

    /// the editor for the current phrase
    phrase_editor: PhraseEditor,

    /// the current phrase
    current_phrase: u32,

    /// whether the app should exit
    exit: bool,
}

impl TuiTrackerApp {
    /// create a new instance of the app
    /// note: this is currently a test function
    pub fn new() -> Self {
        let mut phrase = Box::new(Phrase::new(16, 16));
        phrase.set_note(0, 0, Some(Note::C4.add_semitones(1)));
        phrase.set_note(3, 0, Some(Note::A4));
        phrase.set_note(7, 0, Some(Note::C0));
        phrase.set_note(11, 0, Some(Note::B9));
        phrase.set_note(2, 1, Some(Note::B9));
        phrase.set_note(6, 1, Some(Note::B9));
        phrase.set_note(10, 1, Some(Note::B9));
        phrase.set_note(14, 1, Some(Note::B9));
        phrase.set_effect(0, 0, Some(PhraseEffect::Silence));
        phrase.set_effect(0, 1, Some(PhraseEffect::Silence));
        phrase.set_effect(0, 2, Some(PhraseEffect::SetTransition(PhraseTransitionMode::Release)));
        phrase.set_effect(3, 0, Some(PhraseEffect::SetTransition(PhraseTransitionMode::Lerp)));
        phrase.set_effect(3, 1, Some(PhraseEffect::SetTransition(PhraseTransitionMode::Release)));
        phrase.set_effect(12, 0, Some(PhraseEffect::Silence));

        let mut phrases = HashMap::new();
        phrases.insert(0, phrase);
        Self {
            phrases,
            page: AppPage::Phrase,
            phrase_editor: PhraseEditor::new(),
            current_phrase: 0,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match self.page {
                    AppPage::Phrase => {
                        let phrase = self.phrases.get_mut(&self.current_phrase)
                            .expect("ensure initialize for debug only, remove this later");
                        let page_cmd = self.phrase_editor.handle_key_event(&phrase, key_event);
                        if page_cmd.is_quit() {
                            self.exit = true;
                        } else if let PageCommand::Command(cmd) = page_cmd {
                            cmd.execute(phrase);
                        }
                    },
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &mut TuiTrackerApp {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer
    ) where Self: Sized {
        if area.height < 3 {
            return;
        }

        let page_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height - 2
        };
        let cmd_line_area = Rect {
            x: area.x,
            y: area.height - 1,
            width: area.width,
            height: 1
        };
        let status_line_area = Rect {
            x: area.x,
            y: area.height - 2,
            width: area.width,
            height: 1
        };

        match self.page {
            AppPage::Phrase => {
                let phrase = self.phrases.get(&self.current_phrase)
                    .expect("render should be called after phrase is valid");
                self.phrase_editor.render(phrase, true, page_area, buf);
            }
        }
    }

}
