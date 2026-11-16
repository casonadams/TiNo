//! Evaluating expressions over real numbers.

mod arithmetic;
mod calculus;
mod calls;

use std::{
    cell::Cell,
    f64::consts::{E, PI},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{CalcError, ErrorKind},
    expr::{Constant, Expr},
    settings::Angle,
    table::TableSettings,
    token::Letter,
    window::Window,
};

/// Results at or beyond this magnitude overflow, as on the calculator.
const OVERFLOW: f64 = 1e100;
/// Results smaller than this in magnitude are zero.
const UNDERFLOW: f64 = 1e-99;

/// The values expressions read: variables `A`-`Z`, `θ` and `Ans`, all 0 at
/// first, and the state `rand` draws from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    variables: [f64; 27],
    ans: f64,
    #[serde(default)]
    random: Cell<u64>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            variables: [0.0; 27],
            ans: 0.0,
            random: Cell::new(0),
        }
    }
}

impl Memory {
    pub fn get(&self, letter: Letter) -> f64 {
        self.variables[letter as usize]
    }

    pub fn set(&mut self, letter: Letter, value: f64) {
        self.variables[letter as usize] = value;
    }

    pub fn ans(&self) -> f64 {
        self.ans
    }

    pub fn set_ans(&mut self, value: f64) {
        self.ans = value;
    }

    /// The next pseudo-random number, strictly between 0 and 1 (SplitMix64).
    pub fn random(&self) -> f64 {
        let state = self.random.get().wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.random.set(state);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        ((z >> 11) as f64 + 1.0) / ((1u64 << 53) as f64 + 2.0)
    }
}

/// Everything an expression reads besides its own numbers.
#[derive(Debug)]
pub struct Context<'a> {
    pub memory: &'a Memory,
    pub angle: Angle,
    pub window: &'a Window,
    pub table: &'a TableSettings,
    /// Each Y= function parsed, or why reading it fails.
    pub functions: Vec<Result<Expr, CalcError>>,
}

impl Context<'_> {
    /// `expr` over real numbers, trigonometry in the context's angle units.
    pub fn evaluate(&self, expr: &Expr) -> Result<f64, CalcError> {
        Evaluator::new(self).value(expr)
    }

    /// `expr` at `X = x`, or `None` where it is undefined.
    pub fn y_at(&self, expr: &Expr, x: f64) -> Option<f64> {
        Evaluator::new(self).with(Letter::X, x).value(expr).ok()
    }

    /// Y= function `slot` at `X = x`.
    pub fn function_at(&self, slot: usize, x: f64) -> Result<f64, CalcError> {
        Evaluator::new(self).with(Letter::X, x).y_var(slot, None)
    }
}

#[derive(Clone, Copy)]
struct Evaluator<'a> {
    context: &'a Context<'a>,
    /// Values given to letters by `nDeriv(`, `fnInt(`, `Σ(` and the like or
    /// a Y= function's argument, in place of memory's.
    bound: [Option<f64>; 27],
    /// The Y= slots being evaluated, one bit each, to catch a function that
    /// reads itself.
    active: u16,
}

impl<'a> Evaluator<'a> {
    fn new(context: &'a Context<'a>) -> Self {
        Evaluator {
            context,
            bound: [None; 27],
            active: 0,
        }
    }

    fn with(mut self, letter: Letter, value: f64) -> Self {
        self.bound[letter as usize] = Some(value);
        self
    }

    fn value(&self, expr: &Expr) -> Result<f64, CalcError> {
        let memory = self.context.memory;
        let value = match expr {
            Expr::Number(number) => Ok(number.value),
            Expr::Constant(Constant::Pi) => Ok(PI),
            Expr::Constant(Constant::E) => Ok(E),
            Expr::Variable(letter) => {
                Ok(self.bound[*letter as usize].unwrap_or(memory.get(*letter)))
            }
            Expr::Ans => Ok(memory.ans()),
            Expr::Rand => Ok(memory.random()),
            Expr::Window(field) => Ok(field.get(self.context.window)),
            Expr::Table(field) => Ok(field.get(self.context.table)),
            Expr::YVar(slot, argument) => self.y_var(*slot, argument.as_deref()),
            Expr::Negate(operand) => Ok(-self.value(operand)?),
            Expr::ImpliedMultiply(left, right) => Ok(self.value(left)? * self.value(right)?),
            Expr::Binary(op, left, right, at) => located(
                arithmetic::binary(*op, self.value(left)?, self.value(right)?),
                *at,
            ),
            Expr::Postfix(op, operand, at) => {
                located(arithmetic::postfix(*op, self.value(operand)?), *at)
            }
            Expr::Call(function, arguments, at) => located(self.call(*function, arguments), *at),
        };
        in_range(value?)
    }

    /// Y= function `slot` at the current `X`, or at `argument`. Errors in
    /// it have no place in the entry reading it.
    fn y_var(&self, slot: usize, argument: Option<&Expr>) -> Result<f64, CalcError> {
        let bit = 1 << slot;
        if self.active & bit != 0 {
            return fail(ErrorKind::Invalid);
        }
        let function = match self.context.functions.get(slot) {
            Some(Ok(expr)) => expr,
            Some(Err(error)) => return fail(error.kind),
            None => return fail(ErrorKind::Undefined),
        };
        let mut inner = Evaluator {
            active: self.active | bit,
            ..*self
        };
        if let Some(argument) = argument {
            inner = inner.with(Letter::X, self.value(argument)?);
        }
        inner
            .value(function)
            .map_err(|error| CalcError::new(error.kind))
    }
}

pub(crate) fn fail<T>(kind: ErrorKind) -> Result<T, CalcError> {
    Err(CalcError::new(kind))
}

fn in_range(value: f64) -> Result<f64, CalcError> {
    if value.is_nan() {
        return fail(ErrorKind::Domain);
    }
    if value.abs() >= OVERFLOW {
        return fail(ErrorKind::Overflow);
    }
    Ok(if value.abs() < UNDERFLOW { 0.0 } else { value })
}

/// An operation's result in range, its error placed at the operation's token.
fn located(result: Result<f64, CalcError>, at: usize) -> Result<f64, CalcError> {
    result.and_then(in_range).map_err(|error| error.or_at(at))
}

/// `value` as an integer, or a domain error when it has a fractional part
/// or is out of `range`.
pub(crate) fn whole(value: f64, range: std::ops::RangeInclusive<f64>) -> Result<f64, CalcError> {
    if value.fract() == 0.0 && range.contains(&value) {
        Ok(value)
    } else {
        fail(ErrorKind::Domain)
    }
}

#[cfg(test)]
mod tests;
