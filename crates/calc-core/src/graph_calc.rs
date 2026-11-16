//! The graph's CALC menu: what each item asks for on the graph and what it
//! computes from the points marked.

use crate::{
    error::{CalcError, ErrorKind},
    eval::fail,
    numeric::{self, Extremum},
    trace::TraceCursor,
    window::Window,
};

/// `nDeriv(`'s step, which dy/dx uses as the calculator does.
const DERIVATIVE_STEP: f64 = 1e-3;
/// `fnInt(`'s default tolerance, which ∫f(x)dx uses.
const INTEGRAL_TOLERANCE: f64 = 1e-5;

/// An item of the CALC menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcItem {
    Value,
    Zero,
    Minimum,
    Maximum,
    Intersect,
    Derivative,
    Integral,
}

impl CalcItem {
    pub const ALL: [CalcItem; 7] = [
        CalcItem::Value,
        CalcItem::Zero,
        CalcItem::Minimum,
        CalcItem::Maximum,
        CalcItem::Intersect,
        CalcItem::Derivative,
        CalcItem::Integral,
    ];

    pub fn label(self) -> &'static str {
        match self {
            CalcItem::Value => "value",
            CalcItem::Zero => "zero",
            CalcItem::Minimum => "minimum",
            CalcItem::Maximum => "maximum",
            CalcItem::Intersect => "intersect",
            CalcItem::Derivative => "dy/dx",
            CalcItem::Integral => "∫f(x)dx",
        }
    }

    /// The digit that picks the item: its place in the menu, from 1.
    pub fn digit(self) -> u8 {
        CalcItem::ALL
            .iter()
            .position(|&item| item == self)
            .unwrap_or(0) as u8
            + 1
    }

    /// The prompts for the points to mark on the graph, in order; dy/dx
    /// marks its point without one. `value` asks for an X instead.
    pub fn prompts(self) -> &'static [&'static str] {
        match self {
            CalcItem::Value => &[],
            CalcItem::Zero => &["Left Bound?", "Right Bound?", "Guess?"],
            CalcItem::Minimum | CalcItem::Maximum => &["Left Bound?", "Right Bound?"],
            CalcItem::Intersect => &["First curve?", "Second curve?", "Guess?"],
            CalcItem::Derivative => &[""],
            CalcItem::Integral => &["Lower Limit?", "Upper Limit?"],
        }
    }
}

/// What a finished CALC item shows: the point the cursor moves to, the
/// title over it, a value of its own (`dy/dx=…`), the interval shaded, and
/// the y to show when the item knows it exactly (0 at a zero, not the
/// rounding left in the function's value there).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CalcResult {
    pub cursor: TraceCursor,
    pub title: Option<&'static str>,
    pub value: Option<(&'static str, f64)>,
    pub shade: Option<(f64, f64)>,
    pub exact_y: Option<f64>,
}

/// Computes `item` from the points marked for its prompts, reading graphed
/// function `slot` at `x` with `y`; intersect searches across `window`.
pub fn compute(
    item: CalcItem,
    marks: &[TraceCursor],
    (y, window): (&dyn Fn(usize, f64) -> Result<f64, CalcError>, &Window),
) -> Result<CalcResult, CalcError> {
    let first = marks
        .first()
        .copied()
        .ok_or(CalcError::new(ErrorKind::Invalid))?;
    let f = |x: f64| y(first.slot, x);
    let xs: Vec<f64> = marks.iter().map(|mark| mark.x).collect();
    let at = |x: f64| TraceCursor { x, ..first };
    let result = |cursor, title| CalcResult {
        cursor,
        title,
        value: None,
        shade: None,
        exact_y: None,
    };
    match (item, &xs[..]) {
        (CalcItem::Zero, &[left, right, guess]) => Ok(CalcResult {
            exact_y: Some(0.0),
            ..result(at(zero(f, (left, right), guess)?), Some("Zero"))
        }),
        (CalcItem::Minimum, &[left, right]) => {
            let x = numeric::extremum(f, (left, right), Extremum::Minimum)?;
            Ok(result(at(x), Some("Minimum")))
        }
        (CalcItem::Maximum, &[left, right]) => {
            let x = numeric::extremum(f, (left, right), Extremum::Maximum)?;
            Ok(result(at(x), Some("Maximum")))
        }
        (CalcItem::Derivative, &[x]) => Ok(CalcResult {
            value: Some(("dy/dx", numeric::derivative(f, x, DERIVATIVE_STEP)?)),
            ..result(at(x), None)
        }),
        (CalcItem::Integral, &[lower, upper]) => Ok(CalcResult {
            value: Some((
                "∫f(x)dx",
                numeric::integral(f, (lower, upper), INTEGRAL_TOLERANCE)?,
            )),
            shade: Some((lower, upper)),
            ..result(at(upper), None)
        }),
        (CalcItem::Intersect, &[_, _, guess]) => {
            intersection((first, marks[1]), guess, (y, window))
        }
        _ => fail(ErrorKind::Invalid),
    }
}

/// zero: the root between the bounds, on the side of the guess with a
/// sign change, or between the bounds when neither side alone has one.
fn zero(
    f: impl Fn(f64) -> Result<f64, CalcError>,
    bounds: (f64, f64),
    guess: f64,
) -> Result<f64, CalcError> {
    let (low, high) = (bounds.0.min(bounds.1), bounds.0.max(bounds.1));
    let guess = guess.clamp(low, high);
    numeric::root(&f, (low, guess))
        .or_else(|_| numeric::root(&f, (guess, high)))
        .or_else(|_| numeric::root(&f, (low, high)))
}

/// intersect: where curves `first` and `second` meet nearest `guess`,
/// searching outward from it a column at a time across the window.
fn intersection(
    (first, second): (TraceCursor, TraceCursor),
    guess: f64,
    (y, window): (&dyn Fn(usize, f64) -> Result<f64, CalcError>, &Window),
) -> Result<CalcResult, CalcError> {
    let gap = |x: f64| Ok(y(first.slot, x)? - y(second.slot, x)?);
    let step = window.delta_x();
    let columns = ((window.x_max - window.x_min) / step).ceil() as usize;
    for k in 1..=columns {
        let reach = k as f64 * step;
        for bracket in [(guess - reach, guess), (guess, guess + reach)] {
            if let Ok(x) = numeric::root(gap, bracket) {
                return Ok(CalcResult {
                    cursor: TraceCursor { x, ..first },
                    title: Some("Intersection"),
                    value: None,
                    shade: None,
                    exact_y: None,
                });
            }
        }
    }
    fail(ErrorKind::NoSignChange)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn functions(slot: usize, x: f64) -> Result<f64, CalcError> {
        Ok(match slot {
            0 => x * x - 2.0,
            _ => x,
        })
    }

    fn mark(slot: usize, x: f64) -> TraceCursor {
        TraceCursor { slot, x }
    }

    fn close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual} is not {expected}"
        );
    }

    fn run(item: CalcItem, marks: &[TraceCursor]) -> Result<CalcResult, CalcError> {
        compute(item, marks, (&functions, &Window::standard()))
    }

    #[test]
    fn zero_minimum_and_maximum_land_the_cursor_on_the_point() {
        let marks = [mark(0, -1.0), mark(0, 5.0), mark(0, 1.0)];
        let zero = run(CalcItem::Zero, &marks).unwrap();
        close(zero.cursor.x, 2f64.sqrt());
        assert_eq!(zero.title, Some("Zero"));
        let minimum = run(CalcItem::Minimum, &marks[..2]).unwrap();
        close(minimum.cursor.x, 0.0);
        let maximum = run(CalcItem::Maximum, &marks[..2]).unwrap();
        close(maximum.cursor.x, 5.0);
    }

    #[test]
    fn the_guess_picks_which_zero() {
        let marks = [mark(0, -5.0), mark(0, 5.0), mark(0, -1.0)];
        let zero = run(CalcItem::Zero, &marks).unwrap();
        close(zero.cursor.x, -(2f64.sqrt()));
        let none = [mark(1, 1.0), mark(1, 5.0), mark(1, 2.0)];
        let error = run(CalcItem::Zero, &none).unwrap_err();
        assert_eq!(error.kind, ErrorKind::NoSignChange);
    }

    #[test]
    fn derivative_and_integral_report_their_values() {
        let derivative = run(CalcItem::Derivative, &[mark(0, 3.0)]).unwrap();
        close(derivative.value.unwrap().1, 6.0);
        let integral = run(CalcItem::Integral, &[mark(1, 0.0), mark(1, 2.0)]).unwrap();
        assert_eq!(integral.value.map(|(label, _)| label), Some("∫f(x)dx"));
        close(integral.value.unwrap().1, 2.0);
        assert_eq!(integral.shade, Some((0.0, 2.0)));
    }

    #[test]
    fn intersection_searches_out_from_the_guess() {
        let found = run(
            CalcItem::Intersect,
            &[mark(0, 0.0), mark(1, 0.0), mark(1, 1.5)],
        )
        .unwrap();
        close(found.cursor.x, 2.0);
        assert_eq!(found.title, Some("Intersection"));
        let other = run(
            CalcItem::Intersect,
            &[mark(0, 0.0), mark(1, 0.0), mark(1, -3.0)],
        )
        .unwrap();
        close(other.cursor.x, -1.0);
    }
}
