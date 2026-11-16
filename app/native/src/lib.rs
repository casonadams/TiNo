//! The calculator core as `App.tsx` imports it from `tino/native`: key ids
//! in, the screen out. Behavior lives in `calc-core`; this crate only
//! converts between it and JS values.

mod screen;

use nsh_core::prelude::*;
pub use screen::{
    CursorStyle, ErrorScreen, HistoryLine, HomeScreen, ModeRow, ModeScreen, Screen, ScreenKind,
};

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

    /// What the screen shows now.
    #[getter]
    pub fn screen(&self) -> Screen {
        self.core.screen().into()
    }
}
