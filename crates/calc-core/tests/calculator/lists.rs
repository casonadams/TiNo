use calc_core::{Calculator, Key};

use crate::support::{entry_text, press, pressed, results};

const BRACE: [Key; 2] = [Key::Second, Key::LeftParen];
const CLOSE: [Key; 2] = [Key::Second, Key::RightParen];
const L1: [Key; 2] = [Key::Second, Key::Digit1];
const LIST: [Key; 2] = [Key::Second, Key::Stat];

/// `{1,2,3}→L₁` ENTER.
fn storing_a_list() -> Calculator {
    let mut calculator = pressed(&BRACE);
    press(
        &mut calculator,
        &[
            Key::Digit1,
            Key::Comma,
            Key::Digit2,
            Key::Comma,
            Key::Digit3,
        ],
    );
    press(&mut calculator, &CLOSE);
    press(&mut calculator, &[Key::Store]);
    press(&mut calculator, &L1);
    press(&mut calculator, &[Key::Enter]);
    calculator
}

#[test]
fn a_list_is_written_between_braces_and_stored_by_name() {
    let mut calculator = storing_a_list();
    press(&mut calculator, &L1);
    press(&mut calculator, &[Key::Square, Key::Enter]);
    assert_eq!(results(&calculator), ["{1 2 3}", "{1 4 9}"]);
}

#[test]
fn the_list_menu_types_names_and_list_math() {
    let mut calculator = storing_a_list();
    press(&mut calculator, &LIST);
    press(&mut calculator, &[Key::Left, Key::Digit5]);
    press(&mut calculator, &LIST);
    press(&mut calculator, &[Key::Digit1]);
    assert_eq!(entry_text(&calculator), "sum(L₁");
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator)[1], "6");
}

#[test]
fn a_list_where_a_number_belongs_is_a_data_type_error() {
    let mut calculator = storing_a_list();
    press(&mut calculator, &L1);
    press(
        &mut calculator,
        &[Key::Store, Key::Alpha, Key::Math, Key::Enter],
    );
    let calc_core::ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:DATA TYPE");
}
