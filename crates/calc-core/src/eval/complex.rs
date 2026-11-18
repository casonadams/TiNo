//! Complex numbers: arithmetic, powers and roots, logarithms, and MATH CPX.

use serde::{Deserialize, Serialize};

use super::{arithmetic::truth, fail, value::Value};
use crate::{
    error::{CalcError, ErrorKind},
    settings::Angle,
    vocabulary::{Function, Operator, PostfixOp},
};

type Result<T, E = CalcError> = std::result::Result<T, E>;

/// A part smaller than this fraction of the other is rounding, and dropped.
const NEGLIGIBLE: f64 = 1e-12;
/// Whole exponents up to this size power by repeated multiplication, exactly.
const LARGEST_WHOLE_EXPONENT: f64 = 1024.0;

/// A complex number `re + im·i`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

pub const I: Complex = Complex { re: 0.0, im: 1.0 };

impl Complex {
    pub const fn real(re: f64) -> Self {
        Complex { re, im: 0.0 }
    }

    fn polar(r: f64, theta: f64) -> Self {
        Complex {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    fn conj(self) -> Self {
        Complex {
            im: -self.im,
            ..self
        }
    }

    fn add(self, other: Complex) -> Self {
        Complex {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    fn scale(self, k: f64) -> Self {
        Complex {
            re: k * self.re,
            im: k * self.im,
        }
    }

    fn mul(self, other: Complex) -> Self {
        Complex {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    fn div(self, other: Complex) -> Result<Self> {
        let norm = other.re * other.re + other.im * other.im;
        if norm == 0.0 {
            return fail(ErrorKind::DivideByZero);
        }
        Ok(self.mul(other.conj()).scale(1.0 / norm))
    }

    fn exp(self) -> Self {
        Complex::polar(self.re.exp(), self.im)
    }

    fn ln(self) -> Result<Self> {
        if self.abs() == 0.0 {
            return fail(ErrorKind::Domain);
        }
        Ok(Complex {
            re: self.abs().ln(),
            im: self.arg(),
        })
    }

    /// The principal square root, its real part never negative.
    fn sqrt(self) -> Self {
        let r = self.abs();
        let im = ((r - self.re) / 2.0).sqrt();
        Complex {
            re: ((r + self.re) / 2.0).sqrt(),
            im: if self.im < 0.0 { -im } else { im },
        }
    }

    /// The principal power; whole exponents by repeated multiplication.
    fn pow(self, exponent: Complex) -> Result<Self> {
        let whole = exponent.im == 0.0
            && exponent.re.fract() == 0.0
            && exponent.re.abs() <= LARGEST_WHOLE_EXPONENT;
        if whole {
            return self.whole_power(exponent.re as i32);
        }
        if self.abs() == 0.0 {
            return match exponent.re > 0.0 {
                true => Ok(Complex::real(0.0)),
                false => fail(ErrorKind::Domain),
            };
        }
        Ok(exponent.mul(self.ln()?).exp())
    }

    fn whole_power(self, n: i32) -> Result<Self> {
        let product = (0..n.unsigned_abs()).fold(Complex::real(1.0), |acc, _| acc.mul(self));
        if n < 0 {
            return Complex::real(1.0).div(product);
        }
        Ok(product)
    }
}

/// A complex result as a value: a part negligible beside the other is
/// zero, and a number with no imaginary part is real.
pub fn simplified(z: Complex) -> Value {
    let negligible = |part: f64, other: f64| part.abs() <= NEGLIGIBLE * other.abs();
    let re = if negligible(z.re, z.im) { 0.0 } else { z.re };
    match negligible(z.im, z.re) {
        true => Value::Real(re),
        false => Value::Complex(Complex { re, im: z.im }),
    }
}

/// An operator between complex numbers: arithmetic, powers and roots, and
/// `=` and `≠`; the other tests and logic take real numbers only.
pub(super) fn binary(op: Operator, (a, b): (Complex, Complex)) -> Result<Value> {
    let result = match op {
        Operator::Add => a.add(b),
        Operator::Subtract => a.add(b.scale(-1.0)),
        Operator::Multiply => a.mul(b),
        Operator::Divide => a.div(b)?,
        Operator::Power => a.pow(b)?,
        Operator::XRoot => b.pow(Complex::real(1.0).div(a)?)?,
        Operator::Equal | Operator::NotEqual => {
            return Ok(Value::Real(truth((a == b) == (op == Operator::Equal))));
        }
        _ => return fail(ErrorKind::DataType),
    };
    Ok(simplified(result))
}

/// `²`, `³` and `⁻¹` of a complex number.
pub(super) fn postfix(op: PostfixOp, z: Complex) -> Result<Value> {
    let result = match op {
        PostfixOp::Square => z.mul(z),
        PostfixOp::Cube => z.mul(z).mul(z),
        PostfixOp::Inverse => Complex::real(1.0).div(z)?,
        PostfixOp::Factorial | PostfixOp::Transpose => return fail(ErrorKind::DataType),
    };
    Ok(simplified(result))
}

/// A function of a complex number: roots, logarithms and powers of e and
/// 10, and MATH CPX's; `None` for functions of real numbers only.
pub(super) fn function(function: Function, z: Complex, angle: Angle) -> Option<Result<Value>> {
    let result = match function {
        Function::Sqrt => Ok(z.sqrt()),
        Function::Ln => z.ln(),
        Function::Log => z.ln().map(|ln| ln.scale(1.0 / std::f64::consts::LN_10)),
        Function::ExpPower => Ok(z.exp()),
        Function::TenPower => Ok(z.scale(std::f64::consts::LN_10).exp()),
        Function::Conj => Ok(z.conj()),
        Function::Abs => Ok(Complex::real(z.abs())),
        Function::RealPart => Ok(Complex::real(z.re)),
        Function::ImagPart => Ok(Complex::real(z.im)),
        Function::Angle => Ok(Complex::real(in_units(z.arg(), angle))),
        _ => return None,
    };
    Some(result.map(simplified))
}

/// `radians` in `angle` units.
pub fn in_units(radians: f64, angle: Angle) -> f64 {
    match angle {
        Angle::Radian => radians,
        Angle::Degree => radians.to_degrees(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rounded(value: Value) -> Value {
        let round = |x: f64| (x * 1e9).round() / 1e9;
        match value {
            Value::Complex(z) => Value::Complex(Complex {
                re: round(z.re),
                im: round(z.im),
            }),
            Value::Real(x) => Value::Real(round(x)),
            other => other,
        }
    }

    #[test]
    fn i_squared_is_minus_one_and_e_to_i_pi_is_too() {
        assert_eq!(postfix(PostfixOp::Square, I), Ok(Value::Real(-1.0)));
        let i_pi = I.scale(std::f64::consts::PI);
        assert_eq!(simplified(i_pi.exp()), Value::Real(-1.0));
    }

    #[test]
    fn arithmetic_follows_the_rules_of_i() {
        let a = Complex { re: 3.0, im: 4.0 };
        let b = Complex { re: 1.0, im: -2.0 };
        let cases = [
            (Operator::Add, Complex { re: 4.0, im: 2.0 }),
            (Operator::Multiply, Complex { re: 11.0, im: -2.0 }),
            (Operator::Divide, Complex { re: -1.0, im: 2.0 }),
        ];
        for (op, expected) in cases {
            assert_eq!(binary(op, (a, b)), Ok(Value::Complex(expected)), "{op:?}");
        }
        let zero = Complex::real(0.0);
        assert_eq!(
            binary(Operator::Divide, (a, zero)).unwrap_err().kind,
            ErrorKind::DivideByZero
        );
    }

    #[test]
    fn roots_and_logarithms_of_negative_numbers() {
        let minus_four = Complex::real(-4.0);
        let root = function(Function::Sqrt, minus_four, Angle::Radian);
        assert_eq!(root, Some(Ok(Value::Complex(Complex { re: 0.0, im: 2.0 }))));
        let ln = function(Function::Ln, Complex::real(-1.0), Angle::Radian).unwrap();
        let pi_i = Complex {
            re: 0.0,
            im: std::f64::consts::PI,
        };
        assert_eq!(ln, Ok(Value::Complex(pi_i)));
        let half = binary(Operator::Power, (minus_four, Complex::real(0.5))).unwrap();
        assert_eq!(rounded(half), Value::Complex(Complex { re: 0.0, im: 2.0 }));
    }

    #[test]
    fn cpx_functions_take_a_number_apart() {
        let z = Complex { re: 3.0, im: 4.0 };
        let apply = |f| function(f, z, Angle::Degree).unwrap().unwrap();
        assert_eq!(apply(Function::Abs), Value::Real(5.0));
        assert_eq!(apply(Function::ImagPart), Value::Real(4.0));
        assert_eq!(
            apply(Function::Conj),
            Value::Complex(Complex { re: 3.0, im: -4.0 })
        );
        assert_eq!(rounded(apply(Function::Angle)), Value::Real(53.130102354));
    }
}
