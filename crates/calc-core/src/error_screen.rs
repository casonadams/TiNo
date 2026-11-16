use crate::{error::CalcError, token::Token};

/// The error screen's choices: back to an empty home line, or back to the
/// entry with the cursor at the fault.
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

/// The error screen: the error, the entry that raised it, and the
/// highlighted choice.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorMenu {
    pub error: CalcError,
    pub entry: Vec<Token>,
    pub highlighted: ErrorChoice,
}

impl ErrorMenu {
    pub fn new(error: CalcError, entry: Vec<Token>) -> Self {
        ErrorMenu {
            error,
            entry,
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
