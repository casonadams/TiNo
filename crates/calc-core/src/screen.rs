use std::collections::VecDeque;

use crate::{
    error_screen::ErrorChoice,
    graph_format::GraphFormat,
    home::{HistoryLine, HistorySelection},
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

/// An options screen, MODE or FORMAT: a line per setting.
#[derive(Debug, Clone, PartialEq)]
pub struct ModeView {
    pub rows: Vec<ModeRowView>,
}

/// A line of options: the options, the chosen one, and the highlighted one
/// when the cursor is on this line.
#[derive(Debug, Clone, PartialEq)]
pub struct ModeRowView {
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

/// The table (TABLE, or STAT EDIT's lists): column headers, the rows shown,
/// the selected cell (its column counted among those shown), the selected
/// value written out, and what is typed into the selected cell.
#[derive(Debug, Clone, PartialEq)]
pub struct TableView {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub selected_row: usize,
    pub selected_column: usize,
    /// `X=2`, `Y1=4.5` or `L₁(3)=7`; just `L₁(3)=` while a value is typed.
    pub footer: String,
    pub editing: Option<(Vec<Token>, usize)>,
}

/// The graph screen: the window, the sampled functions, tick positions, the
/// FORMAT settings, the trace cursor when tracing, the free cursor when it
/// is out, and a CALC item under way.
#[derive(Debug, Clone, PartialEq)]
pub struct GraphView {
    pub window: Window,
    pub plot: Plot,
    pub x_ticks: Vec<f64>,
    pub y_ticks: Vec<f64>,
    pub format: GraphFormat,
    pub trace: Option<TraceView>,
    pub free: Option<FreeView>,
    pub calc: Option<CalcView>,
}

/// The free cursor's point and its texts (`X=1.5`), with ZBox's first
/// corner once marked.
#[derive(Debug, Clone, PartialEq)]
pub struct FreeView {
    pub x: f64,
    pub y: f64,
    pub x_text: String,
    pub y_text: String,
    pub corner: Option<(f64, f64)>,
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

/// A menu: its tabs (one for ZOOM), the shown tab's items, and the
/// highlighted one. The screen shows as many as fit, keeping the
/// highlighted one in view.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuView {
    pub tabs: Vec<&'static str>,
    pub tab: usize,
    pub items: Vec<MenuItemView>,
    pub highlighted: usize,
}

/// A menu item and the key that picks it (`1`, `0`, `A`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuItemView {
    pub key: char,
    pub label: &'static str,
}
