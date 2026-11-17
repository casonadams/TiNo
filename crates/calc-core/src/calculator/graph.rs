use super::{
    Calculator, HOME, View, error_view,
    free::{BoxMarking, arrow_step},
};
use crate::{
    error_screen::ErrorOrigin,
    free_cursor::FreeCursor,
    keymap::{Action, Command, Edit},
    plot::{plot, y_extent},
    trace::TraceCursor,
    window::Window,
    zoom::{self, ZoomContext, ZoomItem},
};

/// Widths of x range a gesture zoom may reach; beyond them it is ignored.
const SMALLEST_RANGE: f64 = 1e-9;
const LARGEST_RANGE: f64 = 1e90;

impl Calculator {
    /// The graph screen, tracing from the middle when `trace`; the error
    /// screen when the window or a graphed function is invalid.
    pub(super) fn graph(&mut self, trace: bool) -> View {
        if let Err(error) = self.window.check() {
            return error_view(error, ErrorOrigin::Window);
        }
        if let Err((slot, error)) = self.functions.parse_graphed() {
            return error_view(error, ErrorOrigin::Function(slot));
        }
        let graphed: Vec<usize> = self.functions.graphed().collect();
        View::Graph(
            trace
                .then(|| TraceCursor::start(&graphed, &self.window))
                .flatten(),
        )
    }

    pub(super) fn perform_graph(&mut self, trace: Option<TraceCursor>, action: Action) -> View {
        let Some(cursor) = trace else {
            return match (action, arrow_step(action)) {
                (Action::Edit(Edit::Clear), _) => HOME,
                (_, Some(step)) => View::FreeGraph(FreeCursor::center().stepped(step)),
                _ => View::Graph(None),
            };
        };
        let cursor = match action {
            Action::Edit(Edit::Clear) => return HOME,
            Action::Edit(Edit::Left) => self.step_trace(cursor, -1.0),
            Action::Edit(Edit::Right) => self.step_trace(cursor, 1.0),
            Action::Command(Command::Up) => cursor.switched(&self.graphed(), false),
            Action::Command(Command::Down) => cursor.switched(&self.graphed(), true),
            Action::Command(Command::Enter) => {
                self.window = self.window.centered_on_x(cursor.x);
                cursor
            }
            _ => cursor,
        };
        View::Graph(Some(cursor))
    }

    pub(super) fn step_trace(&mut self, cursor: TraceCursor, direction: f64) -> TraceCursor {
        let (cursor, window) = cursor.stepped(direction * self.window.x_res, self.window);
        self.window = window;
        cursor
    }

    pub(super) fn graphed(&self) -> Vec<usize> {
        self.functions.graphed().collect()
    }

    /// A ZOOM menu item: the new window, then the graph; ZBox goes to the
    /// graph to mark its corners.
    pub(super) fn zoom(&mut self, item: ZoomItem) -> View {
        let fit = match item {
            ZoomItem::ZoomFit => match self.functions.parse_graphed() {
                Ok(functions) => y_extent(&plot(&functions, &self.context())),
                Err((slot, error)) => return error_view(error, ErrorOrigin::Function(slot)),
            },
            _ => None,
        };
        let context = ZoomContext {
            angle: self.settings.angle,
            fit,
        };
        let Some(window) = zoom::apply(item, self.window, context) else {
            return match self.graph(false) {
                View::Graph(_) => View::ZBox(BoxMarking {
                    corner: None,
                    cursor: FreeCursor::center(),
                }),
                other => other,
            };
        };
        self.window = window;
        self.graph(false)
    }

    /// Drags the graph by fractions of its width and height (positive `dx`
    /// shows more to the right, positive `dy` more above). Only on the graph.
    pub fn pan_graph(&mut self, dx: f64, dy: f64) {
        if matches!(self.view, View::Graph(_)) && dx.is_finite() && dy.is_finite() {
            self.window = self.window.panned(dx, dy);
        }
    }

    /// Zooms the graph in by `factor` (out below 1) about `anchor`, given as
    /// fractions of the width from the left and the height from the bottom.
    /// Only on the graph, and only to windows of a usable size.
    pub fn zoom_graph(&mut self, factor: f64, anchor: (f64, f64)) {
        if !matches!(self.view, View::Graph(_)) || !(factor.is_finite() && factor > 0.0) {
            return;
        }
        let zoomed = self.window.zoomed(factor, anchor);
        if usable(&zoomed) {
            self.window = zoomed;
        }
    }

    /// Puts the trace cursor at the column nearest `fraction` of the graph's
    /// width from the left, starting a trace when none is on. Only on the graph.
    pub fn trace_at(&mut self, fraction: f64) {
        if !matches!(self.view, View::Graph(_)) || !fraction.is_finite() {
            return;
        }
        let graphed = self.graphed();
        if let View::Graph(trace) = &mut self.view {
            let cursor = trace.or_else(|| TraceCursor::start(&graphed, &self.window));
            *trace = cursor.map(|c| c.at_fraction(fraction, &self.window));
        }
    }
}

fn usable(window: &Window) -> bool {
    let sizes = [window.x_max - window.x_min, window.y_max - window.y_min];
    window.check().is_ok()
        && sizes
            .iter()
            .all(|s| (SMALLEST_RANGE..LARGEST_RANGE).contains(s))
}
