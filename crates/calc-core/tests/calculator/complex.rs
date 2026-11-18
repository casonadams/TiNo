use calc_core::{Calculator, Key, ScreenView};

use crate::support::{choose, press, pressed, results};

const I: [Key; 2] = [Key::Second, Key::Decimal];
/// MODE's last row: REAL, a+bi, re^θi.
const NUMBER_ROW: usize = 3;

/// `(3+4i)` typed.
fn three_plus_four_i(calculator: &mut Calculator) {
    press(
        calculator,
        &[Key::LeftParen, Key::Digit3, Key::Add, Key::Digit4],
    );
    press(calculator, &I);
    press(calculator, &[Key::RightParen]);
}

#[test]
fn i_squared_is_minus_one_even_in_real_mode() {
    let mut calculator = pressed(&I);
    press(&mut calculator, &[Key::Square, Key::Enter]);
    assert_eq!(results(&calculator), ["⁻1"]);
}

#[test]
fn complex_products_and_quotients_show_as_a_plus_bi() {
    let mut calculator = Calculator::default();
    three_plus_four_i(&mut calculator);
    press(
        &mut calculator,
        &[Key::LeftParen, Key::Digit1, Key::Subtract, Key::Digit2],
    );
    press(&mut calculator, &I);
    press(&mut calculator, &[Key::RightParen, Key::Enter]);
    three_plus_four_i(&mut calculator);
    press(&mut calculator, &[Key::Divide, Key::Digit2]);
    press(&mut calculator, &I);
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator), ["11−2i", "⁻2+1.5i"]);
}

#[test]
fn a_nonreal_answer_is_an_error_in_real_mode_and_complex_in_a_plus_bi() {
    let root = [
        Key::Second,
        Key::Square,
        Key::Negate,
        Key::Digit4,
        Key::Enter,
    ];
    let mut calculator = pressed(&root);
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:NONREAL ANSWER");
    press(&mut calculator, &[Key::Enter]);
    choose(&mut calculator, NUMBER_ROW, 1);
    press(&mut calculator, &root);
    assert_eq!(results(&calculator), ["2i"]);
    assert_eq!(calculator.screen().status[2], "a+bi");
}

#[test]
fn polar_mode_and_cpx_functions() {
    let mut calculator = Calculator::default();
    choose(&mut calculator, NUMBER_ROW, 2);
    press(&mut calculator, &[Key::Digit1, Key::Add]);
    press(&mut calculator, &I);
    press(&mut calculator, &[Key::Enter]);
    press(
        &mut calculator,
        &[Key::Math, Key::Right, Key::Right, Key::Digit5],
    );
    three_plus_four_i(&mut calculator);
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator), ["1.414213562e^(.7853981634i)", "5"]);
}
