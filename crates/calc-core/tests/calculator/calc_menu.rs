use calc_core::{Calculator, GraphView, Key, ScreenView, screen::CalcView};

use crate::support::{press, pressed};

const CALC: [Key; 2] = [Key::Second, Key::Trace];
const RIGHT_30: [Key; 30] = [Key::Right; 30];

fn graph(calculator: &Calculator) -> GraphView {
    match calculator.screen().view {
        ScreenView::Graph(graph) => *graph,
        other => panic!("expected the graph, got {other:?}"),
    }
}

fn calc(calculator: &Calculator) -> CalcView {
    graph(calculator).calc.expect("a CALC item is shown")
}

fn trace_x(calculator: &Calculator) -> f64 {
    graph(calculator).trace.expect("the cursor is shown").x
}

fn error_message(calculator: &Calculator) -> &'static str {
    match calculator.screen().view {
        ScreenView::Error(error) => error.message,
        other => panic!("expected the error screen, got {other:?}"),
    }
}

/// Y1 = X²−2 (and Y2 = X with `with_line`), then 2nd CALC and `item`.
fn calculating(item: Key, with_line: bool) -> Calculator {
    let mut calculator = pressed(&[
        Key::YEquals,
        Key::XtThetaN,
        Key::Square,
        Key::Subtract,
        Key::Digit2,
    ]);
    if with_line {
        press(&mut calculator, &[Key::Down, Key::XtThetaN]);
    }
    press(&mut calculator, &CALC);
    press(&mut calculator, &[item]);
    calculator
}

/// The heading shown after each group of keys.
fn headings(calculator: &mut Calculator, steps: &[&[Key]]) -> Vec<Option<&'static str>> {
    let mut seen = vec![calc(calculator).heading];
    for keys in steps {
        press(calculator, keys);
        seen.push(calc(calculator).heading);
    }
    seen
}

#[test]
fn zero_asks_for_bounds_and_a_guess_then_lands_on_the_root() {
    let mut calculator = calculating(Key::Digit2, false);
    let seen = headings(
        &mut calculator,
        &[&[Key::Enter], &RIGHT_30, &[Key::Enter], &[Key::Enter]],
    );
    let expected = [
        "Left Bound?",
        "Right Bound?",
        "Right Bound?",
        "Guess?",
        "Zero",
    ];
    assert_eq!(seen, expected.map(Some));
    assert!((trace_x(&calculator) - 2f64.sqrt()).abs() < 1e-9);
}

#[test]
fn a_zero_shows_y_as_exactly_zero() {
    let mut calculator = calculating(Key::Digit2, false);
    press(&mut calculator, &[Key::Enter]);
    press(&mut calculator, &RIGHT_30);
    press(&mut calculator, &[Key::Enter, Key::Enter]);
    let trace = graph(&calculator).trace.expect("the cursor is shown");
    assert_eq!((trace.y, trace.y_text.as_str()), (Some(0.0), "Y=0"));
}

#[test]
fn the_next_key_after_a_result_traces_on() {
    let mut calculator = calculating(Key::Digit6, false);
    press(&mut calculator, &[Key::Enter, Key::Right]);
    assert_eq!(graph(&calculator).calc, None);
    assert!(trace_x(&calculator) > 0.0);
}

#[test]
fn minimum_needs_only_bounds() {
    let mut calculator = calculating(Key::Digit3, false);
    let keys = [
        Key::Left,
        Key::Left,
        Key::Enter,
        Key::Right,
        Key::Right,
        Key::Right,
        Key::Enter,
    ];
    press(&mut calculator, &keys);
    assert_eq!(calc(&calculator).heading, Some("Minimum"));
    assert!(trace_x(&calculator).abs() < 1e-6);
}

#[test]
fn no_sign_change_between_the_bounds_is_an_error() {
    let mut calculator = calculating(Key::Digit2, false);
    press(
        &mut calculator,
        &[Key::Enter, Key::Right, Key::Enter, Key::Enter],
    );
    assert_eq!(error_message(&calculator), "ERR:NO SIGN CHNG");
}

#[test]
fn derivative_shows_dy_dx_at_the_cursor() {
    let mut calculator = calculating(Key::Digit6, false);
    assert_eq!(calc(&calculator).heading, None);
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(calc(&calculator).value.as_deref(), Some("dy/dx=0"));
}

#[test]
fn integral_shades_between_the_limits_and_shows_the_area() {
    let mut calculator = calculating(Key::Digit7, false);
    let seen = headings(&mut calculator, &[&[Key::Enter], &RIGHT_30, &[Key::Enter]]);
    let expected = [
        Some("Lower Limit?"),
        Some("Upper Limit?"),
        Some("Upper Limit?"),
        None,
    ];
    assert_eq!(seen, expected);
    let upper = trace_x(&calculator);
    let shown = calc(&calculator);
    assert_eq!(shown.shade, Some((0.0, upper)));
    let value = shown.value.expect("the area is shown");
    let area: f64 = value["∫f(x)dx=".len()..].replace('⁻', "-").parse().unwrap();
    assert!((area - (upper.powi(3) / 3.0 - 2.0 * upper)).abs() < 1e-8);
}

#[test]
fn value_moves_the_cursor_to_the_x_typed() {
    let mut calculator = calculating(Key::Digit1, false);
    press(&mut calculator, &[Key::Digit3]);
    assert_eq!(
        calc(&calculator).entry.map(|(tokens, _)| tokens.len()),
        Some(1)
    );
    press(&mut calculator, &[Key::Enter]);
    let trace = graph(&calculator).trace.expect("the cursor is shown");
    assert_eq!((trace.x, trace.y), (3.0, Some(7.0)));
}

#[test]
fn value_outside_the_window_is_invalid() {
    let mut calculator = calculating(Key::Digit1, false);
    press(&mut calculator, &[Key::Digit3, Key::Digit0, Key::Enter]);
    assert_eq!(error_message(&calculator), "ERR:INVALID");
}

#[test]
fn intersect_moves_to_the_next_curve_for_the_second() {
    let mut calculator = calculating(Key::Digit5, true);
    let seen = headings(&mut calculator, &[&[Key::Enter]]);
    assert_eq!(seen, [Some("First curve?"), Some("Second curve?")]);
    assert_eq!(graph(&calculator).trace.map(|trace| trace.slot), Some(1));
}

#[test]
fn intersect_finds_the_meeting_nearest_the_guess() {
    let mut calculator = calculating(Key::Digit5, true);
    press(&mut calculator, &[Key::Enter, Key::Enter, Key::Enter]);
    assert_eq!(calc(&calculator).heading, Some("Intersection"));
    assert!((trace_x(&calculator) + 1.0).abs() < 1e-9);
}

#[test]
fn clear_at_a_prompt_returns_to_tracing() {
    let mut calculator = calculating(Key::Digit2, false);
    press(&mut calculator, &[Key::Clear]);
    let graph = graph(&calculator);
    assert_eq!((graph.calc, graph.trace.is_some()), (None, true));
}
