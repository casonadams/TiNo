//! FORMAT (2nd ZOOM): how the graph screen is drawn.

use serde::{Deserialize, Serialize};

use crate::choice_screen::{ChoiceCursor, ChoiceRow};

/// The graph's grid: none, a dot at each tick crossing, or lines through them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Grid {
    #[default]
    Off,
    Dot,
    Line,
}

/// The FORMAT settings: whether the free cursor shows its coordinates, the
/// grid, the axes, the axis labels, and the traced function's equation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphFormat {
    pub coordinates: bool,
    pub grid: Grid,
    pub axes: bool,
    pub labels: bool,
    pub expression: bool,
}

impl Default for GraphFormat {
    fn default() -> Self {
        GraphFormat {
            coordinates: true,
            grid: Grid::Off,
            axes: true,
            labels: false,
            expression: true,
        }
    }
}

/// A line of the FORMAT screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatRow {
    Coordinates,
    Grid,
    Axes,
    Labels,
    Expression,
}

/// The highlighted option of the FORMAT screen.
pub type FormatCursor = ChoiceCursor<FormatRow>;

impl ChoiceRow for FormatRow {
    type Settings = GraphFormat;
    const ALL: &'static [FormatRow] = &[
        FormatRow::Coordinates,
        FormatRow::Grid,
        FormatRow::Axes,
        FormatRow::Labels,
        FormatRow::Expression,
    ];

    fn options(self) -> &'static [&'static str] {
        match self {
            FormatRow::Coordinates => &["CoordOn", "CoordOff"],
            FormatRow::Grid => &["GridOff", "GridDot", "GridLine"],
            FormatRow::Axes => &["AxesOn", "AxesOff"],
            FormatRow::Labels => &["LabelOff", "LabelOn"],
            FormatRow::Expression => &["ExprOn", "ExprOff"],
        }
    }

    fn selected(self, format: &GraphFormat) -> usize {
        match self {
            FormatRow::Coordinates => usize::from(!format.coordinates),
            FormatRow::Grid => format.grid as usize,
            FormatRow::Axes => usize::from(!format.axes),
            FormatRow::Labels => usize::from(format.labels),
            FormatRow::Expression => usize::from(!format.expression),
        }
    }

    fn choose(self, option: usize, format: &mut GraphFormat) {
        match self {
            FormatRow::Coordinates => format.coordinates = option == 0,
            FormatRow::Grid => format.grid = [Grid::Off, Grid::Dot, Grid::Line][option],
            FormatRow::Axes => format.axes = option == 0,
            FormatRow::Labels => format.labels = option == 1,
            FormatRow::Expression => format.expression = option == 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_line_shows_and_sets_its_setting() {
        let mut format = GraphFormat::default();
        for row in FormatRow::ALL {
            let other = (row.selected(&format) + 1) % row.options().len();
            row.choose(other, &mut format);
            assert_eq!(row.selected(&format), other, "{row:?}");
        }
        let expected = GraphFormat {
            coordinates: false,
            grid: Grid::Dot,
            axes: false,
            labels: true,
            expression: false,
        };
        assert_eq!(format, expected);
    }
}
