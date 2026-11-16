use crate::support::{entry_text, home, press, pressed, results};
use calc_core::{ErrorChoice, Key, ScreenView, home::HistoryPart};

const ALPHA_A: [Key; 2] = [Key::Alpha, Key::Math];
const ENTRY: [Key; 2] = [Key::Second, Key::Enter];
const QUIT: [Key; 2] = [Key::Second, Key::Mode];

fn error_message(calculator: &calc_core::Calculator) -> Option<&'static str> {
    match calculator.screen().view {
        ScreenView::Error(error) => Some(error.message),
        _ => None,
    }
}

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
    let line = &home(&calculator).history[0];
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
        home(&calculator).history[1].output.latex,
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
fn an_error_opens_the_error_screen_on_quit() {
    let calculator = pressed(&[Key::Digit1, Key::Divide, Key::Digit0, Key::Enter]);
    match calculator.screen().view {
        ScreenView::Error(error) => {
            assert_eq!(error.message, "ERR:DIVIDE BY 0");
            assert_eq!(error.highlighted, ErrorChoice::Quit);
        }
        other => panic!("expected the error screen, got {other:?}"),
    }
}

#[test]
fn quit_returns_to_an_empty_line_and_entry_still_recalls_the_failed_entry() {
    let mut calculator = pressed(&[Key::Digit2, Key::Add, Key::Enter, Key::Digit1]);
    assert_eq!(entry_text(&calculator), "");
    assert!(home(&calculator).history.is_empty());
    press(&mut calculator, &ENTRY);
    assert_eq!(entry_text(&calculator), "2+");
}

#[test]
fn goto_returns_to_the_entry_with_the_cursor_at_the_fault() {
    let mut calculator = pressed(&[
        Key::Digit2,
        Key::Add,
        Key::Multiply,
        Key::Digit3,
        Key::Enter,
    ]);
    press(&mut calculator, &[Key::Down, Key::Enter]);
    assert_eq!(
        (entry_text(&calculator).as_str(), home(&calculator).cursor),
        ("2+×3", 2)
    );
    press(&mut calculator, &[Key::Del, Key::Enter]);
    assert_eq!(results(&calculator), ["5"]);
}

#[test]
fn the_error_screen_waits_for_a_choice() {
    let mut calculator = pressed(&[Key::Ln, Key::Digit0, Key::Enter, Key::Digit7, Key::Sin]);
    assert_eq!(error_message(&calculator), Some("ERR:DOMAIN"));
    press(&mut calculator, &QUIT);
    assert_eq!(error_message(&calculator), None);
    assert_eq!(entry_text(&calculator), "");
}

#[test]
fn clear_on_an_empty_line_clears_the_screen() {
    let mut calculator = pressed(&[Key::Digit1, Key::Enter, Key::Digit2, Key::Clear]);
    assert_eq!(results(&calculator), ["1"]);
    press(&mut calculator, &[Key::Clear]);
    assert!(home(&calculator).history.is_empty());
}

#[test]
fn up_highlights_history_and_enter_types_the_highlighted_part() {
    let mut calculator = pressed(&[Key::Digit1, Key::Divide, Key::Digit4, Key::Enter]);
    press(&mut calculator, &[Key::Digit2, Key::Add, Key::Up]);
    let selected = home(&calculator).selected.expect("a part is highlighted");
    assert_eq!((selected.line, selected.part), (0, HistoryPart::Result));
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(entry_text(&calculator), "2+.25");
    press(
        &mut calculator,
        &[Key::Add, Key::Up, Key::Up, Key::Up, Key::Enter],
    );
    assert_eq!(entry_text(&calculator), "2+.25+1÷4");
    assert_eq!(home(&calculator).selected, None);
}

#[test]
fn down_past_the_newest_result_or_clear_returns_to_the_entry_line() {
    let mut calculator = pressed(&[Key::Digit5, Key::Enter, Key::Up, Key::Down]);
    assert_eq!(home(&calculator).selected, None);
    press(&mut calculator, &[Key::Up, Key::Clear, Key::Digit7]);
    assert_eq!(entry_text(&calculator), "7");
    assert_eq!(results(&calculator), ["5"]);
}

#[test]
fn goto_after_an_evaluation_error_puts_the_cursor_on_the_failing_operation() {
    let mut calculator = pressed(&[
        Key::Digit1,
        Key::Add,
        Key::Digit1,
        Key::Divide,
        Key::Digit0,
        Key::Enter,
    ]);
    press(&mut calculator, &[Key::Digit2]);
    assert_eq!(home(&calculator).cursor, 3);
}
