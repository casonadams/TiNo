//! Numerical calculus over functions that may fail: the derivative,
//! integral, extrema and sum behind `nDeriv(`, `fnInt(`, `fMin(`, `fMax(`
//! and `Σ(`.

use crate::{
    error::{CalcError, ErrorKind},
    eval::{fail, whole},
};

type Value = Result<f64, CalcError>;

/// Subintervals `integral` splits into at most before giving up.
const MOST_SUBINTERVALS: usize = 2000;
/// Steps `extremum`'s golden-section search takes at most.
const MOST_SEARCH_STEPS: usize = 200;
/// Halvings that refine an extremum on the derivative's sign change.
const REFINE_STEPS: usize = 60;
/// Terms `sum` adds at most.
const MOST_TERMS: f64 = 1e6;

/// The symmetric difference quotient `(f(x+ε) − f(x−ε)) / 2ε`.
pub fn derivative(f: impl Fn(f64) -> Value, x: f64, step: f64) -> Value {
    if step == 0.0 {
        return fail(ErrorKind::Domain);
    }
    Ok((f(x + step)? - f(x - step)?) / (2.0 * step))
}

/// The integral of `f` from `a` to `b`, by adaptive Gauss–Kronrod (7, 15)
/// quadrature, to within `tolerance` or 1ᴇ⁻¹⁰ relative, whichever is finer.
pub fn integral(f: impl Fn(f64) -> Value, (a, b): (f64, f64), tolerance: f64) -> Value {
    if tolerance <= 0.0 {
        return fail(ErrorKind::Domain);
    }
    let mut pieces = vec![kronrod(&f, a, b)?];
    loop {
        let total: f64 = pieces.iter().map(|piece| piece.estimate).sum();
        let error: f64 = pieces.iter().map(|piece| piece.error).sum();
        if error <= tolerance.min(1e-10) * total.abs().max(1.0) {
            return Ok(total);
        }
        if pieces.len() >= MOST_SUBINTERVALS {
            return fail(ErrorKind::ToleranceNotMet);
        }
        let worst = (0..pieces.len())
            .max_by(|&i, &j| pieces[i].error.total_cmp(&pieces[j].error))
            .unwrap_or(0);
        let Piece { a, b, .. } = pieces.swap_remove(worst);
        let middle = (a + b) / 2.0;
        pieces.push(kronrod(&f, a, middle)?);
        pieces.push(kronrod(&f, middle, b)?);
    }
}

/// One subinterval's integral and how far it may be off.
struct Piece {
    a: f64,
    b: f64,
    estimate: f64,
    error: f64,
}

/// Kronrod nodes on [0, 1] (the last is the center) and their weights.
const KRONROD_NODES: [f64; 8] = [
    0.991_455_371_120_812_6,
    0.949_107_912_342_758_5,
    0.864_864_423_359_769_1,
    0.741_531_185_599_394_4,
    0.586_087_235_467_691_1,
    0.405_845_151_377_397_2,
    0.207_784_955_007_898_5,
    0.0,
];
const KRONROD_WEIGHTS: [f64; 8] = [
    0.022_935_322_010_529_22,
    0.063_092_092_629_978_55,
    0.104_790_010_322_250_2,
    0.140_653_259_715_525_9,
    0.169_004_726_639_267_9,
    0.190_350_578_064_785_4,
    0.204_432_940_075_298_9,
    0.209_482_141_084_727_8,
];
/// Gauss weights for the odd Kronrod nodes (1, 3, 5 and the center).
const GAUSS_WEIGHTS: [f64; 4] = [
    0.129_484_966_168_869_7,
    0.279_705_391_489_276_7,
    0.381_830_050_505_118_9,
    0.417_959_183_673_469_4,
];

fn kronrod(f: &impl Fn(f64) -> Value, a: f64, b: f64) -> Result<Piece, CalcError> {
    let (center, half) = ((a + b) / 2.0, (b - a) / 2.0);
    let (mut kronrod, mut gauss) = (0.0, 0.0);
    for (index, (&node, &weight)) in KRONROD_NODES.iter().zip(&KRONROD_WEIGHTS).enumerate() {
        let sum = if node == 0.0 {
            f(center)?
        } else {
            f(center - half * node)? + f(center + half * node)?
        };
        kronrod += weight * sum;
        if index % 2 == 1 {
            gauss += GAUSS_WEIGHTS[index / 2] * sum;
        }
    }
    Ok(Piece {
        a,
        b,
        estimate: kronrod * half,
        error: ((kronrod - gauss) * half).abs(),
    })
}

/// Which extremum `extremum` looks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extremum {
    Minimum,
    Maximum,
}

/// Where `f` is least (or greatest) on `[a, b]`: a golden-section search
/// to within `tolerance`, refined where the derivative changes sign, and
/// an end of the interval when the extremum lies there.
pub fn extremum(f: impl Fn(f64) -> Value, (a, b): (f64, f64), kind: Extremum) -> Value {
    let sign = match kind {
        Extremum::Minimum => 1.0,
        Extremum::Maximum => -1.0,
    };
    let g = |x: f64| f(x).map(|y| sign * y);
    let (low, high) = golden_section(&g, a.min(b), a.max(b))?;
    let inside = refine(&g, low, high)?;
    [a, b].into_iter().try_fold(inside, |best, end| {
        Ok(if g(end)? < g(best)? { end } else { best })
    })
}

fn golden_section(
    g: &impl Fn(f64) -> Value,
    mut low: f64,
    mut high: f64,
) -> Result<(f64, f64), CalcError> {
    let ratio = (5f64.sqrt() - 1.0) / 2.0;
    let mut left = high - ratio * (high - low);
    let mut right = low + ratio * (high - low);
    let (mut g_left, mut g_right) = (g(left)?, g(right)?);
    for _ in 0..MOST_SEARCH_STEPS {
        if high - low <= 1e-7 * high.abs().max(low.abs()).max(1.0) {
            break;
        }
        if g_left < g_right {
            (high, right, g_right) = (right, left, g_left);
            left = high - ratio * (high - low);
            g_left = g(left)?;
        } else {
            (low, left, g_left) = (left, right, g_right);
            right = low + ratio * (high - low);
            g_right = g(right)?;
        }
    }
    Ok((low, high))
}

/// Within `[low, high]`, where the derivative of `g` turns from falling to
/// rising, or the middle when it does not.
fn refine(g: &impl Fn(f64) -> Value, mut low: f64, mut high: f64) -> Value {
    let step = 1e-6 * high.abs().max(1.0);
    let slope = |x: f64| derivative(g, x, step);
    if !(slope(low)? < 0.0 && slope(high)? > 0.0) {
        return Ok((low + high) / 2.0);
    }
    for _ in 0..REFINE_STEPS {
        let middle = (low + high) / 2.0;
        if slope(middle)? < 0.0 {
            low = middle;
        } else {
            high = middle;
        }
    }
    Ok((low + high) / 2.0)
}

/// Halvings `root` takes at most; more cannot narrow an `f64` interval.
const ROOT_STEPS: usize = 200;

/// Where `f` crosses zero in `[a, b]`, by bisection; `ERR:NO SIGN CHNG`
/// when `f` has the same sign at both ends.
pub fn root(f: impl Fn(f64) -> Value, (a, b): (f64, f64)) -> Value {
    let (low, high) = (a.min(b), a.max(b));
    let (low_value, high_value) = (f(low)?, f(high)?);
    if low_value == 0.0 || high_value == 0.0 {
        return Ok(if low_value == 0.0 { low } else { high });
    }
    if low_value.signum() == high_value.signum() {
        return fail(ErrorKind::NoSignChange);
    }
    bisect(f, (low, high), low_value.signum())
}

/// Halves `[low, high]`, keeping the half whose ends `f` gives opposite
/// signs, until it cannot narrow; `f` has sign `low_sign` at `low`.
fn bisect(f: impl Fn(f64) -> Value, (mut low, mut high): (f64, f64), low_sign: f64) -> Value {
    for _ in 0..ROOT_STEPS {
        let middle = (low + high) / 2.0;
        if middle <= low || middle >= high {
            break;
        }
        match f(middle)? {
            0.0 => return Ok(middle),
            value if value.signum() == low_sign => low = middle,
            _ => high = middle,
        }
    }
    Ok((low + high) / 2.0)
}

/// `f(start) + f(start + 1) + … + f(end)` over whole `start` and `end`;
/// 0 when `end` is before `start`.
pub fn sum(f: impl Fn(f64) -> Value, start: f64, end: f64) -> Value {
    let start = whole(start, -1e12..=1e12)?;
    let end = whole(end, -1e12..=1e12)?;
    if end - start >= MOST_TERMS {
        return fail(ErrorKind::Domain);
    }
    let mut total = 0.0;
    let mut k = start;
    while k <= end {
        total += f(k)?;
        k += 1.0;
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn close(actual: Value, expected: f64) {
        let actual = actual.unwrap();
        assert!(
            (actual - expected).abs() < 1e-9,
            "{actual} is not {expected}"
        );
    }

    #[test]
    fn derivatives_are_symmetric_difference_quotients() {
        close(derivative(|x| Ok(x * x * x), 2.0, 1e-3), 12.000001);
        assert_eq!(
            derivative(Ok, 1.0, 0.0).unwrap_err().kind,
            ErrorKind::Domain
        );
    }

    #[test]
    fn integrals_match_their_antiderivatives() {
        close(integral(|x| Ok(x * x), (0.0, 3.0), 1e-5), 9.0);
        close(integral(|x| Ok(x.sin()), (0.0, PI), 1e-5), 2.0);
        close(integral(|x| Ok(x.sqrt()), (0.0, 1.0), 1e-5), 2.0 / 3.0);
        close(integral(Ok, (2.0, 0.0), 1e-5), -2.0);
        close(
            integral(|x| Ok(1.0 / (1.0 + x * x)), (-1e3, 1e3), 1e-5),
            2.0 * 1e3f64.atan(),
        );
    }

    #[test]
    fn extrema_are_found_inside_or_at_the_ends() {
        let parabola = |x: f64| Ok(x * x - 2.0 * x);
        close(extremum(parabola, (0.0, 5.0), Extremum::Minimum), 1.0);
        close(extremum(parabola, (0.0, 5.0), Extremum::Maximum), 5.0);
        close(
            extremum(|x| Ok(x.sin()), (0.0, PI), Extremum::Maximum),
            PI / 2.0,
        );
        close(extremum(Ok, (3.0, -1.0), Extremum::Minimum), -1.0);
    }

    #[test]
    fn sums_run_over_whole_numbers() {
        close(sum(|k| Ok(k * k), 1.0, 10.0), 385.0);
        close(sum(Ok, 5.0, 4.0), 0.0);
        assert_eq!(sum(Ok, 0.5, 2.0).unwrap_err().kind, ErrorKind::Domain);
    }

    #[test]
    fn roots_are_found_where_the_sign_changes() {
        close(root(|x| Ok(x * x - 2.0), (0.0, 5.0)), 2f64.sqrt());
        close(root(|x| Ok(x - 1.0), (1.0, 3.0)), 1.0);
        assert_eq!(
            root(|x| Ok(x * x + 1.0), (-1.0, 1.0)).unwrap_err().kind,
            ErrorKind::NoSignChange
        );
    }
}
