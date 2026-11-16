use std::{error::Error, fmt};

/// Why an entry could not be evaluated, named as the calculator names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    DivideByZero,
    Domain,
    NonReal,
    Overflow,
    /// Valid calculator syntax that this calculator does not evaluate yet.
    Unsupported,
}

/// An evaluation error, with the token it was found at when known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalcError {
    pub kind: ErrorKind,
    pub position: Option<usize>,
}

impl CalcError {
    pub fn at(kind: ErrorKind, position: usize) -> Self {
        CalcError {
            kind,
            position: Some(position),
        }
    }

    pub fn new(kind: ErrorKind) -> Self {
        CalcError {
            kind,
            position: None,
        }
    }

    pub fn message(self) -> &'static str {
        match self.kind {
            ErrorKind::Syntax => "ERR:SYNTAX",
            ErrorKind::DivideByZero => "ERR:DIVIDE BY 0",
            ErrorKind::Domain => "ERR:DOMAIN",
            ErrorKind::NonReal => "ERR:NONREAL ANSWER",
            ErrorKind::Overflow => "ERR:OVERFLOW",
            ErrorKind::Unsupported => "ERR:UNSUPPORTED",
        }
    }
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl Error for CalcError {}
