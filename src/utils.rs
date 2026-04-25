use crate::phrase_edit_command::PhraseEditCommand;

/// the result of handling input on a page
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PageCommand<T> {
    /// some result
    Command(T),

    /// exit the editor
    Quit,

    /// no operation
    Nop,
}

impl<T> PageCommand<T> {
    pub fn is_cmd(&self) -> bool {
        if let PageCommand::Command(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_quit(&self) -> bool {
        if let PageCommand::Quit = self {
            true
        } else {
            false
        }
    }

    pub fn is_nop(&self) -> bool {
        if let PageCommand::Nop = self {
            true
        } else {
            false
        }
    }

}

/// an app-level command
#[derive(Debug, Clone)]
pub enum AppCommand {
    Project(ProjectCommand),
    
    /// a phrase-level command
    Phrase{ phrase_id: u32, command: Box<PhraseEditCommand> },
}

/// a project-level command
#[derive(Debug, Clone)]
pub enum ProjectCommand {
    /// sets the tempo in whole notes per second
    SetTempo(f64),
}

