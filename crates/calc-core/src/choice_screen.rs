//! Screens of option lines, MODE and FORMAT: a line per setting, its
//! options across, ENTER choosing the highlighted one.

use std::marker::PhantomData;

/// A line of an options screen and the settings it reads and writes.
pub trait ChoiceRow: Copy + Eq + 'static {
    type Settings;
    /// The lines, top to bottom.
    const ALL: &'static [Self];

    fn options(self) -> &'static [&'static str];
    /// The index of the option `settings` has chosen on this line.
    fn selected(self, settings: &Self::Settings) -> usize;
    fn choose(self, option: usize, settings: &mut Self::Settings);
}

/// The highlighted option of an options screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChoiceCursor<R> {
    row: usize,
    column: usize,
    rows: PhantomData<R>,
}

impl<R: ChoiceRow> ChoiceCursor<R> {
    /// On the first line's chosen option.
    pub fn new(settings: &R::Settings) -> Self {
        Self::on_row(0, settings)
    }

    fn on_row(row: usize, settings: &R::Settings) -> Self {
        ChoiceCursor {
            row,
            column: R::ALL[row].selected(settings),
            rows: PhantomData,
        }
    }

    pub fn row(self) -> R {
        R::ALL[self.row]
    }

    pub fn column(self) -> usize {
        self.column
    }

    /// The line above or below (wrapping), on that line's chosen option.
    pub fn step_row(self, down: bool, settings: &R::Settings) -> Self {
        let count = R::ALL.len();
        let row = if down {
            (self.row + 1) % count
        } else {
            (self.row + count - 1) % count
        };
        Self::on_row(row, settings)
    }

    /// The option to the left or right on this line, stopping at its ends.
    pub fn step_column(self, right: bool) -> Self {
        let last = self.row().options().len() - 1;
        let column = if right {
            (self.column + 1).min(last)
        } else {
            self.column.saturating_sub(1)
        };
        ChoiceCursor { column, ..self }
    }

    /// ENTER: makes the highlighted option the line's setting.
    pub fn choose(self, settings: &mut R::Settings) {
        self.row().choose(self.column, settings);
    }
}
