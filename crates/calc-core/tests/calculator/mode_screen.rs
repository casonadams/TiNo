use crate::support::{press, pressed, results};
use calc_core::{Calculator, Key, ScreenView};

const QUIT: [Key; 2] = [Key::Second, Key::Mode];

/// Opens MODE, moves `down` rows and `right` options, chooses, and quits.
fn choose(calculator: &mut Calculator, down: usize, right: usize) {
    press(calculator, &[Key::Mode]);
    (0..down).for_each(|_| calculator.press(Key::Down));
    (0..right).for_each(|_| calculator.press(Key::Right));
    press(calculator, &[Key::Enter]);
    press(calculator, &QUIT);
}

#[test]
fn the_mode_screen_shows_each_setting_with_the_cursor_on_the_first_row() {
    let calculator = pressed(&[Key::Mode]);
    let ScreenView::Mode(mode) = calculator.screen().view else {
        panic!("expected the MODE screen");
    };
    let rows: Vec<_> = mode
        .rows
        .iter()
        .map(|r| (r.options[0], r.selected, r.highlighted))
        .collect();
    assert_eq!(
        rows,
        [
            ("NORMAL", 0, Some(0)),
            ("FLOAT", 0, None),
            ("RADIAN", 0, None),
        ]
    );
}

#[test]
fn fix_rounds_results_to_its_decimals() {
    let mut calculator = Calculator::default();
    choose(&mut calculator, 1, 3);
    press(
        &mut calculator,
        &[Key::Digit2, Key::Divide, Key::Digit3, Key::Enter],
    );
    assert_eq!(results(&calculator), [".67"]);
    assert_eq!(calculator.screen().status[1], "FIX2");
}

#[test]
fn degree_mode_measures_angles_in_degrees() {
    let mut calculator = Calculator::default();
    choose(&mut calculator, 2, 1);
    press(
        &mut calculator,
        &[Key::Sin, Key::Digit3, Key::Digit0, Key::Enter],
    );
    press(
        &mut calculator,
        &[Key::Sin, Key::Digit1, Key::Digit8, Key::Digit0, Key::Enter],
    );
    assert_eq!(results(&calculator), [".5", "0"]);
    assert_eq!(
        calculator.screen().status,
        ["NORMAL", "FLOAT", "DEGREE"].map(String::from)
    );
}

#[test]
fn scientific_notation_applies_to_every_result() {
    let mut calculator = Calculator::default();
    choose(&mut calculator, 0, 1);
    press(
        &mut calculator,
        &[Key::Digit1, Key::Digit2, Key::Digit3, Key::Enter],
    );
    assert_eq!(results(&calculator), ["1.23ᴇ2"]);
}

#[test]
fn clear_leaves_the_mode_screen_without_choosing() {
    let mut calculator = pressed(&[Key::Mode, Key::Right, Key::Clear]);
    assert!(calculator.screen().home().is_some());
    press(
        &mut calculator,
        &[Key::Digit1, Key::Digit2, Key::Digit3, Key::Enter],
    );
    assert_eq!(results(&calculator), ["123"]);
}
