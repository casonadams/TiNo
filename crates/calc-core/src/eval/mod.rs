//! Evaluating expressions over real numbers, lists and matrices.

mod arithmetic;
mod calculus;
mod calls;
mod lists;
mod matrix;
mod value;

pub use matrix::{LARGEST_DIMENSION, Matrix, dimension};
pub use value::Value;

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
    token::{Letter, ListName, MatrixName},
    vocabulary::{Operator, PostfixOp},
    window::Window,
};

/// Results at or beyond this magnitude overflow, as on the calculator.
const OVERFLOW: f64 = 1e100;
/// Results smaller than this in magnitude are zero.
const UNDERFLOW: f64 = 1e-99;

/// The values expressions read: variables `A`-`Z` and `θ`, all 0 at first;
/// the lists `L₁`-`L₆`, empty at first; the matrices `[A]`-`[J]`, undefined
/// (empty) at first; `Ans`; and the state `rand` draws from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    variables: [f64; 27],
    #[serde(default)]
    lists: [Vec<f64>; 6],
    #[serde(default)]
    matrices: [Matrix; 10],
    ans: Value,
    #[serde(default)]
    random: Cell<u64>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            variables: [0.0; 27],
            lists: Default::default(),
            matrices: Default::default(),
            ans: Value::Real(0.0),
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

    pub fn list(&self, name: ListName) -> &[f64] {
        &self.lists[name as usize]
    }

    pub fn set_list(&mut self, name: ListName, list: Vec<f64>) {
        self.lists[name as usize] = list;
    }

    /// A matrix by name; empty while undefined.
    pub fn matrix(&self, name: MatrixName) -> &Matrix {
        &self.matrices[name as usize]
    }

    pub fn set_matrix(&mut self, name: MatrixName, matrix: Matrix) {
        self.matrices[name as usize] = matrix;
    }

    pub fn ans(&self) -> &Value {
        &self.ans
    }

    pub fn set_ans(&mut self, value: Value) {
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
    /// `expr`, trigonometry in the context's angle units.
    pub fn evaluate(&self, expr: &Expr) -> Result<Value, CalcError> {
        Evaluator::new(self).value(expr)
    }

    /// `expr` as a real number; `ERR:DATA TYPE` when it is a list.
    pub fn evaluate_real(&self, expr: &Expr) -> Result<f64, CalcError> {
        Evaluator::new(self).real(expr)
    }

    /// `expr` at `X = x`, or `None` where it is undefined.
    pub fn y_at(&self, expr: &Expr, x: f64) -> Option<f64> {
        Evaluator::new(self).with(Letter::X, x).real(expr).ok()
    }

    /// Y= function `slot` at `X = x`.
    pub fn function_at(&self, slot: usize, x: f64) -> Result<f64, CalcError> {
        Evaluator::new(self)
            .with(Letter::X, x)
            .y_var(slot, None)?
            .real()
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

    fn value(&self, expr: &Expr) -> Result<Value, CalcError> {
        let memory = self.context.memory;
        let real = |x: f64| Ok(Value::Real(x));
        let value = match expr {
            Expr::Number(number) => real(number.value),
            Expr::Constant(Constant::Pi) => real(PI),
            Expr::Constant(Constant::E) => real(E),
            Expr::Variable(letter) => {
                real(self.bound[*letter as usize].unwrap_or(memory.get(*letter)))
            }
            Expr::Ans => Ok(memory.ans().clone()),
            Expr::Rand => real(memory.random()),
            Expr::Window(field) => real(field.get(self.context.window)),
            Expr::Table(field) => real(field.get(self.context.table)),
            Expr::List(elements) => elements
                .iter()
                .map(|element| self.real(element))
                .collect::<Result<_, _>>()
                .map(Value::List),
            Expr::ListName(name, index) => self.list_name(*name, index.as_deref()),
            Expr::Matrix(rows) => self.matrix(rows),
            Expr::MatrixName(name, index) => self.matrix_name(*name, index.as_deref()),
            Expr::YVar(slot, argument) => self.y_var(*slot, argument.as_deref()),
            Expr::Negate(operand) => self.value(operand)?.map(|x| Ok(-x)),
            Expr::ImpliedMultiply(left, right) => {
                operate(Operator::Multiply, (self.value(left)?, self.value(right)?))
            }
            Expr::Binary(op, left, right, at) => {
                located(operate(*op, (self.value(left)?, self.value(right)?)), *at)
            }
            Expr::Postfix(op, operand, at) => located(postfix(*op, self.value(operand)?), *at),
            Expr::Call(function, arguments, at) => located(self.call(*function, arguments), *at),
        };
        value?.map(in_range)
    }

    fn real(&self, expr: &Expr) -> Result<f64, CalcError> {
        self.value(expr)?.real()
    }

    /// A list by name, or its element at `index` (`L₁(2)`, counting from 1).
    fn list_name(&self, name: ListName, index: Option<&Expr>) -> Result<Value, CalcError> {
        let list = self.context.memory.list(name);
        if list.is_empty() {
            return fail(ErrorKind::InvalidDim);
        }
        let Some(index) = index else {
            return Ok(Value::List(list.to_vec()));
        };
        let position = whole(self.real(index)?, 1.0..=list.len() as f64)
            .map_err(|_| CalcError::new(ErrorKind::InvalidDim))?;
        Ok(Value::Real(list[position as usize - 1]))
    }

    /// A matrix written out: rows of real numbers, all the same length.
    fn matrix(&self, rows: &[Vec<Expr>]) -> Result<Value, CalcError> {
        let rows: Matrix = rows
            .iter()
            .map(|row| row.iter().map(|element| self.real(element)).collect())
            .collect::<Result<_, _>>()?;
        if rows.iter().any(|row| row.len() != rows[0].len()) {
            return fail(ErrorKind::InvalidDim);
        }
        Ok(Value::Matrix(rows))
    }

    /// A matrix by name, or its element at a row and column (`[A](2,1)`).
    fn matrix_name(
        &self,
        name: MatrixName,
        index: Option<&(Expr, Expr)>,
    ) -> Result<Value, CalcError> {
        let matrix = self.context.memory.matrix(name);
        if matrix.is_empty() {
            return fail(ErrorKind::Undefined);
        }
        let Some((row, column)) = index else {
            return Ok(Value::Matrix(matrix.clone()));
        };
        let place = |expr: &Expr, count: usize| {
            whole(self.real(expr)?, 1.0..=count as f64)
                .map(|n| n as usize - 1)
                .map_err(|_| CalcError::new(ErrorKind::InvalidDim))
        };
        let row = place(row, matrix.len())?;
        Ok(Value::Real(matrix[row][place(column, matrix[0].len())?]))
    }

    /// Y= function `slot` at the current `X`, or at `argument` (each of its
    /// elements when it is a list). Errors in it have no place in the entry
    /// reading it.
    fn y_var(&self, slot: usize, argument: Option<&Expr>) -> Result<Value, CalcError> {
        let bit = 1 << slot;
        if self.active & bit != 0 {
            return fail(ErrorKind::Invalid);
        }
        let function = match self.context.functions.get(slot) {
            Some(Ok(expr)) => expr,
            Some(Err(error)) => return fail(error.kind),
            None => return fail(ErrorKind::Undefined),
        };
        let inner = Evaluator {
            active: self.active | bit,
            ..*self
        };
        let result = match argument {
            None => inner.real(function).map(Value::Real),
            Some(argument) => self
                .value(argument)?
                .map(|x| inner.with(Letter::X, x).real(function)),
        };
        result.map_err(|error| CalcError::new(error.kind))
    }
}

/// An infix operator: element by element over reals and lists, as matrix
/// arithmetic when either side is a matrix.
fn operate(op: Operator, (left, right): (Value, Value)) -> Result<Value, CalcError> {
    if matches!(left, Value::Matrix(_)) || matches!(right, Value::Matrix(_)) {
        return matrix::binary(op, left, right);
    }
    left.zip(right, |a, b| arithmetic::binary(op, a, b))
}

fn postfix(op: PostfixOp, operand: Value) -> Result<Value, CalcError> {
    match operand {
        Value::Matrix(m) => matrix::postfix(op, &m).map(Value::Matrix),
        value => value.map(|x| arithmetic::postfix(op, x)),
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

/// An operation's result, its error placed at the operation's token.
fn located(result: Result<Value, CalcError>, at: usize) -> Result<Value, CalcError> {
    result
        .and_then(|value| value.map(in_range))
        .map_err(|error| error.or_at(at))
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
