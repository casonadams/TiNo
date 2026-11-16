//! The screen as JS values: one object with the shown screen's part set.

use nsh_core::prelude::*;

use crate::graphing::{
    FieldsScreen, FunctionsScreen, GraphScreen, MenuScreen, TableScreen, fields_screen,
    functions_screen, graph_screen, menu_screen, table_screen,
};

/// What the cursor shows: the mode the next key press is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum CursorStyle {
    /// Typing replaces the token under the cursor.
    Overwrite,
    /// Typing inserts before the token under the cursor (2nd INS).
    Insert,
    /// The next key reads its 2nd legend.
    Second,
    /// The next key reads its ALPHA legend.
    Alpha,
    /// Every key reads its ALPHA legend until ALPHA is pressed (2nd A-LOCK).
    AlphaLock,
}

impl From<calc_core::CursorStyle> for CursorStyle {
    fn from(style: calc_core::CursorStyle) -> Self {
        match style {
            calc_core::CursorStyle::Overwrite => CursorStyle::Overwrite,
            calc_core::CursorStyle::Insert => CursorStyle::Insert,
            calc_core::CursorStyle::Second => CursorStyle::Second,
            calc_core::CursorStyle::Alpha => CursorStyle::Alpha,
            calc_core::CursorStyle::AlphaLock => CursorStyle::AlphaLock,
        }
    }
}

/// Which screen is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum ScreenKind {
    /// Entries and results (`home` is set).
    Home,
    /// The MODE settings (`mode` is set).
    Mode,
    /// An error and its choices (`error` is set).
    Error,
    /// The Y= functions (`functions` is set).
    Functions,
    /// A settings screen, WINDOW or TBLSET (`fields` is set).
    Fields,
    /// The table (`table` is set).
    Table,
    /// The graph (`graph` is set).
    Graph,
    /// A numbered menu (`menu` is set).
    Menu,
}

/// An evaluated entry on the home screen.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct HistoryLine {
    /// Unique for the calculator's lifetime: the line's key in lists.
    pub id: u32,
    /// The entry as TeX.
    pub latex: String,
    /// The result as the calculator formats it.
    pub result: String,
}

/// The home screen.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct HomeScreen {
    /// Evaluated entries, oldest first.
    pub history: Vec<HistoryLine>,
    /// The entry line, one string per token.
    pub entry: Vec<String>,
    /// The index of the token under the cursor; `entry.length` after the last one.
    pub cursor: u32,
}

/// A MODE row: its options, the chosen one and, on the cursor's row, the highlighted one.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct ModeRow {
    /// The options, as the screen labels them.
    pub options: Vec<String>,
    /// The index of the chosen option.
    pub selected: u32,
    /// The index of the highlighted option, on the cursor's row only.
    pub highlighted: Option<u32>,
}

/// The MODE screen.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct ModeScreen {
    /// One row per setting.
    pub rows: Vec<ModeRow>,
}

/// The error screen.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct ErrorScreen {
    /// The error (`"ERR:SYNTAX"`).
    pub message: String,
    /// The choices, numbered from 1 on screen (`Quit`, `Goto`).
    pub choices: Vec<String>,
    /// The index of the highlighted choice.
    pub highlighted: u32,
}

/// What the calculator's screen shows.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct Screen {
    /// Which screen is shown; its part below is set.
    pub kind: ScreenKind,
    /// The settings in force, as the status bar words them (`NORMAL`, `FLOAT`, `RADIAN`).
    pub status: Vec<String>,
    /// What the cursor shows.
    pub cursor_style: CursorStyle,
    pub home: Option<HomeScreen>,
    pub mode: Option<ModeScreen>,
    pub error: Option<ErrorScreen>,
    pub functions: Option<FunctionsScreen>,
    pub fields: Option<FieldsScreen>,
    pub table: Option<TableScreen>,
    pub graph: Option<GraphScreen>,
    pub menu: Option<MenuScreen>,
}

fn index(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

impl From<calc_core::Screen<'_>> for Screen {
    fn from(screen: calc_core::Screen<'_>) -> Self {
        use calc_core::ScreenView;
        let base = Screen {
            kind: ScreenKind::Home,
            status: screen.status.to_vec(),
            cursor_style: screen.cursor_style.into(),
            home: None,
            mode: None,
            error: None,
            functions: None,
            fields: None,
            table: None,
            graph: None,
            menu: None,
        };
        match screen.view {
            ScreenView::Home(home) => Screen {
                home: Some(home_screen(home)),
                ..base
            },
            ScreenView::Mode(mode) => Screen {
                kind: ScreenKind::Mode,
                mode: Some(mode_screen(&mode)),
                ..base
            },
            ScreenView::Error(error) => Screen {
                kind: ScreenKind::Error,
                error: Some(error_screen(&error)),
                ..base
            },
            ScreenView::Functions(functions) => Screen {
                kind: ScreenKind::Functions,
                functions: Some(functions_screen(&functions)),
                ..base
            },
            ScreenView::Fields(fields) => Screen {
                kind: ScreenKind::Fields,
                fields: Some(fields_screen(&fields)),
                ..base
            },
            ScreenView::Table(table) => Screen {
                kind: ScreenKind::Table,
                table: Some(table_screen(&table)),
                ..base
            },
            ScreenView::Graph(graph) => Screen {
                kind: ScreenKind::Graph,
                graph: Some(graph_screen(&graph)),
                ..base
            },
            ScreenView::Menu(menu) => Screen {
                kind: ScreenKind::Menu,
                menu: Some(menu_screen(&menu)),
                ..base
            },
        }
    }
}

fn home_screen(home: calc_core::HomeView<'_>) -> HomeScreen {
    let history = home.history.iter().map(|line| HistoryLine {
        id: line.id,
        latex: line.output.latex.clone(),
        result: line.output.result.clone(),
    });
    HomeScreen {
        history: history.collect(),
        entry: home.entry.iter().map(|t| t.text().to_owned()).collect(),
        cursor: index(home.cursor),
    }
}

fn mode_screen(mode: &calc_core::ModeView) -> ModeScreen {
    let rows = mode.rows.iter().map(|row| ModeRow {
        options: row
            .options
            .iter()
            .map(|&option| option.to_owned())
            .collect(),
        selected: index(row.selected),
        highlighted: row.highlighted.map(index),
    });
    ModeScreen {
        rows: rows.collect(),
    }
}

fn error_screen(error: &calc_core::ErrorView) -> ErrorScreen {
    let highlighted = error.choices.iter().position(|&c| c == error.highlighted);
    ErrorScreen {
        message: error.message.to_owned(),
        choices: error.choices.iter().map(|c| c.label().to_owned()).collect(),
        highlighted: index(highlighted.unwrap_or(0)),
    }
}
