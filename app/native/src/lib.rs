//! The calculator core as `App.tsx` imports it from `tino/native`: key ids
//! in, the screen out. Behavior lives in `calc-core`; this crate only
//! converts between it and JS values.

use nsh_core::prelude::*;

#[native_module("tino/native")]
fn native(registry: &mut Registry) {
    registry.class::<Calculator>();
}

/// What the cursor shows: the mode the next key press is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum CursorStyle {
    /// Typing replaces the token under the cursor.
    Overwrite,
    /// Typing inserts before the token under the cursor (2nd INS).
    Insert,
    /// The next key reads its 2nd legend.
    Second,
    /// The next key reads its ALPHA legend.
    Alpha,
    /// Every key reads its ALPHA legend until ALPHA is pressed (2nd A-LOCK).
    AlphaLock,
}

impl From<calc_core::CursorStyle> for CursorStyle {
    fn from(style: calc_core::CursorStyle) -> Self {
        match style {
            calc_core::CursorStyle::Overwrite => CursorStyle::Overwrite,
            calc_core::CursorStyle::Insert => CursorStyle::Insert,
            calc_core::CursorStyle::Second => CursorStyle::Second,
            calc_core::CursorStyle::Alpha => CursorStyle::Alpha,
            calc_core::CursorStyle::AlphaLock => CursorStyle::AlphaLock,
        }
    }
}

/// An evaluated entry on the home screen.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct HistoryLine {
    /// Unique for the calculator's lifetime: the line's key in lists.
    pub id: u32,
    /// The entry as TeX.
    pub latex: String,
    /// The result as the calculator formats it.
    pub result: String,
}

impl From<&calc_core::HistoryLine> for HistoryLine {
    fn from(line: &calc_core::HistoryLine) -> Self {
        HistoryLine {
            id: line.id,
            latex: line.output.latex.clone(),
            result: line.output.result.clone(),
        }
    }
}

/// What the calculator's screen shows.
#[derive(Debug, Clone, PartialEq, JsObject)]
pub struct Screen {
    /// Evaluated entries, oldest first.
    pub history: Vec<HistoryLine>,
    /// The entry line, one string per token.
    pub entry: Vec<String>,
    /// The index of the token under the cursor; `entry.length` after the last one.
    pub cursor: u32,
    /// What the cursor shows.
    pub cursor_style: CursorStyle,
    /// The error the last ENTER raised (`"ERR:SYNTAX"`), until the next key press.
    pub error: Option<String>,
}

impl From<calc_core::Screen<'_>> for Screen {
    fn from(screen: calc_core::Screen<'_>) -> Self {
        Screen {
            history: screen.history.iter().map(HistoryLine::from).collect(),
            entry: screen.entry.iter().map(|t| t.text().to_owned()).collect(),
            cursor: u32::try_from(screen.cursor).unwrap_or(u32::MAX),
            cursor_style: screen.cursor_style.into(),
            error: screen.error.map(|e| e.message().to_owned()),
        }
    }
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
