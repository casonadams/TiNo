//! The calculator's domain, free of any UI framework: the keypad's keys, what
//! each press means under 2nd and ALPHA, the entry line's tokens and editing,
//! parsing and evaluating entries under the MODE settings, and the state
//! machine that turns key presses into what the screen shows.

pub mod calculator;
pub mod editor;
pub mod error;
pub mod error_screen;
pub mod eval;
pub mod expr;
pub mod format;
pub mod home;
pub mod key;
pub mod keymap;
pub mod latex;
pub mod mode_screen;
pub mod modifiers;
pub mod parse;
pub mod screen;
pub mod settings;
pub mod token;
pub mod trig;

pub use calculator::Calculator;
pub use error::{CalcError, ErrorKind};
pub use error_screen::ErrorChoice;
pub use home::HistoryLine;
pub use key::{Key, UnknownKey};
pub use mode_screen::ModeRow;
pub use screen::{CursorStyle, ErrorView, HomeView, ModeRowView, ModeView, Screen, ScreenView};
pub use token::Token;
