//! The Y=, WINDOW, graph and menu screens as JS values.

use nsh_core::prelude::*;

/// A Y= row.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct FunctionRow {
    /// The function's name: `Y1` … `Y9`, `Y0`.
    pub label: String,
    /// Its entry, one string per token.
    pub entry: Vec<String>,
    /// Whether it is graphed (its `=` is highlighted).
    pub enabled: bool,
    /// On the focused row in its entry: the index of the token under the cursor.
    pub cursor: Option<u32>,
    /// On the focused row: whether the cursor is on the `=` sign.
    pub on_selector: bool,
}

/// The Y= screen.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct FunctionsScreen {
    pub rows: Vec<FunctionRow>,
}

/// A settings row (WINDOW, TBLSET).
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct FieldRow {
    /// `Xmin`, `TblStart`, …
    pub label: String,
    /// The value as the calculator writes it.
    pub value: String,
    /// On the focused row: what is typed, one string per token.
    pub entry: Option<Vec<String>>,
    /// On the focused row: the index of the token under the cursor.
    pub cursor: Option<u32>,
}

/// A settings screen of typed values (WINDOW, TBLSET).
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct FieldsScreen {
    /// `WINDOW` or `TABLE SETUP`.
    pub title: String,
    pub rows: Vec<FieldRow>,
}

/// The table of X and the graphed functions' values.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct TableScreen {
    /// `X`, then the shown functions' names.
    pub headers: Vec<String>,
    /// The rows shown, one string per column; `ERROR` where a function is undefined.
    pub rows: Vec<Vec<String>>,
    pub selected_row: u32,
    /// The selected column, counted among those shown.
    pub selected_column: u32,
    /// The selected value written out: `X=2`, `Y1=4.5`.
    pub footer: String,
}

/// One function's values at the graph's x positions; `null` where undefined.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct Curve {
    /// The Y= slot: 0 for `Y1` … 9 for `Y0`; it picks the curve's color.
    pub slot: u32,
    pub ys: Vec<Option<f64>>,
}

/// The trace cursor.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct Trace {
    pub slot: u32,
    /// `Y1=X²`.
    pub equation: String,
    pub x: f64,
    pub y: Option<f64>,
    /// `X=1.5`.
    pub x_text: String,
    /// `Y=2.25`, or `Y=` where the function is undefined.
    pub y_text: String,
}

/// The graph screen: the window's ranges, the sampled functions, tick positions and the trace.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct GraphScreen {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    /// The x positions every curve is sampled at.
    pub xs: Vec<f64>,
    pub curves: Vec<Curve>,
    pub x_ticks: Vec<f64>,
    pub y_ticks: Vec<f64>,
    pub trace: Option<Trace>,
}

/// A menu item and the digit that picks it.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct MenuItem {
    pub digit: u32,
    pub label: String,
}

/// A numbered menu (ZOOM).
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct MenuScreen {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub highlighted: u32,
}

fn index(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn texts(tokens: &[calc_core::Token]) -> Vec<String> {
    tokens.iter().map(|t| t.text().to_owned()).collect()
}

pub fn functions_screen(view: &calc_core::FunctionsView<'_>) -> FunctionsScreen {
    let rows = view.rows.iter().map(|row| FunctionRow {
        label: row.label.clone(),
        entry: texts(row.entry),
        enabled: row.enabled,
        cursor: match row.focus {
            Some(calc_core::RowFocus::Entry(cursor)) => Some(index(cursor)),
            _ => None,
        },
        on_selector: row.focus == Some(calc_core::RowFocus::Selector),
    });
    FunctionsScreen {
        rows: rows.collect(),
    }
}

pub fn fields_screen(view: &calc_core::FieldsView<'_>) -> FieldsScreen {
    let rows = view.rows.iter().map(|row| FieldRow {
        label: row.label.to_owned(),
        value: row.value.clone(),
        entry: row.editing.map(|(entry, _)| texts(entry)),
        cursor: row.editing.map(|(_, cursor)| index(cursor)),
    });
    FieldsScreen {
        title: view.title.to_owned(),
        rows: rows.collect(),
    }
}

pub fn table_screen(view: &calc_core::TableView) -> TableScreen {
    TableScreen {
        headers: view.headers.clone(),
        rows: view.rows.clone(),
        selected_row: index(view.selected_row),
        selected_column: index(view.selected_column),
        footer: view.footer.clone(),
    }
}

pub fn graph_screen(view: &calc_core::GraphView) -> GraphScreen {
    let curves = view.plot.curves.iter().map(|curve| Curve {
        slot: index(curve.slot),
        ys: curve.ys.clone(),
    });
    let trace = view.trace.as_ref().map(|trace| Trace {
        slot: index(trace.slot),
        equation: trace.equation.clone(),
        x: trace.x,
        y: trace.y,
        x_text: trace.x_text.clone(),
        y_text: trace.y_text.clone(),
    });
    GraphScreen {
        x_min: view.window.x_min,
        x_max: view.window.x_max,
        y_min: view.window.y_min,
        y_max: view.window.y_max,
        xs: view.plot.xs.clone(),
        curves: curves.collect(),
        x_ticks: view.x_ticks.clone(),
        y_ticks: view.y_ticks.clone(),
        trace,
    }
}

pub fn menu_screen(view: &calc_core::MenuView) -> MenuScreen {
    let items = view.items.iter().map(|item| MenuItem {
        digit: u32::from(item.digit),
        label: item.label.to_owned(),
    });
    MenuScreen {
        title: view.title.to_owned(),
        items: items.collect(),
        highlighted: index(view.highlighted),
    }
}
