use calc_core::{Calculator, Key, ScreenView};

use crate::support::{press, pressed, results};

const OPEN: [Key; 2] = [Key::Second, Key::Multiply];
const CLOSE: [Key; 2] = [Key::Second, Key::Subtract];
const MATRIX: [Key; 2] = [Key::Second, Key::Inverse];
const QUIT: [Key; 2] = [Key::Second, Key::Mode];
/// MATRIX NAMES 1: `[A]`.
const A: [Key; 3] = [Key::Second, Key::Inverse, Key::Digit1];

/// `[[1,2][3,4]]→[A]` ENTER.
fn storing_a_matrix() -> Calculator {
    let mut calculator = pressed(&OPEN);
    for (first, second) in [(Key::Digit1, Key::Digit2), (Key::Digit3, Key::Digit4)] {
        press(&mut calculator, &OPEN);
        press(&mut calculator, &[first, Key::Comma, second]);
        press(&mut calculator, &CLOSE);
    }
    press(&mut calculator, &CLOSE);
    press(&mut calculator, &[Key::Store]);
    press(&mut calculator, &A);
    press(&mut calculator, &[Key::Enter]);
    calculator
}

/// `keys` after `[A]`, then ENTER; the newest result.
fn result_of(calculator: &mut Calculator, keys: &[Key]) -> String {
    press(calculator, &A);
    press(calculator, keys);
    press(calculator, &[Key::Enter]);
    results(calculator).pop().expect("a result")
}

#[test]
fn a_matrix_shows_one_row_per_line_with_aligned_columns() {
    let mut calculator = storing_a_matrix();
    assert_eq!(results(&calculator), ["[[1 2]\n\u{2007}[3 4]]"]);
    let negated = result_of(&mut calculator, &[Key::Multiply, Key::Negate, Key::Digit5]);
    assert_eq!(negated, "[[\u{2007}⁻5 ⁻10]\n\u{2007}[⁻15 ⁻20]]");
}

#[test]
fn matrix_math_applies_the_operators_and_matrix_menu() {
    let mut calculator = storing_a_matrix();
    assert_eq!(
        result_of(&mut calculator, &[Key::Square]),
        "[[\u{2007}7 10]\n\u{2007}[15 22]]"
    );
    assert_eq!(
        result_of(&mut calculator, &[Key::Inverse]),
        "[[\u{2007}⁻2 \u{2007}\u{2007}1]\n\u{2007}[1.5 ⁻.5]]"
    );
    press(&mut calculator, &MATRIX);
    press(&mut calculator, &[Key::Right, Key::Digit1]);
    assert_eq!(result_of(&mut calculator, &[]), "⁻2");
    press(&mut calculator, &MATRIX);
    press(&mut calculator, &[Key::Right, Key::Digit3]);
    assert_eq!(result_of(&mut calculator, &[]), "{2 2}");
}

#[test]
fn matrices_and_lists_do_not_mix() {
    let mut calculator = storing_a_matrix();
    press(&mut calculator, &A);
    press(&mut calculator, &[Key::Add, Key::Digit1, Key::Enter]);
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:DATA TYPE");
}

#[test]
fn the_matrix_editor_grows_the_matrix_as_cells_are_filled() {
    let mut calculator = pressed(&MATRIX);
    press(&mut calculator, &[Key::Left, Key::Digit2]);
    press(
        &mut calculator,
        &[Key::Digit5, Key::Enter, Key::Up, Key::Right],
    );
    press(&mut calculator, &[Key::Digit6, Key::Enter]);
    let ScreenView::Table(table) = calculator.screen().view else {
        panic!("expected the matrix editor");
    };
    assert_eq!(table.headers, ["1", "2", "3"]);
    assert_eq!(table.footer, "[B](2,2)=");
    press(&mut calculator, &QUIT);
    press(
        &mut calculator,
        &[Key::Second, Key::Inverse, Key::Digit2, Key::Enter],
    );
    assert_eq!(results(&calculator), ["[[5 6]]"]);
}
