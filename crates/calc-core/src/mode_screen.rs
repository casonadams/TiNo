use crate::settings::{Angle, Decimals, FixedPlaces, Notation, Settings};

/// A line of the MODE screen: one setting and its choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeRow {
    Notation,
    Decimals,
    Angle,
}

impl ModeRow {
    pub const ALL: [ModeRow; 3] = [ModeRow::Notation, ModeRow::Decimals, ModeRow::Angle];

    pub fn options(self) -> &'static [&'static str] {
        match self {
            ModeRow::Notation => &["NORMAL", "SCI", "ENG"],
            ModeRow::Decimals => &["FLOAT", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
            ModeRow::Angle => &["RADIAN", "DEGREE"],
        }
    }

    /// The index of the option `settings` has chosen on this row.
    pub fn selected(self, settings: Settings) -> usize {
        match self {
            ModeRow::Notation => settings.notation as usize,
            ModeRow::Decimals => match settings.decimals {
                Decimals::Float => 0,
                Decimals::Fix(places) => places.get() + 1,
            },
            ModeRow::Angle => settings.angle as usize,
        }
    }

    fn choose(self, option: usize, settings: &mut Settings) {
        match self {
            ModeRow::Notation => {
                settings.notation = [
                    Notation::Normal,
                    Notation::Scientific,
                    Notation::Engineering,
                ][option];
            }
            ModeRow::Decimals => {
                settings.decimals = option
                    .checked_sub(1)
                    .and_then(|places| FixedPlaces::new(u8::try_from(places).ok()?))
                    .map_or(Decimals::Float, Decimals::Fix);
            }
            ModeRow::Angle => settings.angle = [Angle::Radian, Angle::Degree][option],
        }
    }
}

/// The highlighted option of the MODE screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeCursor {
    row: usize,
    column: usize,
}

impl ModeCursor {
    /// On the first row's chosen option.
    pub fn new(settings: Settings) -> Self {
        Self::on_row(0, settings)
    }

    fn on_row(row: usize, settings: Settings) -> Self {
        ModeCursor {
            row,
            column: ModeRow::ALL[row].selected(settings),
        }
    }

    pub fn row(self) -> ModeRow {
        ModeRow::ALL[self.row]
    }

    pub fn column(self) -> usize {
        self.column
    }

    /// The row above or below (wrapping), on that row's chosen option.
    pub fn step_row(self, down: bool, settings: Settings) -> Self {
        let count = ModeRow::ALL.len();
        let row = if down {
            (self.row + 1) % count
        } else {
            (self.row + count - 1) % count
        };
        Self::on_row(row, settings)
    }

    /// The option to the left or right on this row, stopping at its ends.
    pub fn step_column(self, right: bool) -> Self {
        let last = self.row().options().len() - 1;
        let column = if right {
            (self.column + 1).min(last)
        } else {
            self.column.saturating_sub(1)
        };
        ModeCursor { column, ..self }
    }

    /// ENTER: makes the highlighted option the row's setting.
    pub fn choose(self, settings: &mut Settings) {
        self.row().choose(self.column, settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choosing_options_changes_settings() {
        let mut settings = Settings::default();
        let cursor = ModeCursor::new(settings)
            .step_row(true, settings)
            .step_column(true)
            .step_column(true)
            .step_column(true);
        cursor.choose(&mut settings);
        assert_eq!(
            settings.decimals,
            Decimals::Fix(FixedPlaces::new(2).unwrap())
        );
        let cursor = cursor.step_row(true, settings).step_column(true);
        cursor.choose(&mut settings);
        assert_eq!(settings.angle, Angle::Degree);
    }

    #[test]
    fn moving_rows_lands_on_the_chosen_option_and_wraps() {
        let settings = Settings {
            angle: Angle::Degree,
            ..Settings::default()
        };
        let cursor = ModeCursor::new(settings).step_row(false, settings);
        assert_eq!((cursor.row(), cursor.column()), (ModeRow::Angle, 1));
    }

    #[test]
    fn columns_stop_at_the_ends() {
        let cursor = ModeCursor::new(Settings::default()).step_column(false);
        assert_eq!(cursor.column(), 0);
        let last = (0..5).fold(cursor, |c, _| c.step_column(true));
        assert_eq!(last.column(), 2);
    }
}
