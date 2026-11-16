use crate::expr::{BinaryOp, Constant, Expr, Function, Number, Postfix, Precedence, Statement};

/// The entry as TeX, for typesetting in history: fractions for `÷`, raised
/// exponents, radicals, and parentheses only where the meaning needs them.
pub fn statement(statement: &Statement) -> String {
    let expr = latex(&statement.expr);
    match statement.store {
        Some(letter) => format!("{expr}\\to {}", letter_tex(letter.symbol())),
        None => expr,
    }
}

fn latex(expr: &Expr) -> String {
    match expr {
        Expr::Number(number) => number_tex(number),
        Expr::Constant(Constant::Pi) => "\\pi".to_owned(),
        Expr::Constant(Constant::E) => "e".to_owned(),
        Expr::Variable(letter) => letter_tex(letter.symbol()),
        Expr::Ans => "\\mathrm{Ans}".to_owned(),
        Expr::Negate(operand) => format!("-{}", operand_tex(operand, Precedence::Negation)),
        Expr::Postfix(op, operand) => postfix_tex(*op, operand),
        Expr::Binary(op, left, right) => binary_tex(*op, left, right),
        Expr::Function(function, argument) => function_tex(*function, &latex(argument)),
    }
}

fn number_tex(number: &Number) -> String {
    match (&number.mantissa[..], &number.exponent) {
        (mantissa, None) => mantissa.to_owned(),
        ("", Some(exponent)) => format!("10^{{{exponent}}}"),
        (mantissa, Some(exponent)) => format!("{mantissa}\\times10^{{{exponent}}}"),
    }
}

fn letter_tex(symbol: &str) -> String {
    if symbol == "θ" {
        "\\theta".to_owned()
    } else {
        symbol.to_owned()
    }
}

/// `expr`, parenthesized when it binds looser than `context` requires.
fn operand_tex(expr: &Expr, context: Precedence) -> String {
    let tex = latex(expr);
    if expr.precedence() < context {
        format!("\\left({tex}\\right)")
    } else {
        tex
    }
}

fn postfix_tex(op: Postfix, operand: &Expr) -> String {
    let base = operand_tex(operand, Precedence::Atom);
    match op {
        Postfix::Square => format!("{{{base}}}^{{2}}"),
        Postfix::Inverse => format!("{{{base}}}^{{-1}}"),
    }
}

fn binary_tex(op: BinaryOp, left: &Expr, right: &Expr) -> String {
    match op {
        BinaryOp::Add => format!("{}+{}", latex(left), sum_operand(right, Precedence::Sum)),
        BinaryOp::Subtract => format!(
            "{}-{}",
            latex(left),
            sum_operand(right, Precedence::Product)
        ),
        BinaryOp::Multiply => product_tex(left, right, "\\times "),
        BinaryOp::ImpliedMultiply if starts_with_digit(right) => {
            product_tex(left, right, "\\cdot ")
        }
        BinaryOp::ImpliedMultiply => product_tex(left, right, ""),
        BinaryOp::Divide => format!("\\frac{{{}}}{{{}}}", latex(left), latex(right)),
        BinaryOp::Power => format!(
            "{{{}}}^{{{}}}",
            operand_tex(left, Precedence::Postfix),
            latex(right)
        ),
    }
}

fn product_tex(left: &Expr, right: &Expr, sign: &str) -> String {
    format!(
        "{}{sign}{}",
        operand_tex(left, Precedence::Product),
        operand_tex(right, Precedence::Power)
    )
}

/// A sum's right operand: a negation keeps its parentheses so `2+⁻3` does
/// not read as `2+-3`.
fn sum_operand(right: &Expr, context: Precedence) -> String {
    match right {
        Expr::Negate(_) => format!("\\left({}\\right)", latex(right)),
        _ => operand_tex(right, context),
    }
}

/// Whether `expr` is written starting with a digit, so juxtaposing it after
/// another factor would read as one number.
fn starts_with_digit(expr: &Expr) -> bool {
    match expr {
        Expr::Number(_) => true,
        Expr::Postfix(_, operand) => starts_with_digit(operand),
        Expr::Binary(BinaryOp::Power, base, _) => starts_with_digit(base),
        _ => false,
    }
}

fn function_tex(function: Function, argument: &str) -> String {
    let name = match function {
        Function::Sqrt => return format!("\\sqrt{{{argument}}}"),
        Function::TenPower => return format!("10^{{{argument}}}"),
        Function::ExpPower => return format!("e^{{{argument}}}"),
        Function::Sin => "\\sin",
        Function::Cos => "\\cos",
        Function::Tan => "\\tan",
        Function::ArcSin => "\\sin^{-1}",
        Function::ArcCos => "\\cos^{-1}",
        Function::ArcTan => "\\tan^{-1}",
        Function::Log => "\\log",
        Function::Ln => "\\ln",
    };
    format!("{name}\\left({argument}\\right)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse, tests::tokens};

    #[test]
    fn typesets_entries() {
        let cases = [
            ("2+3*4", "2+3\\times 4"),
            ("1/2p", "\\frac{1}{2}\\pi"),
            ("2(3+4", "2\\left(3+4\\right)"),
            ("(2)(3)", "2\\cdot 3"),
            ("_2²", "-{2}^{2}"),
            ("(_2)²", "{\\left(-2\\right)}^{2}"),
            ("2^3^2", "{\\left({2}^{3}\\right)}^{2}"),
            ("2_3", "2\\left(-3\\right)"),
            ("2+_3", "2+\\left(-3\\right)"),
            ("2-3*4", "2-3\\times 4"),
            ("r(16", "\\sqrt{16}"),
            ("s(p/2", "\\sin\\left(\\frac{\\pi}{2}\\right)"),
            ("2-(3-4", "2-\\left(3-4\\right)"),
            ("1.5E_2", "1.5\\times10^{-2}"),
            ("5>A", "5\\to A"),
            ("2a", "2\\mathrm{Ans}"),
        ];
        for (spelling, tex) in cases {
            assert_eq!(
                statement(&parse(&tokens(spelling)).unwrap()),
                tex,
                "{spelling}"
            );
        }
    }
}
