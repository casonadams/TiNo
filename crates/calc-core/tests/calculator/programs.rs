use calc_core::{Calculator, Key, ScreenView};

use crate::support::{home, press, results};

const QUIT: [Key; 2] = [Key::Second, Key::Mode];

/// PRGM NEW: a program named by the letter `name` types (ALPHA is locked),
/// each line typed with its keys and ENTER between lines, then 2nd QUIT.
fn new_program(calculator: &mut Calculator, name: Key, lines: &[&[Key]]) {
    press(
        calculator,
        &[Key::Prgm, Key::Left, Key::Enter, name, Key::Enter],
    );
    for (index, keys) in lines.iter().enumerate() {
        if index > 0 {
            press(calculator, &[Key::Enter]);
        }
        press(calculator, keys);
    }
    press(calculator, &QUIT);
}

/// PRGM EXEC: the first program's name typed and run.
fn run_first(calculator: &mut Calculator) {
    press(calculator, &[Key::Prgm, Key::Digit1, Key::Enter]);
}

#[test]
fn a_program_loops_and_displays_then_says_done() {
    let mut calculator = Calculator::default();
    let for_loop = [
        Key::Prgm,
        Key::Digit4,
        Key::XtThetaN,
        Key::Comma,
        Key::Digit1,
        Key::Comma,
        Key::Digit3,
    ];
    let display = [
        Key::Prgm,
        Key::Right,
        Key::Digit3,
        Key::XtThetaN,
        Key::Square,
    ];
    new_program(
        &mut calculator,
        Key::Math,
        &[&for_loop, &display, &[Key::Prgm, Key::Digit7]],
    );
    run_first(&mut calculator);
    assert_eq!(results(&calculator), ["", "1", "4", "9", "Done"]);
}

#[test]
fn prompt_waits_for_a_value() {
    let mut calculator = Calculator::default();
    let prompt = [Key::Prgm, Key::Right, Key::Digit2, Key::Alpha, Key::Apps];
    let display = [
        Key::Prgm,
        Key::Right,
        Key::Digit3,
        Key::Alpha,
        Key::Apps,
        Key::Square,
    ];
    new_program(&mut calculator, Key::Apps, &[&prompt, &display]);
    run_first(&mut calculator);
    assert_eq!(home(&calculator).prompt, Some("B=?"));
    press(&mut calculator, &[Key::Digit4, Key::Enter]);
    let last = home(&calculator)
        .history
        .iter()
        .rev()
        .nth(2)
        .unwrap()
        .output
        .lines
        .clone();
    assert_eq!(last, ["B=?4"]);
    assert_eq!(results(&calculator)[2..], ["16", "Done"]);
}

#[test]
fn an_error_goes_to_the_program_line_at_fault() {
    let mut calculator = Calculator::default();
    let fine = [Key::Digit1];
    let fault = [Key::Digit1, Key::Divide, Key::Digit0];
    new_program(&mut calculator, Key::Math, &[&fine, &fault]);
    run_first(&mut calculator);
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:DIVIDE BY 0");
    press(&mut calculator, &[Key::Digit2]);
    let ScreenView::Program(program) = calculator.screen().view else {
        panic!("expected the program editor");
    };
    assert_eq!(program.title, "PROGRAM:A");
    assert_eq!(program.lines[1].cursor, Some(0));
}

#[test]
fn on_breaks_a_program_that_never_ends() {
    let mut calculator = Calculator::default();
    let forever = [Key::Prgm, Key::Digit5, Key::Digit1];
    new_program(
        &mut calculator,
        Key::Math,
        &[&forever, &[Key::Prgm, Key::Digit7]],
    );
    run_first(&mut calculator);
    assert!(calculator.screen().busy);
    calculator.resume();
    assert!(calculator.screen().busy);
    press(&mut calculator, &[Key::On]);
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:BREAK");
}
