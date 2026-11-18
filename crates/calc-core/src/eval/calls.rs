use super::{
    Evaluator, arithmetic, fail,
    lists::list_function,
    value::{Value, broadcast},
    whole,
};
use crate::{
    error::{CalcError, ErrorKind},
    expr::Expr,
    trig,
    vocabulary::Function,
};

/// Decimal places `round(` takes at most.
const MOST_PLACES: f64 = 9.0;
/// Integers `lcm(`, `gcd(` and `remainder(` take are below this.
const LARGEST_INTEGER: f64 = 1e12;

impl Evaluator<'_> {
    /// `function` applied to `arguments`, element by element where they are
    /// lists; an argument count it does not take is `ERR:ARGUMENT`.
    pub(super) fn call(&self, function: Function, arguments: &[Expr]) -> Result<Value, CalcError> {
        let (least, most) = function.arity();
        if function.is_command() {
            return fail(ErrorKind::Syntax);
        }
        if !(least..=most).contains(&arguments.len()) {
            return fail(ErrorKind::Argument);
        }
        if function.binds_variable() {
            return self.calculus(function, arguments);
        }
        let values = arguments
            .iter()
            .map(|argument| self.value(argument))
            .collect::<Result<Vec<_>, _>>()?;
        if let Some(result) = list_function(function, &values) {
            return result;
        }
        broadcast(&values, |reals| match *reals {
            [x] if least == 1 => self.unary(function, x),
            [a, b] => self.binary(function, (a, b)),
            _ => fail(ErrorKind::Argument),
        })
    }

    fn unary(&self, function: Function, x: f64) -> Result<f64, CalcError> {
        let angle = self.context.angle;
        match function {
            Function::Sin | Function::Cos | Function::Tan => trig::forward(function, x, angle),
            Function::ArcSin | Function::ArcCos | Function::ArcTan => {
                trig::inverse(function, x, angle)
            }
            Function::Round => Ok(significant(x)),
            _ => elementary(function, x),
        }
    }

    fn binary(&self, function: Function, (a, b): (f64, f64)) -> Result<f64, CalcError> {
        match function {
            Function::Round => round(a, b),
            Function::LogBase => log_base(a, b),
            Function::Min => Ok(a.min(b)),
            Function::Max => Ok(a.max(b)),
            Function::RandInt => self.random_integer(a, b),
            _ => integer_pair(function, a, b),
        }
    }

    /// `randInt(low, high)`: a whole number from `low` to `high`, either order.
    fn random_integer(&self, a: f64, b: f64) -> Result<f64, CalcError> {
        let (low, high) = (
            whole(a.min(b), -LARGEST_INTEGER..=LARGEST_INTEGER)?,
            whole(a.max(b), -LARGEST_INTEGER..=LARGEST_INTEGER)?,
        );
        Ok(low + (self.context.memory.random() * (high - low + 1.0)).floor())
    }
}

fn elementary(function: Function, x: f64) -> Result<f64, CalcError> {
    match function {
        Function::Sqrt if x < 0.0 => fail(ErrorKind::NonReal),
        Function::Sqrt => Ok(x.sqrt()),
        Function::CubeRoot => Ok(x.cbrt()),
        Function::Log | Function::Ln => logarithm(function, x),
        Function::TenPower => Ok(10f64.powf(x)),
        Function::ExpPower => Ok(x.exp()),
        Function::Abs => Ok(x.abs()),
        Function::IPart => Ok(x.trunc()),
        Function::FPart => Ok(x - x.trunc()),
        Function::Int => Ok(x.floor()),
        _ => Ok(arithmetic::truth(x == 0.0)),
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

/// `logBASE(x, b)`: the base-`b` logarithm of `x`.
fn log_base(x: f64, base: f64) -> Result<f64, CalcError> {
    let numerator = logarithm(Function::Ln, x)?;
    match logarithm(Function::Ln, base)? {
        0.0 => fail(ErrorKind::DivideByZero),
        denominator => Ok(numerator / denominator),
    }
}

/// `round(x)`: `x` to the 10 significant digits the calculator shows.
fn significant(x: f64) -> f64 {
    format!("{x:.9e}").parse().unwrap_or(x)
}

/// `round(x, places)`: `x` to 0 to 9 places, halves away from zero.
fn round(x: f64, places: f64) -> Result<f64, CalcError> {
    let scale = 10f64.powf(whole(places, 0.0..=MOST_PLACES)?);
    Ok((x * scale).round() / scale)
}

/// `lcm(`, `gcd(` and `remainder(` of two whole numbers.
fn integer_pair(function: Function, a: f64, b: f64) -> Result<f64, CalcError> {
    let range = -LARGEST_INTEGER..=LARGEST_INTEGER;
    let (a, b) = (whole(a, range.clone())?, whole(b, range)?);
    match function {
        Function::Remainder if b == 0.0 => fail(ErrorKind::DivideByZero),
        Function::Remainder => Ok(a % b),
        Function::Gcd => Ok(gcd(a.abs(), b.abs())),
        _ if a == 0.0 || b == 0.0 => Ok(0.0),
        _ => Ok((a * b).abs() / gcd(a.abs(), b.abs())),
    }
}

fn gcd(mut a: f64, mut b: f64) -> f64 {
    while b != 0.0 {
        (a, b) = (b, a % b);
    }
    a
}
