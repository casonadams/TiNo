use std::f64::consts::{E, PI};

use serde::{Deserialize, Serialize};

use crate::{
    error::{CalcError, ErrorKind},
    expr::{BinaryOp, Constant, Expr, Function, Postfix},
    settings::Angle,
    token::Letter,
    trig,
};

/// Results at or beyond this magnitude overflow, as on the calculator.
const OVERFLOW: f64 = 1e100;
/// Results smaller than this in magnitude are zero.
const UNDERFLOW: f64 = 1e-99;

/// The values expressions read: variables `A`-`Z`, `θ` and `Ans`, all 0 at first.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    variables: [f64; 27],
    ans: f64,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            variables: [0.0; 27],
            ans: 0.0,
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
            Expr::Negate(operand) => -self.value(operand)?,
            Expr::Postfix(op, operand) => postfix(*op, self.value(operand)?)?,
            Expr::Binary(op, left, right) => binary(*op, self.value(left)?, self.value(right)?)?,
            Expr::Function(function, argument) => self.apply(*function, self.value(argument)?)?,
        };
        in_range(value)
    }

    fn apply(&self, function: Function, x: f64) -> Result<f64, CalcError> {
        match function {
            Function::Sqrt if x < 0.0 => fail(ErrorKind::NonReal),
            Function::Sqrt => Ok(x.sqrt()),
            Function::Sin | Function::Cos | Function::Tan => trig::forward(function, x, self.angle),
            Function::ArcSin | Function::ArcCos | Function::ArcTan => {
                trig::inverse(function, x, self.angle)
            }
            Function::Log | Function::Ln => logarithm(function, x),
            Function::TenPower => Ok(10f64.powf(x)),
            Function::ExpPower => Ok(x.exp()),
        }
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

fn postfix(op: Postfix, x: f64) -> Result<f64, CalcError> {
    match op {
        Postfix::Square => Ok(x * x),
        Postfix::Inverse if x == 0.0 => fail(ErrorKind::DivideByZero),
        Postfix::Inverse => Ok(1.0 / x),
    }
}

fn binary(op: BinaryOp, left: f64, right: f64) -> Result<f64, CalcError> {
    match op {
        BinaryOp::Add => Ok(left + right),
        BinaryOp::Subtract => Ok(left - right),
        BinaryOp::Multiply | BinaryOp::ImpliedMultiply => Ok(left * right),
        BinaryOp::Divide if right == 0.0 => fail(ErrorKind::DivideByZero),
        BinaryOp::Divide => Ok(left / right),
        BinaryOp::Power => power(left, right),
    }
}

fn power(base: f64, exponent: f64) -> Result<f64, CalcError> {
    if base == 0.0 {
        return zero_power(exponent);
    }
    if base < 0.0 && exponent.fract() != 0.0 {
        return negative_root(base, exponent);
    }
    Ok(base.powf(exponent))
}

fn zero_power(exponent: f64) -> Result<f64, CalcError> {
    match exponent {
        0.0 => fail(ErrorKind::Domain),
        e if e < 0.0 => fail(ErrorKind::DivideByZero),
        _ => Ok(0.0),
    }
}

/// A negative base to a fractional power is real only when the exponent is
/// a ratio with an odd denominator (`(⁻8)^(1/3)` is `⁻2`).
fn negative_root(base: f64, exponent: f64) -> Result<f64, CalcError> {
    let ratio = (1..100)
        .step_by(2)
        .map(|denominator| (denominator, exponent * f64::from(denominator)))
        .find(|(_, numerator)| (numerator - numerator.round()).abs() < 1e-9);
    match ratio {
        Some((_, numerator)) => {
            let magnitude = (-base).powf(exponent);
            Ok(if numerator.round() % 2.0 == 0.0 {
                magnitude
            } else {
                -magnitude
            })
        }
        None => fail(ErrorKind::NonReal),
    }
}

fn logarithm(function: Function, x: f64) -> Result<f64, CalcError> {
    match x {
        0.0 => fail(ErrorKind::Domain),
        x if x < 0.0 => fail(ErrorKind::NonReal),
        x if function == Function::Log => Ok(x.log10()),
        x => Ok(x.ln()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse, tests::tokens};

    fn value_of(spelling: &str, memory: &Memory) -> Result<f64, CalcError> {
        evaluate(
            &parse(&tokens(spelling)).unwrap().expr,
            memory,
            Angle::Radian,
        )
    }

    #[test]
    fn evaluates_with_the_calculators_precedence() {
        let memory = Memory::default();
        let cases = [
            ("2+3*4", 14.0),
            ("_2²", -4.0),
            ("2^3^2", 64.0),
            ("2_3", -6.0),
            ("2^_1", 0.5),
            ("2(3+4", 14.0),
            ("(2)(3)", 6.0),
            ("1/2p", PI / 2.0),
            ("2p", 2.0 * PI),
            ("r(16", 4.0),
            ("r(9)²", 9.0),
            ("4!", 0.25),
            ("E3", 1000.0),
            ("1.5E_2", 0.015),
            (".5+5.", 5.5),
            ("s(0", 0.0),
            ("(_8)^(1/3", -2.0),
            ("(_8)^(2/3", 4.0),
        ];
        for (spelling, expected) in cases {
            let value = value_of(spelling, &memory).unwrap();
            assert!(
                (value - expected).abs() < 1e-12,
                "{spelling}: {value} != {expected}"
            );
        }
    }

    #[test]
    fn reads_variables_and_ans() {
        let mut memory = Memory::default();
        memory.set(Letter::A, 3.0);
        memory.set_ans(10.0);
        assert_eq!(value_of("2A+a", &memory), Ok(16.0));
        assert_eq!(value_of("AB", &memory), Ok(0.0));
    }

    #[test]
    fn reports_the_calculators_errors() {
        let memory = Memory::default();
        let cases = [
            ("1/0", ErrorKind::DivideByZero),
            ("0!", ErrorKind::DivideByZero),
            ("0^0", ErrorKind::Domain),
            ("0^_1", ErrorKind::DivideByZero),
            ("r(_4", ErrorKind::NonReal),
            ("(_2)^.5", ErrorKind::NonReal),
            ("E99*10", ErrorKind::Overflow),
            ("E100", ErrorKind::Overflow),
        ];
        for (spelling, kind) in cases {
            assert_eq!(
                value_of(spelling, &memory).map_err(|e| e.kind),
                Err(kind),
                "{spelling}"
            );
        }
    }

    #[test]
    fn tiny_results_are_zero() {
        assert_eq!(value_of("E_60*E_60", &Memory::default()), Ok(0.0));
    }
}
