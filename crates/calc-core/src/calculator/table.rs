use super::{Calculator, HOME, View, error_view};
use crate::{
    error_screen::ErrorOrigin,
    expr::Expr,
    field_editor::FieldCursor,
    format::format_real,
    functions::Functions,
    keymap::{Action, Command, Edit},
    screen::TableView,
    table::{TABLE_FUNCTION_COLUMNS, TABLE_ROWS, TableCursor, TableField},
};

impl Calculator {
    /// The TBLSET cursor on `field`, showing its value.
    pub(super) fn table_setup_cursor(&self, field: TableField) -> FieldCursor<TableField> {
        FieldCursor::new(field, field.get(&self.table))
    }

    pub(super) fn perform_table_setup(
        &mut self,
        mut cursor: FieldCursor<TableField>,
        action: Action,
    ) -> View {
        let Some(down) = cursor.perform(action) else {
            return View::TableSetup(cursor);
        };
        match self.commit_table_setup(&cursor) {
            Ok(()) => View::TableSetup(self.table_setup_cursor(cursor.neighbor(down))),
            Err(error) => error,
        }
    }

    /// Writes the TBLSET field's entry into the table settings; the error
    /// screen when it does not evaluate or is a step of 0.
    pub(super) fn commit_table_setup(
        &mut self,
        cursor: &FieldCursor<TableField>,
    ) -> Result<(), View> {
        let value = cursor.value(&self.context());
        let written =
            value.and_then(|value| value.map_or(Ok(()), |v| cursor.field.set(&mut self.table, v)));
        written.map_err(|error| {
            let origin = ErrorOrigin::TableField(cursor.field, cursor.editor.tokens().to_vec());
            error_view(error, origin)
        })
    }

    /// The table, from TblStart; the error screen when a graphed function
    /// does not parse.
    pub(super) fn table(&mut self) -> View {
        match self.functions.parse_graphed() {
            Ok(_) => View::Table(TableCursor::default()),
            Err((slot, error)) => error_view(error, ErrorOrigin::Function(slot)),
        }
    }

    pub(super) fn perform_table(&mut self, cursor: TableCursor, action: Action) -> View {
        let functions = self.functions.graphed().count();
        View::Table(match action {
            Action::Edit(Edit::Clear) => return HOME,
            Action::Command(Command::Up) => cursor.step_row(false),
            Action::Command(Command::Down) => cursor.step_row(true),
            Action::Edit(Edit::Left) => cursor.step_column(false, functions),
            Action::Edit(Edit::Right) => cursor.step_column(true, functions),
            _ => cursor,
        })
    }

    /// Scrolls the table by `rows` (down when positive). Only on the table.
    pub fn scroll_table(&mut self, rows: i64) {
        if let View::Table(cursor) = &mut self.view {
            *cursor = cursor.scrolled(rows);
        }
    }

    pub(super) fn table_view(&self, cursor: TableCursor) -> TableView {
        let functions = self.functions.parse_graphed().unwrap_or_default();
        let shown: Vec<&(usize, Expr)> = functions
            .iter()
            .skip(cursor.first_function)
            .take(TABLE_FUNCTION_COLUMNS)
            .collect();
        let context = self.context();
        let cell = |x: f64, expr: &Expr| {
            context
                .y_at(expr, x)
                .map_or_else(|| "ERROR".to_owned(), |y| format_real(y, self.settings))
        };
        let rows = (0..TABLE_ROWS).map(|row| {
            let x = self.table.x(cursor.first_row + row as i64);
            let mut cells = vec![format_real(x, self.settings)];
            cells.extend(shown.iter().map(|(_, expr)| cell(x, expr)));
            cells
        });
        let x = self.table.x(cursor.selected_row());
        let footer = match cursor.column.checked_sub(1).and_then(|f| functions.get(f)) {
            Some((slot, expr)) => format!("{}={}", Functions::label(*slot), cell(x, expr)),
            None => format!("X={}", format_real(x, self.settings)),
        };
        let mut headers = vec!["X".to_owned()];
        headers.extend(shown.iter().map(|(slot, _)| Functions::label(*slot)));
        TableView {
            headers,
            rows: rows.collect(),
            selected_row: cursor.row,
            selected_column: cursor.column.saturating_sub(cursor.first_function),
            footer,
        }
    }
}
