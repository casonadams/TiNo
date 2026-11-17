use super::{Calculator, HOME, View};
use crate::{
    format::format_real,
    free_cursor::FreeCursor,
    keymap::{Action, Command, Edit},
    screen::{FreeView, GraphView},
};

/// ZBox under way: the first corner once marked, and the cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct BoxMarking {
    pub(super) corner: Option<(f64, f64)>,
    pub(super) cursor: FreeCursor,
}

/// The pixel step an arrow key moves the free cursor.
pub(super) fn arrow_step(action: Action) -> Option<(isize, isize)> {
    match action {
        Action::Edit(Edit::Left) => Some((-1, 0)),
        Action::Edit(Edit::Right) => Some((1, 0)),
        Action::Command(Command::Up) => Some((0, 1)),
        Action::Command(Command::Down) => Some((0, -1)),
        _ => None,
    }
}

impl Calculator {
    /// The free cursor on the graph: arrows move it a pixel; CLEAR leaves
    /// for the home screen.
    pub(super) fn perform_free(&mut self, cursor: FreeCursor, action: Action) -> View {
        match (action, arrow_step(action)) {
            (Action::Edit(Edit::Clear), _) => HOME,
            (_, Some(step)) => View::FreeGraph(cursor.stepped(step)),
            _ => View::FreeGraph(cursor),
        }
    }

    /// ZBox: arrows move the cursor; ENTER marks the first corner, then the
    /// opposite one, which makes the box the window. CLEAR cancels.
    pub(super) fn perform_zbox(&mut self, marking: BoxMarking, action: Action) -> View {
        let point = marking.cursor.point(&self.window);
        match (action, marking.corner, arrow_step(action)) {
            (Action::Edit(Edit::Clear), ..) => self.graph(false),
            (_, _, Some(step)) => View::ZBox(BoxMarking {
                cursor: marking.cursor.stepped(step),
                ..marking
            }),
            (Action::Command(Command::Enter), None, _) => View::ZBox(BoxMarking {
                corner: Some(point),
                ..marking
            }),
            (Action::Command(Command::Enter), Some(corner), _) => {
                match self.window.boxed(corner, point) {
                    Some(window) => {
                        self.window = window;
                        self.graph(false)
                    }
                    None => View::ZBox(marking),
                }
            }
            _ => View::ZBox(marking),
        }
    }

    /// The graph with the free cursor, and the ZBox corner when one is marked.
    pub(super) fn free_graph_view(
        &self,
        cursor: FreeCursor,
        corner: Option<(f64, f64)>,
    ) -> GraphView {
        let (x, y) = cursor.point(&self.window);
        let free = FreeView {
            x,
            y,
            x_text: format!("X={}", format_real(x, self.settings)),
            y_text: format!("Y={}", format_real(y, self.settings)),
            corner,
        };
        GraphView {
            free: Some(free),
            ..self.graph_view(None)
        }
    }
}
