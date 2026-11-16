use crate::window::{GRAPH_COLUMNS, Window};

/// The trace cursor: which function it rides and at what x.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraceCursor {
    pub slot: usize,
    pub x: f64,
}

impl TraceCursor {
    /// On the first graphed function, at the middle column; `None` when
    /// nothing is graphed.
    pub fn start(graphed: &[usize], window: &Window) -> Option<Self> {
        let slot = *graphed.first()?;
        Some(TraceCursor {
            slot,
            x: window.column_x(GRAPH_COLUMNS / 2),
        })
    }

    /// `columns` columns to the right (left when negative); a window that
    /// no longer shows the cursor recenters on it.
    pub fn stepped(self, columns: f64, window: Window) -> (Self, Window) {
        let x = self.x + columns * window.delta_x();
        let window = if (window.x_min..=window.x_max).contains(&x) {
            window
        } else {
            window.centered_on_x(x)
        };
        (TraceCursor { x, ..self }, window)
    }

    /// The next (or previous) graphed function, wrapping; this one when it
    /// is no longer graphed and none other is.
    pub fn switched(self, graphed: &[usize], down: bool) -> Self {
        let Some(current) = graphed.iter().position(|&s| s == self.slot) else {
            return TraceCursor {
                slot: graphed.first().copied().unwrap_or(self.slot),
                ..self
            };
        };
        let len = graphed.len();
        let next = if down {
            (current + 1) % len
        } else {
            (current + len - 1) % len
        };
        TraceCursor {
            slot: graphed[next],
            ..self
        }
    }

    /// At the column nearest `fraction` of the window's width from the left.
    pub fn at_fraction(self, fraction: f64, window: &Window) -> Self {
        let last = (GRAPH_COLUMNS - 1) as f64;
        let column = (fraction.clamp(0.0, 1.0) * last).round() as usize;
        TraceCursor {
            x: window.column_x(column),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_mid_screen_on_the_first_function() {
        let cursor = TraceCursor::start(&[2, 5], &Window::standard()).unwrap();
        assert_eq!((cursor.slot, cursor.x), (2, 0.0));
        assert_eq!(TraceCursor::start(&[], &Window::standard()), None);
    }

    #[test]
    fn stepping_past_the_edge_recenters_the_window() {
        let window = Window::standard();
        let cursor = TraceCursor { slot: 0, x: 10.0 };
        let (cursor, window) = cursor.stepped(1.0, window);
        assert!(cursor.x > 10.0);
        assert!((window.x_min + window.x_max) / 2.0 - cursor.x < 1e-12);
    }

    #[test]
    fn switching_wraps_through_graphed_functions() {
        let cursor = TraceCursor { slot: 5, x: 0.0 };
        assert_eq!(cursor.switched(&[2, 5], true).slot, 2);
        assert_eq!(cursor.switched(&[2, 5], false).slot, 2);
        assert_eq!(cursor.switched(&[3], true).slot, 3);
    }

    #[test]
    fn a_fraction_snaps_to_a_column() {
        let cursor = TraceCursor { slot: 0, x: 0.0 }.at_fraction(1.0, &Window::standard());
        assert_eq!(cursor.x, 10.0);
    }
}
