//! STAT EDIT: the lists L₁–L₆ side by side, a cell selected.

use crate::{editor::Editor, token::ListName};

/// Lists shown at once.
pub const EDITOR_COLUMNS: usize = 3;
/// Rows shown at once.
pub const EDITOR_ROWS: usize = 7;

/// The list editor's selection: a list (column) and an element (row, where
/// the row after the last element adds one), the first column and row
/// shown, and what is typed into the cell.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ListEditor {
    pub column: usize,
    pub row: usize,
    pub first_column: usize,
    pub first_row: usize,
    pub entry: Option<Editor>,
}

impl ListEditor {
    pub fn list(&self) -> ListName {
        ListName::ALL[self.column]
    }

    /// One cell right, left, down or up (`dx`, `dy` of 1 or -1): across the
    /// six lists, and down to the row after a list's last element; the shown
    /// columns and rows follow. `length` is each list's length.
    pub fn stepped(self, (dx, dy): (isize, isize), length: impl Fn(ListName) -> usize) -> Self {
        let column = self
            .column
            .saturating_add_signed(dx)
            .min(ListName::ALL.len() - 1);
        let row = self
            .row
            .saturating_add_signed(dy)
            .min(length(ListName::ALL[column]));
        ListEditor {
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
    fn steps_stop_after_the_last_element_and_scroll_to_stay_shown() {
        let length = |name: ListName| if name == ListName::L1 { 9 } else { 0 };
        let down = (0..12).fold(ListEditor::default(), |editor, _| {
            editor.stepped((0, 1), length)
        });
        assert_eq!((down.row, down.first_row), (9, 3));
        let right = (0..4).fold(down, |editor, _| editor.stepped((1, 0), length));
        assert_eq!((right.column, right.row, right.first_column), (4, 0, 2));
        let back = (0..4).fold(right, |editor, _| editor.stepped((-1, 0), length));
        assert_eq!((back.column, back.first_column), (0, 0));
    }
}
