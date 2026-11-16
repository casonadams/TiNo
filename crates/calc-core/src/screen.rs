use std::collections::VecDeque;

use crate::{error_screen::ErrorChoice, home::HistoryLine, mode_screen::ModeRow, token::Token};

/// What the cursor shows: the mode the next key press is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Overwrite,
    Insert,
    Second,
    Alpha,
    AlphaLock,
}

/// What the screen shows: the status bar's words and the current screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Screen<'a> {
    /// The settings in force, as the status bar shows them (`NORMAL FLOAT RADIAN`).
    pub status: [String; 3],
    pub cursor_style: CursorStyle,
    pub view: ScreenView<'a>,
}

impl<'a> Screen<'a> {
    /// The home screen, when it is the one shown.
    pub fn home(&self) -> Option<&HomeView<'a>> {
        match &self.view {
            ScreenView::Home(home) => Some(home),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScreenView<'a> {
    Home(HomeView<'a>),
    Mode(ModeView),
    Error(ErrorView),
}

/// The home screen: evaluated entries over the entry line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HomeView<'a> {
    pub history: &'a VecDeque<HistoryLine>,
    pub entry: &'a [Token],
    pub cursor: usize,
}

/// The MODE screen: a row per setting.
#[derive(Debug, Clone, PartialEq)]
pub struct ModeView {
    pub rows: Vec<ModeRowView>,
}

/// A MODE row: its options, the chosen one, and the highlighted one when
/// the cursor is on this row.
#[derive(Debug, Clone, PartialEq)]
pub struct ModeRowView {
    pub row: ModeRow,
    pub options: &'static [&'static str],
    pub selected: usize,
    pub highlighted: Option<usize>,
}

/// The error screen: the error and its choices.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorView {
    pub message: &'static str,
    pub choices: [ErrorChoice; 2],
    pub highlighted: ErrorChoice,
}
