//! The calculator core as `App.tsx` imports it from `tino/native`: key ids
//! in, the screen out. Behavior lives in `calc-core`; this crate only
//! converts between it and JS values.

mod graphing;
mod screen;

pub use graphing::{
    Curve, FunctionRow, FunctionsScreen, GraphScreen, MenuItem, MenuScreen, Trace, WindowRow,
    WindowScreen,
};
use nsh_core::prelude::*;
pub use screen::{
    CursorStyle, ErrorScreen, HistoryLine, HomeScreen, ModeRow, ModeScreen, Screen, ScreenKind,
};

/// A point given as fractions of the graph: `x` of its width from the left,
/// `y` of its height from the bottom.
#[derive(Debug, Clone, Copy, PartialEq, JsObject)]
pub struct GraphPoint {
    pub x: f64,
    pub y: f64,
}

#[native_module("tino/native")]
fn native(registry: &mut Registry) {
    registry.class::<Calculator>();
}

/// A calculator whose state lives in Rust.
#[class]
#[derive(Default)]
pub struct Calculator {
    core: calc_core::Calculator,
}

#[class]
impl Calculator {
    /// A calculator with an empty entry line.
    pub fn new() -> Self {
        Self::default()
    }

    /// Presses the key with id `key` (`"7"`, `"sin"`, `"second"`, ...); the
    /// screen after it. Throws for an id no key has.
    pub fn press(&mut self, key: String) -> Result<Screen, String> {
        let key: calc_core::Key = key
            .parse()
            .map_err(|e: calc_core::UnknownKey| e.to_string())?;
        self.core.press(key);
        Ok(self.screen())
    }

    /// Drags the graph by fractions of its width and height (positive `dx`
    /// shows more to the right, positive `dy` more above); the screen after
    /// it. Only on the graph.
    pub fn pan_graph(&mut self, dx: f64, dy: f64) -> Screen {
        self.core.pan_graph(dx, dy);
        self.screen()
    }

    /// Zooms the graph in by `factor` (out below 1) about `anchor`; the
    /// screen after it. Only on the graph, and only to usable windows.
    pub fn zoom_graph(&mut self, factor: f64, anchor: GraphPoint) -> Screen {
        self.core.zoom_graph(factor, (anchor.x, anchor.y));
        self.screen()
    }

    /// Traces at the column nearest `fraction` of the graph's width from
    /// the left; the screen after it. Only on the graph.
    pub fn trace_at(&mut self, fraction: f64) -> Screen {
        self.core.trace_at(fraction);
        self.screen()
    }

    /// What the screen shows now.
    #[getter]
    pub fn screen(&self) -> Screen {
        self.core.screen().into()
    }
}
