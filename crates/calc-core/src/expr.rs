use crate::{
    table::TableField,
    token::{Letter, ListName},
    vocabulary::{DisplayAs, Function, Operator, PostfixOp},
    window_editor::WindowField,
};

/// A parsed expression. Operators and calls keep the position of their
/// token, where an error evaluating them points.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(Number),
    Constant(Constant),
    Variable(Letter),
    Ans,
    Rand,
    /// A WINDOW setting read as a variable (`Xmin`).
    Window(WindowField),
    /// A TBLSET setting read as a variable (`ΔTbl`).
    Table(TableField),
    /// A Y= function by slot: at the current `X`, or at its argument (`Y₁(3)`).
    YVar(usize, Option<Box<Expr>>),
    /// A list written out: `{1,2,3}`.
    List(Vec<Expr>),
    /// A stored list by name, or its element at the index (`L₁(2)`).
    ListName(ListName, Option<Box<Expr>>),
    Negate(Box<Expr>),
    Postfix(PostfixOp, Box<Expr>, usize),
    Binary(Operator, Box<Expr>, Box<Expr>, usize),
    /// Multiplication written by juxtaposition (`2π`, `3(4)`).
    ImpliedMultiply(Box<Expr>, Box<Expr>),
    Call(Function, Vec<Expr>, usize),
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
/// and stored (`→A`, `→L₁`).
#[derive(Debug, Clone, PartialEq)]
pub struct Statement {
    pub expr: Expr,
    pub display: Option<DisplayAs>,
    pub store: Option<StoreTarget>,
}

/// Where `→` stores: a variable takes a number, a list name a list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreTarget {
    Variable(Letter),
    List(ListName),
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
