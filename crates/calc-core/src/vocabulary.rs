//! The operators and functions entries are written with, shared by tokens
//! (what is typed) and expressions (what is parsed).

use serde::{Deserialize, Serialize};

/// An operator written between two operands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    /// `n ˣ√ x`: the `n`th root of `x`.
    XRoot,
    Permutations,
    Combinations,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
    Xor,
}

impl Operator {
    pub const fn text(self) -> &'static str {
        match self {
            Operator::Add => "+",
            Operator::Subtract => "−",
            Operator::Multiply => "×",
            Operator::Divide => "÷",
            Operator::Power => "^",
            Operator::XRoot => "ˣ√",
            Operator::Permutations => " nPr ",
            Operator::Combinations => " nCr ",
            Operator::Equal => "=",
            Operator::NotEqual => "≠",
            Operator::Greater => ">",
            Operator::GreaterEqual => "≥",
            Operator::Less => "<",
            Operator::LessEqual => "≤",
            Operator::And => " and ",
            Operator::Or => " or ",
            Operator::Xor => " xor ",
        }
    }
}

/// An operator written after its operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PostfixOp {
    Square,
    Cube,
    Inverse,
    Factorial,
}

impl PostfixOp {
    pub const fn text(self) -> &'static str {
        match self {
            PostfixOp::Square => "²",
            PostfixOp::Cube => "³",
            PostfixOp::Inverse => "⁻¹",
            PostfixOp::Factorial => "!",
        }
    }
}

/// A function whose token opens its own parenthesis (`sin(`), taking its
/// arguments separated by commas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Function {
    Sqrt,
    CubeRoot,
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
    LogBase,
    Abs,
    Round,
    IPart,
    FPart,
    Int,
    Min,
    Max,
    Lcm,
    Gcd,
    Remainder,
    RandInt,
    Not,
}

impl Function {
    pub const fn text(self) -> &'static str {
        match self {
            Function::Sqrt => "√(",
            Function::CubeRoot => "³√(",
            Function::Sin => "sin(",
            Function::Cos => "cos(",
            Function::Tan => "tan(",
            Function::ArcSin => "sin⁻¹(",
            Function::ArcCos => "cos⁻¹(",
            Function::ArcTan => "tan⁻¹(",
            Function::Log => "log(",
            Function::Ln => "ln(",
            Function::TenPower => "10^(",
            Function::ExpPower => "e^(",
            Function::LogBase => "logBASE(",
            Function::Abs => "abs(",
            Function::Round => "round(",
            Function::IPart => "iPart(",
            Function::FPart => "fPart(",
            Function::Int => "int(",
            Function::Min => "min(",
            Function::Max => "max(",
            Function::Lcm => "lcm(",
            Function::Gcd => "gcd(",
            Function::Remainder => "remainder(",
            Function::RandInt => "randInt(",
            Function::Not => "not(",
        }
    }

    /// How many arguments the function takes: at least, at most.
    pub fn arity(self) -> (usize, usize) {
        match self {
            Function::Round => (1, 2),
            Function::LogBase
            | Function::Min
            | Function::Max
            | Function::Lcm
            | Function::Gcd
            | Function::Remainder
            | Function::RandInt => (2, 2),
            _ => (1, 1),
        }
    }
}

/// How the result of an entry is written, chosen by ending it with
/// `►Frac` or `►Dec`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DisplayAs {
    Fraction,
    Decimal,
}

impl DisplayAs {
    pub const fn text(self) -> &'static str {
        match self {
            DisplayAs::Fraction => "►Frac",
            DisplayAs::Decimal => "►Dec",
        }
    }
}
