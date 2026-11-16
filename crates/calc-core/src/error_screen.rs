use crate::{error::CalcError, token::Token, window_editor::WindowField};

/// The error screen's choices: back to the home screen, or back to where
/// the error was raised with the cursor at the fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorChoice {
    Quit,
    Goto,
}

impl ErrorChoice {
    pub const ALL: [ErrorChoice; 2] = [ErrorChoice::Quit, ErrorChoice::Goto];

    pub fn label(self) -> &'static str {
        match self {
            ErrorChoice::Quit => "Quit",
            ErrorChoice::Goto => "Goto",
        }
    }
}

/// Where an error was raised, which Goto returns to.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorOrigin {
    /// A home screen entry.
    Home(Vec<Token>),
    /// The Y= function in this slot.
    Function(usize),
    /// A WINDOW field and the entry typed into it.
    WindowField(WindowField, Vec<Token>),
    /// The WINDOW settings as a whole.
    Window,
}

/// The error screen: the error, where it was raised, and the highlighted
/// choice.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorMenu {
    pub error: CalcError,
    pub origin: ErrorOrigin,
    pub highlighted: ErrorChoice,
}

impl ErrorMenu {
    pub fn new(error: CalcError, origin: ErrorOrigin) -> Self {
        ErrorMenu {
            error,
            origin,
            highlighted: ErrorChoice::Quit,
        }
    }

    /// ▲ or ▼: the other choice (the menu has two).
    pub fn toggled(self) -> Self {
        let highlighted = match self.highlighted {
            ErrorChoice::Quit => ErrorChoice::Goto,
            ErrorChoice::Goto => ErrorChoice::Quit,
        };
        ErrorMenu {
            highlighted,
            ..self
        }
    }
}
