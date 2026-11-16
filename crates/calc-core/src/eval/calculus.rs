use super::{Evaluator, fail};
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

impl Evaluator<'_> {
    /// `nDeriv(`, `fnInt(`, `fMin(`, `fMax(` and `Σ(`: the first argument
    /// evaluated with the variable the second names set to each value the
    /// method needs; the rest are numbers. Integrals and extrema are found
    /// at least as closely as the tolerance asks.
    pub(super) fn calculus(
        &self,
        function: Function,
        arguments: &[Expr],
    ) -> Result<f64, CalcError> {
        let [body, Expr::Variable(letter), rest @ ..] = arguments else {
            return fail(ErrorKind::Syntax);
        };
        let f = |value: f64| self.with(*letter, value).value(body);
        let numbers = rest
            .iter()
            .map(|argument| self.value(argument))
            .collect::<Result<Vec<_>, _>>()?;
        let tolerance = numbers.get(2).copied().unwrap_or(DEFAULT_TOLERANCE);
        if tolerance <= 0.0 {
            return fail(ErrorKind::Domain);
        }
        match (function, numbers.as_slice()) {
            (Function::NDeriv, &[x]) => numeric::derivative(f, x, DEFAULT_STEP),
            (Function::NDeriv, &[x, step]) => numeric::derivative(f, x, step),
            (Function::FnInt, &[a, b, ..]) => numeric::integral(f, (a, b), tolerance),
            (Function::FMin, &[a, b, ..]) => numeric::extremum(f, (a, b), Extremum::Minimum),
            (Function::FMax, &[a, b, ..]) => numeric::extremum(f, (a, b), Extremum::Maximum),
            (_, &[start, end]) => numeric::sum(f, start, end),
            _ => fail(ErrorKind::Argument),
        }
    }
}
