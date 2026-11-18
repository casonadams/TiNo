use super::{Evaluator, fail, value::Value};
use crate::{
    error::{CalcError, ErrorKind},
    expr::Expr,
    numeric::{self, Extremum},
    vocabulary::Function,
};

/// `nDeriv(`'s step when none is given.
const DEFAULT_STEP: f64 = 1e-3;
/// `fnInt(`, `fMin(` and `fMax(`'s tolerance when none is given.
const DEFAULT_TOLERANCE: f64 = 1e-5;
/// Elements a list holds at most, as on the calculator.
const LONGEST_LIST: f64 = 999.0;

impl Evaluator<'_> {
    /// `nDeriv(`, `fnInt(`, `fMin(`, `fMax(`, `Σ(` and `seq(`: the first
    /// argument evaluated with the variable the second names set to each
    /// value the method needs; the rest are numbers. Integrals and extrema
    /// are found at least as closely as the tolerance asks.
    pub(super) fn calculus(
        &self,
        function: Function,
        arguments: &[Expr],
    ) -> Result<Value, CalcError> {
        let [body, Expr::Variable(letter), rest @ ..] = arguments else {
            return fail(ErrorKind::Syntax);
        };
        let f = |value: f64| self.with(*letter, value).real(body);
        let numbers = rest
            .iter()
            .map(|argument| self.real(argument))
            .collect::<Result<Vec<_>, _>>()?;
        if function == Function::Seq {
            return sequence(f, &numbers).map(Value::List);
        }
        let tolerance = numbers.get(2).copied().unwrap_or(DEFAULT_TOLERANCE);
        if tolerance <= 0.0 {
            return fail(ErrorKind::Domain);
        }
        let value = match (function, numbers.as_slice()) {
            (Function::NDeriv, &[x]) => numeric::derivative(f, x, DEFAULT_STEP),
            (Function::NDeriv, &[x, step]) => numeric::derivative(f, x, step),
            (Function::FnInt, &[a, b, ..]) => numeric::integral(f, (a, b), tolerance),
            (Function::FMin, &[a, b, ..]) => numeric::extremum(f, (a, b), Extremum::Minimum),
            (Function::FMax, &[a, b, ..]) => numeric::extremum(f, (a, b), Extremum::Maximum),
            (_, &[start, end]) => numeric::sum(f, start, end),
            _ => fail(ErrorKind::Argument),
        };
        value.map(Value::Real)
    }
}

/// `seq(`: `f` from `start` toward `end` by `step` (1 when left out), as far
/// as `end`; at most 999 elements.
fn sequence(
    f: impl Fn(f64) -> Result<f64, CalcError>,
    numbers: &[f64],
) -> Result<Vec<f64>, CalcError> {
    let (start, end, step) = match *numbers {
        [start, end] => (start, end, 1.0),
        [start, end, step] => (start, end, step),
        _ => return fail(ErrorKind::Argument),
    };
    if step == 0.0 {
        return fail(ErrorKind::Domain);
    }
    let count = ((end - start) / step).floor() + 1.0;
    if count > LONGEST_LIST {
        return fail(ErrorKind::InvalidDim);
    }
    (0..count.max(0.0) as usize)
        .map(|k| f(start + k as f64 * step))
        .collect()
}
