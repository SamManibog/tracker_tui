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
