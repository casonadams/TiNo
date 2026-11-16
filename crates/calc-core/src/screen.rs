use std::collections::VecDeque;

use crate::{
    error_screen::ErrorChoice,
    home::{HistoryLine, HistorySelection},
    mode_screen::ModeRow,
    plot::Plot,
    token::Token,
    window::Window,
};

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
    Functions(FunctionsView<'a>),
    Fields(FieldsView<'a>),
    Table(TableView),
    Graph(Box<GraphView>),
    Menu(MenuView),
}

/// The home screen: evaluated entries over the entry line, with the entry
/// or result ▲ ▼ highlight.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HomeView<'a> {
    pub history: &'a VecDeque<HistoryLine>,
    pub entry: &'a [Token],
    pub cursor: usize,
    pub selected: Option<HistorySelection>,
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

/// The Y= screen: one row per function.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionsView<'a> {
    pub rows: Vec<FunctionRowView<'a>>,
}

/// A Y= row: its name, entry and whether it is graphed; on the focused row,
/// where the cursor is.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRowView<'a> {
    pub label: String,
    pub entry: &'a [Token],
    pub enabled: bool,
    pub focus: Option<RowFocus>,
}

/// Where the cursor is on the focused Y= row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowFocus {
    /// On the `=` sign.
    Selector,
    /// In the entry, at this token.
    Entry(usize),
}

/// A settings screen of typed values (WINDOW, TBLSET): one row per field.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldsView<'a> {
    pub title: &'static str,
    pub rows: Vec<FieldRowView<'a>>,
}

/// A settings row: its label and value, or the entry being typed into the
/// focused one with the cursor's position.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldRowView<'a> {
    pub label: &'static str,
    pub value: String,
    pub editing: Option<(&'a [Token], usize)>,
}

/// The table: column headers, the rows shown, the selected cell (its
/// column counted among those shown) and the selected value written out.
#[derive(Debug, Clone, PartialEq)]
pub struct TableView {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub selected_row: usize,
    pub selected_column: usize,
    /// `X=2` or `Y1=4.5`.
    pub footer: String,
}

/// The graph screen: the window, the sampled functions, tick positions, the
/// trace cursor when tracing, and a CALC item under way.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphView {
    pub window: Window,
    pub plot: Plot,
    pub x_ticks: Vec<f64>,
    pub y_ticks: Vec<f64>,
    pub trace: Option<TraceView>,
    pub calc: Option<CalcView>,
}

/// What a CALC item shows over the graph.
#[derive(Debug, Clone, PartialEq)]
pub struct CalcView {
    /// The prompt waiting for a point (`Left Bound?`), or the result's title (`Zero`).
    pub heading: Option<&'static str>,
    /// value: the X typed so far and the cursor in it.
    pub entry: Option<(Vec<Token>, usize)>,
    /// The result's own value as the calculator writes it (`dy/dx=4`).
    pub value: Option<String>,
    /// The interval of x shaded under the traced function, for ∫f(x)dx.
    pub shade: Option<(f64, f64)>,
}

/// The trace cursor's function and point, with the bottom line's texts.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceView {
    pub slot: usize,
    /// The function as typed: `Y1=X²`.
    pub equation: String,
    pub x: f64,
    pub y: Option<f64>,
    /// The point as the calculator writes it: `X=1.5`, `Y=2.25` (`Y=` alone where undefined).
    pub x_text: String,
    pub y_text: String,
}

/// Menu items the screen shows at once, below the tabs.
pub const MENU_ROWS: usize = 9;

/// A menu: its tabs (one for ZOOM), the shown items of the shown tab, and
/// the highlighted one among them.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuView {
    pub tabs: Vec<&'static str>,
    pub tab: usize,
    pub items: Vec<MenuItemView>,
    pub highlighted: usize,
}

impl MenuView {
    /// At most `MENU_ROWS` items, scrolled so the highlighted one is the
    /// last shown when it lies below the first `MENU_ROWS`.
    pub fn scrolled(self) -> Self {
        let first = (self.highlighted + 1).saturating_sub(MENU_ROWS);
        MenuView {
            items: self.items.into_iter().skip(first).take(MENU_ROWS).collect(),
            highlighted: self.highlighted - first,
            ..self
        }
    }
}

/// A menu item and the key that picks it (`1`, `0`, `A`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItemView {
    pub key: char,
    pub label: &'static str,
}
