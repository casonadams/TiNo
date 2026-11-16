use calc_core::{Calculator, ErrorKind, Key};

fn pressed(keys: &[Key]) -> Calculator {
    let mut calculator = Calculator::default();
    press(&mut calculator, keys);
    calculator
}

fn press(calculator: &mut Calculator, keys: &[Key]) {
    keys.iter().for_each(|&key| calculator.press(key));
}

fn entry_text(calculator: &Calculator) -> String {
    calculator.screen().entry.iter().map(|t| t.text()).collect()
}

fn results(calculator: &Calculator) -> Vec<&str> {
    let history = calculator.screen().history;
    history
        .iter()
        .map(|line| line.output.result.as_str())
        .collect()
}

const ALPHA_A: [Key; 2] = [Key::Alpha, Key::Math];
const ENTRY: [Key; 2] = [Key::Second, Key::Enter];

#[test]
fn enter_evaluates_the_line_into_history() {
    let calculator = pressed(&[
        Key::Digit2,
        Key::Add,
        Key::Digit3,
        Key::Multiply,
        Key::Digit4,
        Key::Enter,
    ]);
    let line = &calculator.screen().history[0];
    assert_eq!(line.output.result, "14");
    assert_eq!(line.output.latex, "2+3\\times 4");
    assert_eq!(entry_text(&calculator), "");
}

#[test]
fn an_operator_first_on_the_line_applies_to_ans() {
    let calculator = pressed(&[
        Key::Digit5,
        Key::Enter,
        Key::Multiply,
        Key::Digit2,
        Key::Enter,
    ]);
    assert_eq!(results(&calculator), ["5", "10"]);
    assert_eq!(
        calculator.screen().history[1].output.latex,
        "\\mathrm{Ans}\\times 2"
    );
}

#[test]
fn enter_on_an_empty_line_repeats_the_last_entry() {
    let calculator = pressed(&[
        Key::Digit1,
        Key::Enter,
        Key::Add,
        Key::Digit1,
        Key::Enter,
        Key::Enter,
    ]);
    assert_eq!(results(&calculator), ["1", "2", "3"]);
}

#[test]
fn entry_recalls_older_entries_on_each_press() {
    let mut calculator = pressed(&[Key::Digit1, Key::Enter, Key::Digit2, Key::Enter]);
    press(&mut calculator, &ENTRY);
    assert_eq!(entry_text(&calculator), "2");
    press(&mut calculator, &ENTRY);
    assert_eq!(entry_text(&calculator), "1");
    press(&mut calculator, &[Key::Add, Key::Digit5, Key::Enter]);
    assert_eq!(results(&calculator), ["1", "2", "6"]);
}

#[test]
fn store_assigns_a_variable_that_later_entries_read() {
    let mut calculator = pressed(&[Key::Digit5, Key::Store]);
    press(&mut calculator, &ALPHA_A);
    press(&mut calculator, &[Key::Enter]);
    press(&mut calculator, &ALPHA_A);
    press(&mut calculator, &[Key::Digit2, Key::Enter]);
    assert_eq!(results(&calculator), ["5", "10"]);
}

#[test]
fn an_error_keeps_the_entry_with_the_cursor_at_the_fault() {
    let calculator = pressed(&[Key::Digit2, Key::Add, Key::Enter]);
    let screen = calculator.screen();
    assert_eq!(screen.error.map(|e| e.kind), Some(ErrorKind::Syntax));
    assert_eq!((entry_text(&calculator).as_str(), screen.cursor), ("2+", 2));
    assert!(screen.history.is_empty());
}

#[test]
fn the_next_key_dismisses_the_error_and_editing_continues() {
    let mut calculator = pressed(&[Key::Digit2, Key::Add, Key::Enter, Key::Digit3]);
    assert_eq!(calculator.screen().error, None);
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator), ["5"]);
}

#[test]
fn evaluation_errors_are_reported() {
    let calculator = pressed(&[Key::Digit1, Key::Divide, Key::Digit0, Key::Enter]);
    assert_eq!(
        calculator.screen().error.map(|e| e.kind),
        Some(ErrorKind::DivideByZero)
    );
}

#[test]
fn clear_on_an_empty_line_clears_the_screen() {
    let mut calculator = pressed(&[Key::Digit1, Key::Enter, Key::Digit2, Key::Clear]);
    assert_eq!(results(&calculator), ["1"]);
    press(&mut calculator, &[Key::Clear]);
    assert!(calculator.screen().history.is_empty());
}
