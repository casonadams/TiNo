use crate::token::Letter;

/// A parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(Number),
    Constant(Constant),
    Variable(Letter),
    Ans,
    Negate(Box<Expr>),
    Postfix(Postfix, Box<Expr>),
    Binary(BinaryOp, Box<Expr>, Box<Expr>),
    Function(Function, Box<Expr>),
}

/// A number literal: its value and how it was typed (`.5`, `2ᴇ⁻3`).
#[derive(Debug, Clone, PartialEq)]
pub struct Number {
    pub value: f64,
    pub mantissa: String,
    pub exponent: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constant {
    Pi,
    E,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Postfix {
    Square,
    Inverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    /// Multiplication written by juxtaposition (`2π`, `3(4)`).
    ImpliedMultiply,
    Divide,
    Power,
}

/// A function whose token opens its own parenthesis (`sin(`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Function {
    Sqrt,
    Sin,
    Cos,
    Tan,
    ArcSin,
    ArcCos,
    ArcTan,
    Log,
    Ln,
    TenPower,
    ExpPower,
}

/// A whole entry: an expression, stored into a variable when it ends in `→`.
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub expr: Expr,
    pub store: Option<Letter>,
}

/// Binding strength, loosest first: sums, products, negation, powers,
/// postfix operators, atoms. Implied multiplication binds as `×` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Sum,
    Product,
    Negation,
    Power,
    Postfix,
    Atom,
}

impl Expr {
    pub fn precedence(&self) -> Precedence {
        match self {
            Expr::Binary(BinaryOp::Add | BinaryOp::Subtract, ..) => Precedence::Sum,
            Expr::Binary(BinaryOp::Power, ..) => Precedence::Power,
            Expr::Binary(..) => Precedence::Product,
            Expr::Negate(_) => Precedence::Negation,
            Expr::Postfix(..) => Precedence::Postfix,
            _ => Precedence::Atom,
        }
    }
}
