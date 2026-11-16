//! The calculator's domain, free of any UI framework: the keypad's keys, what
//! each press means under 2nd and ALPHA, the entry line's tokens and editing,
//! parsing and evaluating entries, and the state machine that turns key
//! presses into what the screen shows.

pub mod calculator;
pub mod editor;
pub mod error;
pub mod eval;
pub mod expr;
pub mod format;
pub mod home;
pub mod key;
pub mod keymap;
pub mod latex;
pub mod modifiers;
pub mod parse;
pub mod token;

pub use calculator::{Calculator, CursorStyle, Screen};
pub use error::{CalcError, ErrorKind};
pub use home::HistoryLine;
pub use key::{Key, UnknownKey};
pub use token::Token;
