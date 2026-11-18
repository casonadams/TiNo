use super::{fail, whole};
use crate::{
    error::{CalcError, ErrorKind},
    vocabulary::{Operator, PostfixOp},
};

/// The largest `n` whose `n!` is below the overflow limit.
const LARGEST_FACTORIAL: f64 = 69.0;

pub(super) fn postfix(op: PostfixOp, x: f64) -> Result<f64, CalcError> {
    match op {
        PostfixOp::Square => Ok(x * x),
        PostfixOp::Cube => Ok(x * x * x),
        PostfixOp::Inverse if x == 0.0 => fail(ErrorKind::DivideByZero),
        PostfixOp::Inverse => Ok(1.0 / x),
        PostfixOp::Factorial => factorial(x),
        PostfixOp::Transpose => fail(ErrorKind::DataType),
    }
}

fn factorial(n: f64) -> Result<f64, CalcError> {
    let n = whole(n, 0.0..=LARGEST_FACTORIAL)?;
    Ok((1..=n as u32).map(f64::from).product())
}

pub(super) fn binary(op: Operator, left: f64, right: f64) -> Result<f64, CalcError> {
    match op.group() {
        Group::Arithmetic => arithmetic(op, left, right),
        Group::Counting => counting(op, left, right),
        Group::Comparison => Ok(truth(compare(op, left, right))),
        Group::Logic => Ok(truth(logic(op, left != 0.0, right != 0.0))),
    }
}

enum Group {
    Arithmetic,
    Counting,
    Comparison,
    Logic,
}

impl Operator {
    fn group(self) -> Group {
        match self {
            Operator::Permutations | Operator::Combinations => Group::Counting,
            Operator::And | Operator::Or | Operator::Xor => Group::Logic,
            Operator::Equal
            | Operator::NotEqual
            | Operator::Greater
            | Operator::GreaterEqual
            | Operator::Less
            | Operator::LessEqual => Group::Comparison,
            _ => Group::Arithmetic,
        }
    }
}

fn arithmetic(op: Operator, left: f64, right: f64) -> Result<f64, CalcError> {
    match op {
        Operator::Add => Ok(left + right),
        Operator::Subtract => Ok(left - right),
        Operator::Multiply => Ok(left * right),
        Operator::Divide if right == 0.0 => fail(ErrorKind::DivideByZero),
        Operator::Divide => Ok(left / right),
        Operator::XRoot if left == 0.0 => fail(ErrorKind::DivideByZero),
        Operator::XRoot => power(right, 1.0 / left),
        _ => power(left, right),
    }
}

/// `n nPr r` and `n nCr r` for whole `n` and `r`; 0 when `r` exceeds `n`.
fn counting(op: Operator, n: f64, r: f64) -> Result<f64, CalcError> {
    let (n, r) = (whole(n, 0.0..=f64::MAX)?, whole(r, 0.0..=f64::MAX)?);
    if r > n {
        return Ok(0.0);
    }
    let permutations: f64 = (0..r as u64).map(|k| n - k as f64).product();
    Ok(match op {
        Operator::Permutations => permutations,
        _ => (permutations / (1..=r as u64).map(|k| k as f64).product::<f64>()).round(),
    })
}

fn compare(op: Operator, left: f64, right: f64) -> bool {
    match op {
        Operator::Equal => left == right,
        Operator::NotEqual => left != right,
        Operator::Greater => left > right,
        Operator::GreaterEqual => left >= right,
        Operator::Less => left < right,
        _ => left <= right,
    }
}

fn logic(op: Operator, left: bool, right: bool) -> bool {
    match op {
        Operator::And => left && right,
        Operator::Or => left || right,
        _ => left != right,
    }
}

/// 1 for true, 0 for false, as the calculator's tests return.
pub(super) fn truth(value: bool) -> f64 {
    if value { 1.0 } else { 0.0 }
}

pub(super) fn power(base: f64, exponent: f64) -> Result<f64, CalcError> {
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
    let numerator = (1..100)
        .step_by(2)
        .map(|denominator| exponent * f64::from(denominator))
        .find(|numerator| (numerator - numerator.round()).abs() < 1e-9);
    match numerator {
        Some(numerator) => {
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
