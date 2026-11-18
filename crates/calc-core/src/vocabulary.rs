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
    /// `fMin(expression, variable, lower, upper[, tolerance])`.
    FMin,
    FMax,
    /// `nDeriv(expression, variable, value[, ε])`.
    NDeriv,
    /// `fnInt(expression, variable, lower, upper[, tolerance])`.
    FnInt,
    /// `Σ(expression, variable, start, end)`.
    Summation,
    /// The LIST functions: length, sums, statistics, running sums and
    /// differences, joining, and `seq(expression, variable, start, end[, step])`.
    Dim,
    Sum,
    Prod,
    Mean,
    Median,
    StdDev,
    Variance,
    CumSum,
    DeltaList,
    Augment,
    Seq,
    /// STAT CALC's commands, written without a parenthesis and taking lists:
    /// `1-Var Stats [list]` and `LinReg(ax+b) [xlist, ylist]`.
    OneVarStats,
    LinReg,
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
            Function::FMin => "fMin(",
            Function::FMax => "fMax(",
            Function::NDeriv => "nDeriv(",
            Function::FnInt => "fnInt(",
            Function::Summation => "Σ(",
            Function::Dim => "dim(",
            Function::Sum => "sum(",
            Function::Prod => "prod(",
            Function::Mean => "mean(",
            Function::Median => "median(",
            Function::StdDev => "stdDev(",
            Function::Variance => "variance(",
            Function::CumSum => "cumSum(",
            Function::DeltaList => "ΔList(",
            Function::Augment => "augment(",
            Function::Seq => "seq(",
            Function::OneVarStats => "1-Var Stats ",
            Function::LinReg => "LinReg(ax+b) ",
        }
    }

    /// How many arguments the function takes: at least, at most.
    pub fn arity(self) -> (usize, usize) {
        match self {
            Function::Round | Function::Min | Function::Max => (1, 2),
            Function::LogBase
            | Function::Lcm
            | Function::Gcd
            | Function::Remainder
            | Function::RandInt
            | Function::Augment => (2, 2),
            Function::NDeriv => (3, 4),
            Function::Summation => (4, 4),
            Function::FMin | Function::FMax | Function::FnInt | Function::Seq => (4, 5),
            Function::OneVarStats => (0, 1),
            Function::LinReg => (0, 2),
            _ => (1, 1),
        }
    }

    /// Whether the function's first argument is an expression in the
    /// variable its second argument names, rather than a value.
    pub fn binds_variable(self) -> bool {
        matches!(
            self,
            Function::FMin
                | Function::FMax
                | Function::NDeriv
                | Function::FnInt
                | Function::Summation
                | Function::Seq
        )
    }

    /// Whether it is a command: a whole entry of its own that shows a report
    /// instead of a value.
    pub fn is_command(self) -> bool {
        matches!(self, Function::OneVarStats | Function::LinReg)
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
