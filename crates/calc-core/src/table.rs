use serde::{Deserialize, Serialize};

use crate::{
    error::{CalcError, ErrorKind},
    field_editor::Field,
};

/// Rows the table shows at once.
pub const TABLE_ROWS: usize = 8;
/// Function columns the table shows at once, beside X.
pub const TABLE_FUNCTION_COLUMNS: usize = 3;

/// TBLSET: the table's first X and the step between rows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TableSettings {
    pub start: f64,
    pub step: f64,
}

impl Default for TableSettings {
    fn default() -> Self {
        TableSettings {
            start: 0.0,
            step: 1.0,
        }
    }
}

impl TableSettings {
    /// The X of row `row`, counted from `start` (negative rows lie above it).
    pub fn x(&self, row: i64) -> f64 {
        self.start + row as f64 * self.step
    }
}

/// A line of the TBLSET screen, also read as a variable from VARS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TableField {
    Start,
    Step,
}

impl Field for TableField {
    const TITLE: &'static str = "TABLE SETUP";
    const ALL: &'static [Self] = &[TableField::Start, TableField::Step];

    fn label(self) -> &'static str {
        match self {
            TableField::Start => "TblStart",
            TableField::Step => "ΔTbl",
        }
    }
}

impl TableField {
    pub fn get(self, settings: &TableSettings) -> f64 {
        match self {
            TableField::Start => settings.start,
            TableField::Step => settings.step,
        }
    }

    /// Sets the field; a step of 0 is a domain error, as rows would not advance.
    pub fn set(self, settings: &mut TableSettings, value: f64) -> Result<(), CalcError> {
        match self {
            TableField::Start => settings.start = value,
            TableField::Step if value == 0.0 => return Err(CalcError::new(ErrorKind::Domain)),
            TableField::Step => settings.step = value,
        }
        Ok(())
    }
}

/// The table's scroll position and selected cell. Rows count from
/// TblStart; column 0 is X, column `n` the `n`th graphed function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableCursor {
    /// The row shown at the top.
    pub first_row: i64,
    /// The selected row, counted from the top one shown.
    pub row: usize,
    pub column: usize,
    /// The first function column shown.
    pub first_function: usize,
}

impl TableCursor {
    /// ▼ (or ▲): the next row, scrolling when the selection is at the edge.
    pub fn step_row(self, down: bool) -> Self {
        match (down, self.row) {
            (true, row) if row + 1 < TABLE_ROWS => TableCursor {
                row: row + 1,
                ..self
            },
            (true, _) => self.scrolled(1),
            (false, 0) => self.scrolled(-1),
            (false, row) => TableCursor {
                row: row - 1,
                ..self
            },
        }
    }

    /// ▶ (or ◀): the next column among X and `functions` columns, scrolling
    /// the function columns to keep it shown.
    pub fn step_column(self, right: bool, functions: usize) -> Self {
        let column = if right {
            (self.column + 1).min(functions)
        } else {
            self.column.saturating_sub(1)
        };
        let first_function = match column.checked_sub(1) {
            Some(function) if function < self.first_function => function,
            Some(function) if function >= self.first_function + TABLE_FUNCTION_COLUMNS => {
                function + 1 - TABLE_FUNCTION_COLUMNS
            }
            _ => self.first_function,
        };
        TableCursor {
            column,
            first_function,
            ..self
        }
    }

    /// The rows moved by `rows` (down when positive), the selection staying
    /// at the same place on screen.
    pub fn scrolled(self, rows: i64) -> Self {
        TableCursor {
            first_row: self.first_row.saturating_add(rows),
            ..self
        }
    }

    /// The selected row, counted from TblStart.
    pub fn selected_row(self) -> i64 {
        self.first_row + self.row as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_count_from_tbl_start_by_the_step() {
        let settings = TableSettings {
            start: -2.0,
            step: 0.5,
        };
        assert_eq!(
            (settings.x(0), settings.x(3), settings.x(-1)),
            (-2.0, -0.5, -2.5)
        );
    }

    #[test]
    fn a_zero_step_is_a_domain_error() {
        let mut settings = TableSettings::default();
        let error = TableField::Step.set(&mut settings, 0.0).unwrap_err();
        assert_eq!((error.kind, settings.step), (ErrorKind::Domain, 1.0));
    }

    #[test]
    fn moving_past_the_last_shown_row_scrolls() {
        let cursor = (0..TABLE_ROWS).fold(TableCursor::default(), |c, _| c.step_row(true));
        assert_eq!(
            (cursor.first_row, cursor.row, cursor.selected_row()),
            (1, TABLE_ROWS - 1, 8)
        );
        let above = TableCursor::default().step_row(false);
        assert_eq!((above.first_row, above.selected_row()), (-1, -1));
    }

    #[test]
    fn function_columns_scroll_to_keep_the_selection_shown() {
        let right = |c: TableCursor| c.step_column(true, 5);
        let cursor = (0..4).fold(TableCursor::default(), |c, _| right(c));
        assert_eq!((cursor.column, cursor.first_function), (4, 1));
        let cursor = (0..3).fold(cursor, |c, _| c.step_column(false, 5));
        assert_eq!((cursor.column, cursor.first_function), (1, 0));
        assert_eq!(
            right(right(TableCursor::default()))
                .step_column(true, 2)
                .column,
            2
        );
    }
}
