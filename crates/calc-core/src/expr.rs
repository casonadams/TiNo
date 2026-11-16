use crate::{
    token::Letter,
    vocabulary::{DisplayAs, Function, Operator, PostfixOp},
};

/// A parsed expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(Number),
    Constant(Constant),
    Variable(Letter),
    Ans,
    Rand,
    Negate(Box<Expr>),
    Postfix(PostfixOp, Box<Expr>),
    Binary(Operator, Box<Expr>, Box<Expr>),
    /// Multiplication written by juxtaposition (`2π`, `3(4)`).
    ImpliedMultiply(Box<Expr>, Box<Expr>),
    Call(Function, Vec<Expr>),
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

/// A whole entry: an expression, optionally converted for display (`►Frac`)
/// and stored into a variable (`→A`).
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub expr: Expr,
    pub display: Option<DisplayAs>,
    pub store: Option<Letter>,
}

/// Binding strength, loosest first, in the calculator's order: `or` and
/// `xor`, `and`, relations, sums, products (implied multiplication binds as
/// `×` does), `nPr` and `nCr`, negation, powers and roots, postfix
/// operators, atoms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    Either,
    Both,
    Relation,
    Sum,
    Product,
    Combination,
    Negation,
    Power,
    Postfix,
    Atom,
}

impl Operator {
    pub fn precedence(self) -> Precedence {
        match self {
            Operator::Or | Operator::Xor => Precedence::Either,
            Operator::And => Precedence::Both,
            Operator::Equal
            | Operator::NotEqual
            | Operator::Greater
            | Operator::GreaterEqual
            | Operator::Less
            | Operator::LessEqual => Precedence::Relation,
            Operator::Add | Operator::Subtract => Precedence::Sum,
            Operator::Multiply | Operator::Divide => Precedence::Product,
            Operator::Permutations | Operator::Combinations => Precedence::Combination,
            Operator::Power | Operator::XRoot => Precedence::Power,
        }
    }
}

impl Expr {
    pub fn precedence(&self) -> Precedence {
        match self {
            Expr::Binary(op, ..) => op.precedence(),
            Expr::ImpliedMultiply(..) => Precedence::Product,
            Expr::Negate(_) => Precedence::Negation,
            Expr::Postfix(..) => Precedence::Postfix,
            _ => Precedence::Atom,
        }
    }
}
