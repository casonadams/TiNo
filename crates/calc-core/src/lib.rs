//! The calculator's domain, free of any UI framework: the keypad's keys, what
//! each press means under 2nd and ALPHA, the entry line's tokens and editing,
//! parsing and evaluating entries under the MODE settings, graphing the Y=
//! functions in the WINDOW, and the state machine that turns key presses
//! into what the screen shows.

pub mod calculator;
pub mod editor;
pub mod error;
pub mod error_screen;
pub mod eval;
pub mod expr;
pub mod field_editor;
pub mod format;
pub mod functions;
pub mod home;
pub mod key;
pub mod keymap;
pub mod latex;
pub mod list_menu;
pub mod menus;
pub mod mode_screen;
pub mod modifiers;
pub mod numeric;
pub mod parse;
pub mod plot;
pub mod screen;
pub mod settings;
pub mod table;
pub mod token;
pub mod trace;
pub mod trig;
pub mod vocabulary;
pub mod window;
pub mod window_editor;
pub mod y_editor;
pub mod zoom;

pub use calculator::{Calculator, LoadError};
pub use error::{CalcError, ErrorKind};
pub use error_screen::ErrorChoice;
pub use home::HistoryLine;
pub use key::{Key, UnknownKey};
pub use mode_screen::ModeRow;
pub use screen::{
    CursorStyle, ErrorView, FieldRowView, FieldsView, FunctionRowView, FunctionsView, GraphView,
    HomeView, MenuItemView, MenuView, ModeRowView, ModeView, RowFocus, Screen, ScreenView,
    TableView, TraceView,
};
pub use token::Token;
pub use window_editor::WindowField;
