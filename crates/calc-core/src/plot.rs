use crate::{
    eval::{Memory, evaluate},
    expr::Expr,
    settings::Angle,
    token::Letter,
    window::{GRAPH_COLUMNS, Window},
};

/// How far beyond the y range plotted values are kept, in window heights,
/// so a steep curve still runs off the edge without huge coordinates.
const OVERDRAW: f64 = 10.0;

/// One function's values at the plot's x positions; `None` where it is
/// undefined (a gap in the curve).
#[derive(Debug, Clone, PartialEq)]
pub struct Curve {
    pub slot: usize,
    pub ys: Vec<Option<f64>>,
}

/// The functions sampled across the window: one x per evaluated column.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Plot {
    pub xs: Vec<f64>,
    pub curves: Vec<Curve>,
}

/// What evaluating functions of `X` needs besides the expression.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    pub memory: &'a Memory,
    pub angle: Angle,
}

impl Context<'_> {
    /// `expr` at `X = x`, or `None` where it is undefined.
    pub fn y_at(self, expr: &Expr, x: f64) -> Option<f64> {
        let mut memory = self.memory.clone();
        memory.set(Letter::X, x);
        evaluate(expr, &memory, self.angle).ok()
    }
}

/// Samples `functions` at every `x_res`-th column of `window`.
pub fn plot(functions: &[(usize, Expr)], window: &Window, context: Context<'_>) -> Plot {
    let step = window.x_res as usize;
    let xs: Vec<f64> = (0..GRAPH_COLUMNS)
        .step_by(step.max(1))
        .map(|column| window.column_x(column))
        .collect();
    let height = window.y_max - window.y_min;
    let limits = (
        window.y_min - OVERDRAW * height,
        window.y_max + OVERDRAW * height,
    );
    let curves = functions
        .iter()
        .map(|(slot, expr)| Curve {
            slot: *slot,
            ys: xs
                .iter()
                .map(|&x| context.y_at(expr, x).map(|y| y.clamp(limits.0, limits.1)))
                .collect(),
        })
        .collect();
    Plot { xs, curves }
}

/// The smallest and largest value the plot's curves take, if any.
pub fn y_extent(plot: &Plot) -> Option<(f64, f64)> {
    plot.curves
        .iter()
        .flat_map(|curve| curve.ys.iter().flatten())
        .fold(None, |extent, &y| match extent {
            None => Some((y, y)),
            Some((low, high)) => Some((f64::min(low, y), f64::max(high, y))),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse, tests::tokens};

    fn function(spelling: &str) -> Expr {
        parse(&tokens(spelling)).unwrap().expr
    }

    fn context(memory: &Memory) -> Context<'_> {
        Context {
            memory,
            angle: Angle::Radian,
        }
    }

    #[test]
    fn samples_every_column_from_x_min_to_x_max() {
        let memory = Memory::default();
        let plot = plot(
            &[(0, function("X²"))],
            &Window::standard(),
            context(&memory),
        );
        assert_eq!(plot.xs.len(), GRAPH_COLUMNS);
        assert_eq!((plot.xs[0], plot.xs[GRAPH_COLUMNS - 1]), (-10.0, 10.0));
        assert_eq!(plot.curves[0].ys[0], Some(100.0));
    }

    #[test]
    fn undefined_points_are_gaps() {
        let memory = Memory::default();
        let plot = plot(
            &[(2, function("r(X"))],
            &Window::standard(),
            context(&memory),
        );
        assert_eq!(plot.curves[0].slot, 2);
        assert_eq!(plot.curves[0].ys[0], None);
        assert_eq!(plot.curves[0].ys[GRAPH_COLUMNS - 1], Some(10f64.sqrt()));
    }

    #[test]
    fn coarser_resolution_samples_fewer_columns() {
        let window = Window {
            x_res: 4.0,
            ..Window::standard()
        };
        let memory = Memory::default();
        let plot = plot(&[(0, function("X"))], &window, context(&memory));
        assert_eq!(plot.xs.len(), GRAPH_COLUMNS.div_ceil(4));
    }

    #[test]
    fn extent_spans_every_curve() {
        let memory = Memory::default();
        let plot = plot(
            &[(0, function("X")), (1, function("X²"))],
            &Window::standard(),
            context(&memory),
        );
        assert_eq!(y_extent(&plot), Some((-10.0, 100.0)));
    }
}
