//! The editors laid out as a grid of cells: STAT EDIT's lists side by side,
//! and MATRIX EDIT's rows and columns.

use crate::{editor::Editor, token::MatrixName};

/// What a grid editor edits: the lists, a list per column, or a matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grid {
    Lists,
    Matrix(MatrixName),
}

/// Columns shown at once.
pub const EDITOR_COLUMNS: usize = 3;
/// Rows shown at once.
pub const EDITOR_ROWS: usize = 7;

/// A grid editor's selection: a column and a row, the first column and row
/// shown, and what is typed into the selected cell.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GridEditor {
    pub column: usize,
    pub row: usize,
    pub first_column: usize,
    pub first_row: usize,
    pub entry: Option<Editor>,
}

impl GridEditor {
    /// One cell right, left, down or up (`dx`, `dy` of 1 or -1), up to the
    /// last column and that column's last row; the shown columns and rows
    /// follow.
    pub fn stepped(
        self,
        (dx, dy): (isize, isize),
        (last_column, last_row): (usize, impl Fn(usize) -> usize),
    ) -> Self {
        let column = self.column.saturating_add_signed(dx).min(last_column);
        let row = self.row.saturating_add_signed(dy).min(last_row(column));
        GridEditor {
            column,
            row,
            first_column: follow(self.first_column, column, EDITOR_COLUMNS),
            first_row: follow(self.first_row, row, EDITOR_ROWS),
            entry: None,
        }
    }
}

/// The first of `shown` places, moved as little as keeps `selected` among them.
fn follow(first: usize, selected: usize, shown: usize) -> usize {
    first
        .min(selected)
        .max((selected + 1).saturating_sub(shown))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stop_at_the_last_cell_and_scroll_to_stay_shown() {
        let bounds = || (5, |column: usize| if column == 0 { 9 } else { 0 });
        let down = (0..12).fold(GridEditor::default(), |editor, _| {
            editor.stepped((0, 1), bounds())
        });
        assert_eq!((down.row, down.first_row), (9, 3));
        let right = (0..9).fold(down, |editor, _| editor.stepped((1, 0), bounds()));
        assert_eq!((right.column, right.row, right.first_column), (5, 0, 3));
        let back = (0..9).fold(right, |editor, _| editor.stepped((-1, 0), bounds()));
        assert_eq!((back.column, back.first_column), (0, 0));
    }
}
