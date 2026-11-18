use std::{error::Error, fmt};

/// Why an entry could not be evaluated, named as the calculator names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    DivideByZero,
    Domain,
    NonReal,
    Overflow,
    /// A function given more or fewer arguments than it takes.
    Argument,
    /// fnInt( could not reach its tolerance.
    ToleranceNotMet,
    /// A Y= function read while empty.
    Undefined,
    /// A Y= function that reads itself, directly or through others.
    Invalid,
    /// CALC zero or intersect found no sign change to close in on.
    NoSignChange,
    /// A list where a number belongs, a matrix where a list belongs, and the like.
    DataType,
    /// Lists or matrices of different sizes combined element by element, or
    /// matrices whose sizes do not multiply.
    DimMismatch,
    /// A list or matrix element that does not exist, an empty list, or a
    /// matrix that is not square where one must be.
    InvalidDim,
    /// A matrix with no inverse.
    SingularMatrix,
    /// A WINDOW whose ranges do not increase or whose Xres is not 1 to 8.
    WindowRange,
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

    /// The error, placed at `position` unless it already has a place.
    pub fn or_at(self, position: usize) -> Self {
        CalcError {
            position: self.position.or(Some(position)),
            ..self
        }
    }

    pub fn message(self) -> &'static str {
        match self.kind {
            ErrorKind::Syntax => "ERR:SYNTAX",
            ErrorKind::DivideByZero => "ERR:DIVIDE BY 0",
            ErrorKind::Domain => "ERR:DOMAIN",
            ErrorKind::NonReal => "ERR:NONREAL ANSWER",
            ErrorKind::Overflow => "ERR:OVERFLOW",
            ErrorKind::Argument => "ERR:ARGUMENT",
            ErrorKind::ToleranceNotMet => "ERR:TOL NOT MET",
            ErrorKind::Undefined => "ERR:UNDEFINED",
            ErrorKind::Invalid => "ERR:INVALID",
            ErrorKind::NoSignChange => "ERR:NO SIGN CHNG",
            ErrorKind::DataType => "ERR:DATA TYPE",
            ErrorKind::DimMismatch => "ERR:DIM MISMATCH",
            ErrorKind::InvalidDim => "ERR:INVALID DIM",
            ErrorKind::SingularMatrix => "ERR:SINGULAR MAT",
            ErrorKind::WindowRange => "ERR:WINDOW RANGE",
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
