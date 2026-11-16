//! Evaluating expressions over real numbers.

mod arithmetic;
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
    token::Letter,
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

/// Evaluates `expr` over real numbers, trigonometry in `angle` units.
pub fn evaluate(expr: &Expr, memory: &Memory, angle: Angle) -> Result<f64, CalcError> {
    Evaluator { memory, angle }.value(expr)
}

struct Evaluator<'a> {
    memory: &'a Memory,
    angle: Angle,
}

impl Evaluator<'_> {
    fn value(&self, expr: &Expr) -> Result<f64, CalcError> {
        let value = match expr {
            Expr::Number(number) => number.value,
            Expr::Constant(Constant::Pi) => PI,
            Expr::Constant(Constant::E) => E,
            Expr::Variable(letter) => self.memory.get(*letter),
            Expr::Ans => self.memory.ans(),
            Expr::Rand => self.memory.random(),
            Expr::Negate(operand) => -self.value(operand)?,
            Expr::Postfix(op, operand) => arithmetic::postfix(*op, self.value(operand)?)?,
            Expr::Binary(op, left, right) => {
                arithmetic::binary(*op, self.value(left)?, self.value(right)?)?
            }
            Expr::ImpliedMultiply(left, right) => self.value(left)? * self.value(right)?,
            Expr::Call(function, arguments) => self.call(*function, arguments)?,
        };
        in_range(value)
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
