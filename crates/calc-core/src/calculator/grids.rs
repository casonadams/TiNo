use super::{Calculator, View, error_view, free::arrow_step};
use crate::{
    editor::Editor,
    error::CalcError,
    error_screen::ErrorOrigin,
    eval::{LARGEST_DIMENSION, Matrix},
    format::format_real,
    grid_editor::{EDITOR_COLUMNS, EDITOR_ROWS, Grid, GridEditor},
    keymap::{Action, Command, Edit},
    parse::parse,
    screen::TableView,
    token::{ListName, MatrixName, Token},
};

impl Calculator {
    /// STAT EDIT and MATRIX EDIT: arrows move between cells (ending what is
    /// typed), typing edits the selected cell, ENTER stores it and moves
    /// down, CLEAR empties what is typed; in STAT EDIT, DEL removes the
    /// element.
    pub(super) fn perform_grid(
        &mut self,
        (grid, mut editor): (Grid, GridEditor),
        action: Action,
    ) -> View {
        if let Some(step) = arrow_step(action) {
            return View::Grid(grid, self.step_grid((grid, editor), (step.0, -step.1)));
        }
        match action {
            Action::Command(Command::Enter) => return self.store_cell(grid, editor),
            Action::Insert(token) => editor
                .entry
                .get_or_insert_with(Editor::default)
                .type_token(token),
            Action::Edit(Edit::Delete) if editor.entry.is_none() => self.delete_cell(grid, &editor),
            Action::Edit(edit) => {
                if let Some(entry) = editor.entry.as_mut() {
                    entry.apply(edit);
                }
            }
            Action::Command(_) => {}
        }
        View::Grid(grid, editor)
    }

    fn step_grid(&self, (grid, editor): (Grid, GridEditor), step: (isize, isize)) -> GridEditor {
        let last_rows = self.last_rows(grid);
        editor.stepped(step, (last_rows.len() - 1, |column| last_rows[column]))
    }

    /// Each column's last row the cursor reaches: a list's row after its
    /// last element; a matrix's row and column after its last, to grow it.
    fn last_rows(&self, grid: Grid) -> Vec<usize> {
        match grid {
            Grid::Lists => ListName::ALL
                .iter()
                .map(|&name| self.memory.list(name).len())
                .collect(),
            Grid::Matrix(name) => {
                let (rows, columns) = size(self.memory.matrix(name));
                vec![rows.min(LARGEST_DIMENSION - 1); columns.min(LARGEST_DIMENSION - 1) + 1]
            }
        }
    }

    /// ENTER: the typed value into the selected cell (growing the list or
    /// matrix past its end), then the row below.
    fn store_cell(&mut self, grid: Grid, editor: GridEditor) -> View {
        let Some(entry) = &editor.entry else {
            return View::Grid(grid, self.step_grid((grid, editor), (0, 1)));
        };
        match self.cell_value(entry.tokens()) {
            Ok(value) => {
                self.set_cell((grid, editor.column, editor.row), value);
                View::Grid(grid, self.step_grid((grid, editor), (0, 1)))
            }
            Err(error) => error_view(error, ErrorOrigin::Grid(grid, editor)),
        }
    }

    fn cell_value(&self, tokens: &[Token]) -> Result<f64, CalcError> {
        self.context().evaluate_real(&parse(tokens)?.expr)
    }

    fn set_cell(&mut self, (grid, column, row): (Grid, usize, usize), value: f64) {
        match grid {
            Grid::Lists => {
                let name = ListName::ALL[column];
                let mut list = self.memory.list(name).to_vec();
                list.resize(list.len().max(row + 1), 0.0);
                list[row] = value;
                self.memory.set_list(name, list);
            }
            Grid::Matrix(name) => {
                let mut matrix = grown(self.memory.matrix(name).clone(), (row + 1, column + 1));
                matrix[row][column] = value;
                self.memory.set_matrix(name, matrix);
            }
        }
    }

    fn delete_cell(&mut self, grid: Grid, editor: &GridEditor) {
        let Grid::Lists = grid else {
            return;
        };
        let name = ListName::ALL[editor.column];
        let mut list = self.memory.list(name).to_vec();
        if editor.row < list.len() {
            list.remove(editor.row);
            self.memory.set_list(name, list);
        }
    }

    fn cell(&self, grid: Grid, (column, row): (usize, usize)) -> Option<f64> {
        match grid {
            Grid::Lists => self.memory.list(ListName::ALL[column]).get(row).copied(),
            Grid::Matrix(name) => self.memory.matrix(name).get(row)?.get(column).copied(),
        }
    }

    /// The grid shown as a table: the lists' names or the matrix's column
    /// numbers, the cells, the selected cell, and `L₁(3)=` or `[A](2,1)=`
    /// with its value or what is typed.
    pub(super) fn grid_view(&self, grid: Grid, editor: &GridEditor) -> TableView {
        let shown = EDITOR_COLUMNS.min(self.last_rows(grid).len() - editor.first_column);
        let columns = editor.first_column..editor.first_column + shown;
        let text = |column: usize, row: usize| {
            let value = self.cell(grid, (column, row));
            value.map_or_else(String::new, |x| format_real(x, self.settings))
        };
        let rows = (editor.first_row..editor.first_row + EDITOR_ROWS)
            .map(|row| columns.clone().map(|column| text(column, row)).collect());
        let label = cell_label(grid, (editor.column, editor.row));
        TableView {
            headers: columns.clone().map(|column| header(grid, column)).collect(),
            rows: rows.collect(),
            selected_row: editor.row - editor.first_row,
            selected_column: editor.column - editor.first_column,
            footer: match editor.entry {
                Some(_) => label,
                None => label + &text(editor.column, editor.row),
            },
            editing: editor
                .entry
                .as_ref()
                .map(|entry| (entry.tokens().to_vec(), entry.cursor())),
        }
    }
}

fn size(matrix: &Matrix) -> (usize, usize) {
    (matrix.len(), matrix.first().map_or(0, Vec::len))
}

/// `matrix` with zero rows and columns added to reach `rows` by `columns`.
fn grown(mut matrix: Matrix, (rows, columns): (usize, usize)) -> Matrix {
    let columns = columns.max(size(&matrix).1);
    matrix.resize(rows.max(matrix.len()), Vec::new());
    for row in &mut matrix {
        row.resize(columns, 0.0);
    }
    matrix
}

fn header(grid: Grid, column: usize) -> String {
    match grid {
        Grid::Lists => Token::List(ListName::ALL[column]).text().to_owned(),
        Grid::Matrix(_) => (column + 1).to_string(),
    }
}

fn cell_label(grid: Grid, (column, row): (usize, usize)) -> String {
    match grid {
        Grid::Lists => format!(
            "{}({})=",
            Token::List(ListName::ALL[column]).text(),
            row + 1
        ),
        Grid::Matrix(name) => format!("{}({},{})=", MatrixName::text(name), row + 1, column + 1),
    }
}
