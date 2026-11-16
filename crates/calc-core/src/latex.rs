use crate::{
    expr::{Constant, Expr, Number, Precedence, Statement},
    field_editor::Field,
    functions::FUNCTION_COUNT,
    vocabulary::{DisplayAs, Function, Operator, PostfixOp},
};

/// The entry as TeX, for typesetting in history: fractions for `÷`, raised
/// exponents, radicals, and parentheses only where the meaning needs them.
pub fn statement(statement: &Statement) -> String {
    let mut tex = latex(&statement.expr);
    if let Some(display) = statement.display {
        tex.push_str(match display {
            DisplayAs::Fraction => "\\blacktriangleright\\mathrm{Frac}",
            DisplayAs::Decimal => "\\blacktriangleright\\mathrm{Dec}",
        });
    }
    if let Some(letter) = statement.store {
        tex.push_str(&format!("\\to {}", letter_tex(letter.symbol())));
    }
    tex
}

fn latex(expr: &Expr) -> String {
    match expr {
        Expr::Number(number) => number_tex(number),
        Expr::Constant(Constant::Pi) => "\\pi".to_owned(),
        Expr::Constant(Constant::E) => "e".to_owned(),
        Expr::Variable(letter) => letter_tex(letter.symbol()),
        Expr::Ans => "\\mathrm{Ans}".to_owned(),
        Expr::Rand => "\\mathrm{rand}".to_owned(),
        Expr::Window(field) => name_tex(field.label()),
        Expr::Table(field) => name_tex(field.label()),
        Expr::YVar(slot, argument) => y_var_tex(*slot, argument.as_deref()),
        Expr::Negate(operand) => format!("-{}", operand_tex(operand, Precedence::Negation)),
        Expr::Postfix(op, operand, _) => postfix_tex(*op, operand),
        Expr::Binary(op, left, right, _) => binary_tex(*op, left, right),
        Expr::ImpliedMultiply(left, right) => implied_tex(left, right),
        Expr::Call(function, arguments, _) => call_tex(*function, arguments),
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

/// A system variable's name upright (`Xmin`), `Δ` as the Greek letter.
fn name_tex(label: &str) -> String {
    match label.strip_prefix('Δ') {
        Some(rest) => format!("\\Delta\\mathrm{{{rest}}}"),
        None => format!("\\mathrm{{{label}}}"),
    }
}

fn y_var_tex(slot: usize, argument: Option<&Expr>) -> String {
    let name = format!("Y_{{{}}}", (slot + 1) % FUNCTION_COUNT);
    match argument {
        Some(argument) => format!("{name}\\left({}\\right)", latex(argument)),
        None => name,
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

fn postfix_tex(op: PostfixOp, operand: &Expr) -> String {
    let base = operand_tex(operand, Precedence::Atom);
    match op {
        PostfixOp::Square => format!("{{{base}}}^{{2}}"),
        PostfixOp::Cube => format!("{{{base}}}^{{3}}"),
        PostfixOp::Inverse => format!("{{{base}}}^{{-1}}"),
        PostfixOp::Factorial => format!("{base}!"),
    }
}

fn binary_tex(op: Operator, left: &Expr, right: &Expr) -> String {
    match op {
        Operator::Add => format!("{}+{}", latex(left), sum_operand(right, Precedence::Sum)),
        Operator::Subtract => format!(
            "{}-{}",
            latex(left),
            sum_operand(right, Precedence::Product)
        ),
        Operator::Multiply => product_tex(left, right, "\\times "),
        Operator::Divide => format!("\\frac{{{}}}{{{}}}", latex(left), latex(right)),
        Operator::Power => format!(
            "{{{}}}^{{{}}}",
            operand_tex(left, Precedence::Postfix),
            latex(right)
        ),
        Operator::XRoot => format!("\\sqrt[{}]{{{}}}", latex(left), latex(right)),
        _ => infix_tex(op, left, right),
    }
}

/// Counting, relations and logic: the operator between its operands, the
/// right one parenthesized unless it binds tighter.
fn infix_tex(op: Operator, left: &Expr, right: &Expr) -> String {
    let symbol = match op {
        Operator::Permutations => "\\,\\mathrm{nPr}\\,",
        Operator::Combinations => "\\,\\mathrm{nCr}\\,",
        Operator::Equal => "=",
        Operator::NotEqual => "\\ne ",
        Operator::Greater => ">",
        Operator::GreaterEqual => "\\ge ",
        Operator::Less => "<",
        Operator::LessEqual => "\\le ",
        Operator::And => "\\;\\mathrm{and}\\;",
        Operator::Or => "\\;\\mathrm{or}\\;",
        _ => "\\;\\mathrm{xor}\\;",
    };
    let level = op.precedence();
    let right_tex = match right.precedence() {
        tighter if tighter > level => latex(right),
        _ => format!("\\left({}\\right)", latex(right)),
    };
    format!("{}{symbol}{right_tex}", operand_tex(left, level))
}

fn implied_tex(left: &Expr, right: &Expr) -> String {
    let sign = if starts_with_digit(right) {
        "\\cdot "
    } else {
        ""
    };
    product_tex(left, right, sign)
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
        Expr::Postfix(_, operand, _) => starts_with_digit(operand),
        Expr::Binary(Operator::Power, base, ..) => starts_with_digit(base),
        _ => false,
    }
}

fn call_tex(function: Function, arguments: &[Expr]) -> String {
    let all: Vec<String> = arguments.iter().map(latex).collect();
    match (function, &all[..]) {
        (Function::Sqrt, [x]) => format!("\\sqrt{{{x}}}"),
        (Function::CubeRoot, [x]) => format!("\\sqrt[3]{{{x}}}"),
        (Function::TenPower, [x]) => format!("10^{{{x}}}"),
        (Function::ExpPower, [x]) => format!("e^{{{x}}}"),
        (Function::Abs, [x]) => format!("\\left|{x}\\right|"),
        (Function::LogBase, [x, base]) => format!("\\log_{{{base}}}\\left({x}\\right)"),
        (Function::Summation, [body, k, start, end]) => {
            format!("\\sum_{{{k}={start}}}^{{{end}}}\\left({body}\\right)")
        }
        (Function::FnInt, [body, x, a, b]) => format!("\\int_{{{a}}}^{{{b}}}{body}\\,d{x}"),
        _ => format!(
            "{}\\left({}\\right)",
            function_name(function),
            all.join(", ")
        ),
    }
}

fn function_name(function: Function) -> String {
    let name = match function {
        Function::Sin => "\\sin",
        Function::Cos => "\\cos",
        Function::Tan => "\\tan",
        Function::ArcSin => "\\sin^{-1}",
        Function::ArcCos => "\\cos^{-1}",
        Function::ArcTan => "\\tan^{-1}",
        Function::Log => "\\log",
        Function::Ln => "\\ln",
        Function::Min => "\\min",
        Function::Max => "\\max",
        Function::Gcd => "\\gcd",
        _ => {
            let text = function.text().trim_end_matches('(');
            return format!("\\operatorname{{{text}}}");
        }
    };
    name.to_owned()
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
            ("5→A", "5\\to A"),
            ("2a", "2\\mathrm{Ans}"),
            ("1+1=2", "1+1=2"),
            ("1<2&2<3", "1<2\\;\\mathrm{and}\\;2<3"),
            ("m1,2", "\\max\\left(1, 2\\right)"),
            ("4f", "4!"),
            ("aF", "\\mathrm{Ans}\\blacktriangleright\\mathrm{Frac}"),
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
