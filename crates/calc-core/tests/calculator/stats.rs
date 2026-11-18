use calc_core::{Calculator, Key, ScreenView, TableView};

use crate::support::{home, press, pressed, results};

const L1: [Key; 2] = [Key::Second, Key::Digit1];
const L2: [Key; 2] = [Key::Second, Key::Digit2];
const QUIT: [Key; 2] = [Key::Second, Key::Mode];

fn table(calculator: &Calculator) -> TableView {
    let ScreenView::Table(table) = calculator.screen().view else {
        panic!("expected the list editor");
    };
    table
}

/// STAT 1:Edit…, then 1, 2, 3 into L₁ and 3, 5, 7 into L₂.
fn editing_lists() -> Calculator {
    let mut calculator = pressed(&[Key::Stat, Key::Enter]);
    for digit in [Key::Digit1, Key::Digit2, Key::Digit3] {
        press(&mut calculator, &[digit, Key::Enter]);
    }
    press(&mut calculator, &[Key::Right]);
    for digit in [Key::Digit3, Key::Digit5, Key::Digit7] {
        press(&mut calculator, &[digit, Key::Enter]);
    }
    calculator
}

/// The report lines of the newest history line.
fn report(calculator: &Calculator) -> Vec<String> {
    let history = home(calculator).history;
    history.back().expect("a history line").output.lines.clone()
}

#[test]
fn the_list_editor_stores_typed_values_into_the_lists() {
    let mut calculator = editing_lists();
    let shown = table(&calculator);
    assert_eq!(shown.headers, ["L₁", "L₂", "L₃"]);
    assert_eq!(shown.rows[0], ["1", "3", ""]);
    assert_eq!(shown.footer, "L₂(4)=");
    press(&mut calculator, &QUIT);
    press(&mut calculator, &L1);
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator), ["{1 2 3}"]);
}

#[test]
fn del_removes_the_selected_element() {
    let mut calculator = editing_lists();
    press(&mut calculator, &[Key::Up, Key::Up, Key::Del]);
    assert_eq!(table(&calculator).footer, "L₂(2)=7");
    press(&mut calculator, &QUIT);
    press(&mut calculator, &L2);
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator), ["{3 7}"]);
}

#[test]
fn a_cell_that_fails_to_evaluate_goes_back_to_the_typed_value() {
    let mut calculator = pressed(&[Key::Stat, Key::Enter]);
    press(
        &mut calculator,
        &[Key::Digit1, Key::Divide, Key::Digit0, Key::Enter],
    );
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:DIVIDE BY 0");
    press(&mut calculator, &[Key::Digit2]);
    let typed = table(&calculator).editing.expect("the typed value");
    assert_eq!(typed.0.len(), 3);
}

#[test]
fn one_variable_stats_reports_on_l1() {
    let mut calculator = editing_lists();
    press(&mut calculator, &QUIT);
    press(
        &mut calculator,
        &[Key::Stat, Key::Right, Key::Digit1, Key::Enter],
    );
    let lines = report(&calculator);
    assert_eq!(&lines[..3], ["x̄=2", "Σx=6", "Σx²=14"]);
    assert!(lines.contains(&"n=3".to_owned()), "{lines:?}");
}

#[test]
fn linear_regression_fits_l2_against_l1() {
    let mut calculator = editing_lists();
    press(&mut calculator, &QUIT);
    press(
        &mut calculator,
        &[Key::Stat, Key::Right, Key::Digit4, Key::Enter],
    );
    assert_eq!(report(&calculator), ["y=ax+b", "a=2", "b=1", "r²=1", "r=1"]);
}

#[test]
fn a_command_inside_an_expression_is_a_syntax_error() {
    let mut calculator = pressed(&[Key::Digit1, Key::Add]);
    press(
        &mut calculator,
        &[Key::Stat, Key::Right, Key::Digit1, Key::Enter],
    );
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:SYNTAX");
}
