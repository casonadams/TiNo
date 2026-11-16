//! The calculator's domain, free of any UI framework: the keypad's keys, what
//! each press means under 2nd and ALPHA, the entry line's tokens and editing,
//! and the state machine that turns key presses into what the screen shows.

pub mod calculator;
pub mod editor;
pub mod key;
pub mod keymap;
pub mod token;

pub use calculator::{Calculator, CursorStyle, Screen};
pub use key::{Key, UnknownKey};
pub use token::Token;
