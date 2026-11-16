use std::f64::consts::PI;

use super::*;
use crate::parse::{parse, tests::tokens};
use crate::{
    token::Token,
    vocabulary::{Function, Operator, PostfixOp},
};

/// `entry` evaluated with `memory`, a standard window, default TBLSET and
/// `functions` as Y₁, Y₂, ….
fn value_in(entry: &[Token], memory: &Memory, functions: &[&str]) -> Result<f64, CalcError> {
    let context = Context {
        memory,
        angle: Angle::Radian,
        window: &Window::standard(),
        table: &TableSettings::default(),
        functions: functions
            .iter()
            .map(|function| parse(&tokens(function)).map(|statement| statement.expr))
            .collect(),
    };
    context.evaluate(&parse(entry).unwrap().expr)
}

fn value_with(spelling: &str, functions: &[&str]) -> Result<f64, CalcError> {
    value_in(&tokens(spelling), &Memory::default(), functions)
}

fn value_of(spelling: &str, memory: &Memory) -> Result<f64, CalcError> {
    value_in(&tokens(spelling), memory, &[])
}

fn value_of_tokens(tokens: &[Token]) -> Result<f64, CalcError> {
    value_in(tokens, &Memory::default(), &[])
}

fn assert_close(cases: &[(&str, f64)]) {
    let memory = Memory::default();
    for &(spelling, expected) in cases {
        let value = value_of(spelling, &memory).unwrap();
        assert!(
            (value - expected).abs() < 1e-12,
            "{spelling}: {value} != {expected}"
        );
    }
}

#[test]
fn evaluates_with_the_calculators_precedence() {
    assert_close(&[
        ("2+3*4", 14.0),
        ("_2²", -4.0),
        ("2^3^2", 64.0),
        ("2_3", -6.0),
        ("2^_1", 0.5),
        ("2(3+4", 14.0),
        ("(2)(3)", 6.0),
        ("1/2p", PI / 2.0),
        ("2p", 2.0 * PI),
        ("r(16", 4.0),
        ("r(9)²", 9.0),
        ("4!", 0.25),
        ("2³", 8.0),
        ("4f", 24.0),
        ("E3", 1000.0),
        ("1.5E_2", 0.015),
        (".5+5.", 5.5),
        ("s(0", 0.0),
        ("(_8)^(1/3", -2.0),
        ("(_8)^(2/3", 4.0),
    ]);
}

#[test]
fn relations_and_logic_give_one_or_zero_and_bind_loosest() {
    assert_close(&[
        ("2+1=3", 1.0),
        ("2<1", 0.0),
        ("1+1>1", 1.0),
        ("1=1&2=3", 0.0),
        ("1=2|2=2", 1.0),
        ("2(1<3)", 2.0),
    ]);
}

#[test]
fn functions_take_their_arguments() {
    assert_close(&[("m3,_2)", 3.0), ("R2.555,2", 2.56), ("R1/3", 0.3333333333)]);
}

#[test]
fn menu_functions_compute_as_the_calculator_does() {
    let call = |function, arguments: &[f64]| {
        let mut tokens = vec![Token::Func(function)];
        for (index, &argument) in arguments.iter().enumerate() {
            if index > 0 {
                tokens.push(Token::Comma);
            }
            if argument < 0.0 {
                tokens.push(Token::Negate);
            }
            tokens.extend(super::super::parse::tests::tokens(
                &argument.abs().to_string(),
            ));
        }
        value_of_tokens(&tokens)
    };
    let cases = [
        (Function::Abs, vec![-3.5], 3.5),
        (Function::IPart, vec![-3.5], -3.0),
        (Function::FPart, vec![-3.5], -0.5),
        (Function::Int, vec![-3.5], -4.0),
        (Function::CubeRoot, vec![-27.0], -3.0),
        (Function::LogBase, vec![8.0, 2.0], 3.0),
        (Function::Lcm, vec![4.0, 6.0], 12.0),
        (Function::Gcd, vec![12.0, 18.0], 6.0),
        (Function::Remainder, vec![-7.0, 3.0], -1.0),
        (Function::Not, vec![0.0], 1.0),
    ];
    for (function, arguments, expected) in cases {
        let value = call(function, &arguments).unwrap();
        assert!(
            (value - expected).abs() < 1e-12,
            "{function:?}{arguments:?}: {value}"
        );
    }
}

#[test]
fn counting_and_roots_written_between_operands() {
    let tokens = |n: &str, op, r: &str| {
        let mut all = super::super::parse::tests::tokens(n);
        all.push(Token::Op(op));
        all.extend(super::super::parse::tests::tokens(r));
        all
    };
    let cases = [
        (tokens("5", Operator::Permutations, "2"), 20.0),
        (tokens("5", Operator::Combinations, "2"), 10.0),
        (tokens("2", Operator::Combinations, "5"), 0.0),
        (tokens("3", Operator::XRoot, "_8"), -2.0),
    ];
    for (tokens, expected) in cases {
        assert_eq!(value_of_tokens(&tokens), Ok(expected));
    }
}

#[test]
fn rand_stays_strictly_between_zero_and_one_and_moves_on() {
    let memory = Memory::default();
    let draws: Vec<f64> = (0..1000).map(|_| memory.random()).collect();
    assert!(draws.iter().all(|&r| r > 0.0 && r < 1.0));
    assert_ne!(draws[0], draws[1]);
}

#[test]
fn reads_variables_and_ans() {
    let mut memory = Memory::default();
    memory.set(Letter::A, 3.0);
    memory.set_ans(10.0);
    assert_eq!(value_of("2A+a", &memory), Ok(16.0));
    assert_eq!(value_of("AB", &memory), Ok(0.0));
}

#[test]
fn reports_the_calculators_errors() {
    let memory = Memory::default();
    let cases = [
        ("1/0", ErrorKind::DivideByZero),
        ("0!", ErrorKind::DivideByZero),
        ("0^0", ErrorKind::Domain),
        ("0^_1", ErrorKind::DivideByZero),
        ("r(_4", ErrorKind::NonReal),
        ("(_2)^.5", ErrorKind::NonReal),
        ("E99*10", ErrorKind::Overflow),
        ("E100", ErrorKind::Overflow),
        ("2.5f", ErrorKind::Domain),
        ("70f", ErrorKind::Domain),
        ("m1", ErrorKind::Argument),
        ("R1,2,3", ErrorKind::Argument),
        ("R1,10", ErrorKind::Domain),
    ];
    for (spelling, kind) in cases {
        assert_eq!(
            value_of(spelling, &memory).map_err(|e| e.kind),
            Err(kind),
            "{spelling}"
        );
    }
    let factorial = [
        Token::Digit(crate::token::Digit::Three),
        Token::Postfix(PostfixOp::Factorial),
    ];
    assert_eq!(value_of_tokens(&factorial), Ok(6.0));
}

#[test]
fn tiny_results_are_zero() {
    assert_eq!(value_of("E_60*E_60", &Memory::default()), Ok(0.0));
}

#[test]
fn calculus_functions_evaluate_their_expression_in_the_named_variable() {
    let close = |spelling: &str, expected: f64| {
        let value = value_with(spelling, &[]).unwrap();
        assert!((value - expected).abs() < 1e-9, "{spelling}: {value}");
    };
    close("dX³,X,2", 12.000001);
    close("dX³,X,2,.1", 12.01);
    close("iX²,X,0,3", 9.0);
    close("isX),X,0,p", 2.0);
    close("nX²-2X,X,0,5", 1.0);
    close("xsX),X,0,p", PI / 2.0);
    close("SK²,K,1,10", 385.0);
    close("SSK,K,1,A),A,1,3", 10.0);
    assert_eq!(
        value_with("dX,2,3", &[]).map_err(|e| e.kind),
        Err(ErrorKind::Syntax)
    );
    assert_eq!(
        value_with("iX,X,0,1,0", &[]).map_err(|e| e.kind),
        Err(ErrorKind::Domain)
    );
}

#[test]
fn y_vars_read_y_equals_at_x_or_their_argument() {
    let mut memory = Memory::default();
    memory.set(Letter::X, 2.0);
    let functions = ["X²", "y+1"];
    let value = |spelling: &str| value_in(&tokens(spelling), &memory, &functions);
    assert_eq!(value("y"), Ok(4.0));
    assert_eq!(value("y(3)"), Ok(9.0));
    assert_eq!(value("z(_1)"), Ok(2.0));
}

#[test]
fn y_vars_that_read_themselves_or_empty_slots_fail() {
    let kind =
        |spelling: &str, functions: &[&str]| value_with(spelling, functions).map_err(|e| e.kind);
    assert_eq!(kind("y", &["y+1"]), Err(ErrorKind::Invalid));
    assert_eq!(kind("y", &["z", "y"]), Err(ErrorKind::Invalid));
    assert_eq!(kind("z", &["X"]), Err(ErrorKind::Undefined));
}

#[test]
fn window_and_table_settings_read_as_variables() {
    assert_eq!(value_with("w", &[]), Ok(-10.0));
    assert_eq!(value_with("2t", &[]), Ok(2.0));
}

#[test]
fn evaluation_errors_point_at_the_failing_operation() {
    let at = |spelling: &str| value_with(spelling, &[]).unwrap_err().position;
    assert_eq!(at("1+1/0"), Some(3));
    assert_eq!(at("2+r(_4"), Some(2));
    assert_eq!(at("(_2)^.5*3"), Some(4));
}
