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

/// The table of X and the graphed functions' values, or STAT EDIT's lists.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct TableScreen {
    /// `X` and the shown functions' names, or the shown lists' names.
    pub headers: Vec<String>,
    /// The rows shown, one string per column; `ERROR` where a function is undefined.
    pub rows: Vec<Vec<String>>,
    pub selected_row: u32,
    /// The selected column, counted among those shown.
    pub selected_column: u32,
    /// The selected value written out: `X=2`, `Y1=4.5`, `L₁(3)=7`; just `L₁(3)=` while a value is typed.
    pub footer: String,
    /// In STAT EDIT, what is typed into the selected cell, one string per token.
    pub entry: Option<Vec<String>>,
    /// The index of the token under the cursor in `entry`.
    pub entry_cursor: u32,
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

/// What a CALC item shows over the graph.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct CalcPanel {
    /// The prompt waiting for a point (`Left Bound?`), or the result's title (`Zero`).
    pub heading: Option<String>,
    /// value: the X typed so far, one string per token; `null` for other items.
    pub entry: Option<Vec<String>>,
    /// The index of the token under the cursor in `entry`.
    pub entry_cursor: u32,
    /// The result's own value (`dy/dx=4`).
    pub value: Option<String>,
    /// For ∫f(x)dx: the x interval shaded under the traced function.
    pub shade_from: Option<f64>,
    pub shade_to: Option<f64>,
}

/// The FORMAT grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum GridStyle {
    Off,
    /// A dot at each tick crossing.
    Dot,
    /// Lines through the ticks.
    Line,
}

/// The free-moving cursor and, during ZBox, the first corner once marked.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct FreePoint {
    pub x: f64,
    pub y: f64,
    /// `X=1.5`.
    pub x_text: String,
    pub y_text: String,
    pub corner_x: Option<f64>,
    pub corner_y: Option<f64>,
}

/// The graph screen: the window's ranges, the sampled functions, tick positions, the FORMAT settings, the trace or
/// free cursor and a CALC item.
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
    /// Whether the free cursor shows its coordinates.
    pub coordinates: bool,
    pub grid: GridStyle,
    pub axes: bool,
    /// Whether the axes are labeled x and y.
    pub labels: bool,
    /// Whether the trace shows the traced function's equation.
    pub expression: bool,
    pub trace: Option<Trace>,
    pub free: Option<FreePoint>,
    pub calc: Option<CalcPanel>,
}

/// A menu item and the key that picks it (`1`, `0`, `A`).
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct MenuItem {
    pub key: String,
    pub label: String,
}

/// A menu: its tab titles (one for ZOOM), the shown tab, its items and the
/// highlighted one.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct MenuScreen {
    pub tabs: Vec<String>,
    pub tab: u32,
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
        entry: view.editing.as_ref().map(|(tokens, _)| texts(tokens)),
        entry_cursor: view
            .editing
            .as_ref()
            .map_or(0, |(_, cursor)| index(*cursor)),
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
        coordinates: view.format.coordinates,
        grid: match view.format.grid {
            calc_core::graph_format::Grid::Off => GridStyle::Off,
            calc_core::graph_format::Grid::Dot => GridStyle::Dot,
            calc_core::graph_format::Grid::Line => GridStyle::Line,
        },
        axes: view.format.axes,
        labels: view.format.labels,
        expression: view.format.expression,
        trace,
        free: view.free.as_ref().map(|free| FreePoint {
            x: free.x,
            y: free.y,
            x_text: free.x_text.clone(),
            y_text: free.y_text.clone(),
            corner_x: free.corner.map(|(x, _)| x),
            corner_y: free.corner.map(|(_, y)| y),
        }),
        calc: view.calc.as_ref().map(calc_panel),
    }
}

fn calc_panel(calc: &calc_core::screen::CalcView) -> CalcPanel {
    let (entry, entry_cursor) = match &calc.entry {
        Some((tokens, cursor)) => (Some(texts(tokens)), index(*cursor)),
        None => (None, 0),
    };
    CalcPanel {
        heading: calc.heading.map(str::to_owned),
        entry,
        entry_cursor,
        value: calc.value.clone(),
        shade_from: calc.shade.map(|(from, _)| from),
        shade_to: calc.shade.map(|(_, to)| to),
    }
}

pub fn menu_screen(view: &calc_core::MenuView) -> MenuScreen {
    let items = view.items.iter().map(|item| MenuItem {
        key: item.key.to_string(),
        label: item.label.to_owned(),
    });
    MenuScreen {
        tabs: view.tabs.iter().map(|&tab| tab.to_owned()).collect(),
        tab: index(view.tab),
        items: items.collect(),
        highlighted: index(view.highlighted),
    }
}
