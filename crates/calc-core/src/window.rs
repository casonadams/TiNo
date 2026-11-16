use crate::error::{CalcError, ErrorKind};

/// Columns of the graph area, in pixels of the color models' screen.
pub const GRAPH_COLUMNS: usize = 265;
/// Rows of the graph area.
pub const GRAPH_ROWS: usize = 165;
/// Tick marks drawn per axis at most; a finer scale draws none.
const MAX_TICKS: usize = 400;

/// The WINDOW variables: the graph's ranges, tick spacing and resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Window {
    pub x_min: f64,
    pub x_max: f64,
    pub x_scale: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub y_scale: f64,
    /// Every how many columns a function is evaluated, 1 to 8.
    pub x_res: f64,
}

impl Default for Window {
    fn default() -> Self {
        Window::standard()
    }
}

impl Window {
    /// ZStandard: -10 to 10 on both axes, ticks every 1.
    pub fn standard() -> Self {
        Window {
            x_min: -10.0,
            x_max: 10.0,
            x_scale: 1.0,
            y_min: -10.0,
            y_max: 10.0,
            y_scale: 1.0,
            x_res: 1.0,
        }
    }

    /// Whether the window can be graphed: increasing ranges and an integer
    /// resolution from 1 to 8.
    pub fn check(&self) -> Result<(), CalcError> {
        let ranges = self.x_min < self.x_max && self.y_min < self.y_max;
        let resolution = (1.0..=8.0).contains(&self.x_res) && self.x_res.fract() == 0.0;
        if ranges && resolution {
            Ok(())
        } else {
            Err(CalcError::new(ErrorKind::WindowRange))
        }
    }

    /// The x distance between two columns.
    pub fn delta_x(&self) -> f64 {
        (self.x_max - self.x_min) / (GRAPH_COLUMNS - 1) as f64
    }

    /// The y distance between two rows.
    pub fn delta_y(&self) -> f64 {
        (self.y_max - self.y_min) / (GRAPH_ROWS - 1) as f64
    }

    /// The x of graph column `column` (0 is `x_min`, the last is `x_max`).
    pub fn column_x(&self, column: usize) -> f64 {
        self.x_min + (self.x_max - self.x_min) * column as f64 / (GRAPH_COLUMNS - 1) as f64
    }

    /// The window moved by fractions of its width and height (positive `dx`
    /// shows more to the right, positive `dy` more above).
    pub fn panned(self, dx: f64, dy: f64) -> Self {
        let x = dx * (self.x_max - self.x_min);
        let y = dy * (self.y_max - self.y_min);
        Window {
            x_min: self.x_min + x,
            x_max: self.x_max + x,
            y_min: self.y_min + y,
            y_max: self.y_max + y,
            ..self
        }
    }

    /// The window scaled by `1 / factor` about `anchor`, a point given as
    /// fractions of the width (from the left) and height (from the bottom),
    /// which stays where it is: `factor` above 1 zooms in.
    pub fn zoomed(self, factor: f64, anchor: (f64, f64)) -> Self {
        let (x_min, x_max) = scale_range((self.x_min, self.x_max), factor, anchor.0);
        let (y_min, y_max) = scale_range((self.y_min, self.y_max), factor, anchor.1);
        Window {
            x_min,
            x_max,
            y_min,
            y_max,
            ..self
        }
    }

    /// The window moved so `x` is in the middle of the x range.
    pub fn centered_on_x(self, x: f64) -> Self {
        let half = (self.x_max - self.x_min) / 2.0;
        Window {
            x_min: x - half,
            x_max: x + half,
            ..self
        }
    }

    pub fn x_ticks(&self) -> Vec<f64> {
        ticks(self.x_min, self.x_max, self.x_scale)
    }

    pub fn y_ticks(&self) -> Vec<f64> {
        ticks(self.y_min, self.y_max, self.y_scale)
    }
}

fn scale_range((min, max): (f64, f64), factor: f64, anchor: f64) -> (f64, f64) {
    let pivot = min + anchor * (max - min);
    (
        pivot + (min - pivot) / factor,
        pivot + (max - pivot) / factor,
    )
}

/// Multiples of `scale` within `min..=max`, none when `scale` is not
/// positive or would draw more than `MAX_TICKS`.
fn ticks(min: f64, max: f64, scale: f64) -> Vec<f64> {
    if scale <= 0.0 || (max - min) / scale > MAX_TICKS as f64 {
        return Vec::new();
    }
    let first = (min / scale).ceil() as i64;
    let last = (max / scale).floor() as i64;
    (first..=last).map(|n| n as f64 * scale).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_window_checks_and_ticks() {
        let window = Window::standard();
        assert_eq!(window.check(), Ok(()));
        assert_eq!(window.x_ticks().len(), 21);
        assert!((window.delta_x() - 20.0 / 264.0).abs() < 1e-15);
    }

    #[test]
    fn decreasing_ranges_and_bad_resolutions_are_window_range_errors() {
        let flipped = Window {
            x_min: 5.0,
            x_max: 1.0,
            ..Window::standard()
        };
        let coarse = Window {
            x_res: 9.0,
            ..Window::standard()
        };
        for window in [flipped, coarse] {
            assert_eq!(
                window.check().map_err(|e| e.kind),
                Err(ErrorKind::WindowRange)
            );
        }
    }

    #[test]
    fn zoom_keeps_the_anchor_in_place() {
        let zoomed = Window::standard().zoomed(4.0, (0.75, 0.5));
        assert_eq!((zoomed.x_min, zoomed.x_max), (1.25, 6.25));
        assert_eq!((zoomed.y_min, zoomed.y_max), (-2.5, 2.5));
    }

    #[test]
    fn pan_moves_by_fractions_of_the_ranges() {
        let panned = Window::standard().panned(0.25, -0.5);
        assert_eq!((panned.x_min, panned.x_max), (-5.0, 15.0));
        assert_eq!((panned.y_min, panned.y_max), (-20.0, 0.0));
    }

    #[test]
    fn too_fine_a_scale_draws_no_ticks() {
        let fine = Window {
            x_scale: 0.001,
            ..Window::standard()
        };
        assert!(fine.x_ticks().is_empty());
    }
}
