use crate::window::{GRAPH_COLUMNS, GRAPH_ROWS, Window};

/// The graph's free-moving cursor: a pixel of the graph area, column 0 at
/// the left and row 0 at the bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FreeCursor {
    pub column: usize,
    pub row: usize,
}

impl FreeCursor {
    /// At the middle of the graph area.
    pub fn center() -> Self {
        FreeCursor {
            column: GRAPH_COLUMNS / 2,
            row: GRAPH_ROWS / 2,
        }
    }

    /// One pixel right, left, up or down (`dx`, `dy` of 1 or -1), stopping
    /// at the edges.
    pub fn stepped(self, (dx, dy): (isize, isize)) -> Self {
        FreeCursor {
            column: self.column.saturating_add_signed(dx).min(GRAPH_COLUMNS - 1),
            row: self.row.saturating_add_signed(dy).min(GRAPH_ROWS - 1),
        }
    }

    /// The point under the cursor in `window`.
    pub fn point(self, window: &Window) -> (f64, f64) {
        (window.column_x(self.column), window.row_y(self.row))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stop_at_the_edges_of_the_graph() {
        let corner = FreeCursor { column: 0, row: 0 }.stepped((-1, -1));
        assert_eq!(corner, FreeCursor { column: 0, row: 0 });
        let top = FreeCursor {
            column: GRAPH_COLUMNS - 1,
            row: GRAPH_ROWS - 1,
        };
        assert_eq!(top.stepped((1, 1)), top);
        assert_eq!(FreeCursor::center().point(&Window::standard()), (0.0, 0.0));
    }
}
