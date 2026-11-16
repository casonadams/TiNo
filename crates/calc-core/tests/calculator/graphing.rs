use calc_core::{Calculator, GraphView, Key, RowFocus, ScreenView};

use crate::support::{field_value, press, pressed};

const XT: Key = Key::XtThetaN;

fn graph(calculator: &Calculator) -> GraphView {
    match calculator.screen().view {
        ScreenView::Graph(graph) => *graph,
        other => panic!("expected the graph, got {other:?}"),
    }
}

fn error_message(calculator: &Calculator) -> &'static str {
    match calculator.screen().view {
        ScreenView::Error(error) => error.message,
        other => panic!("expected the error screen, got {other:?}"),
    }
}

/// Y1 = X², then GRAPH.
fn graphing_x_squared() -> Calculator {
    pressed(&[Key::YEquals, XT, Key::Square, Key::Graph])
}

#[test]
fn graph_plots_the_functions_typed_in_y_equals() {
    let graph = graph(&graphing_x_squared());
    let curve = &graph.plot.curves[0];
    assert_eq!(curve.slot, 0);
    assert_eq!(curve.ys[0], Some(100.0));
    assert_eq!(graph.x_ticks.len(), 21);
}

#[test]
fn turning_a_function_off_leaves_it_out_of_the_graph() {
    let calculator = pressed(&[
        Key::YEquals,
        XT,
        Key::Left,
        Key::Left,
        Key::Enter,
        Key::Graph,
    ]);
    assert!(graph(&calculator).plot.curves.is_empty());
}

#[test]
fn a_function_that_does_not_parse_goes_back_to_y_equals_at_the_fault() {
    let mut calculator = pressed(&[Key::YEquals, Key::Down, XT, Key::Add, Key::Graph]);
    assert_eq!(error_message(&calculator), "ERR:SYNTAX");
    press(&mut calculator, &[Key::Digit2]);
    let ScreenView::Functions(functions) = calculator.screen().view else {
        panic!("expected Y=");
    };
    assert_eq!(functions.rows[1].focus, Some(RowFocus::Entry(2)));
}

#[test]
fn window_entries_are_evaluated_when_the_cursor_leaves_them() {
    let mut calculator = pressed(&[Key::Window, Key::Clear, Key::Negate, Key::Digit2, Key::Down]);
    press(
        &mut calculator,
        &[Key::Clear, Key::Digit2, Key::Second, Key::Power, Key::Enter],
    );
    assert_eq!(field_value(&calculator, "Xmin"), "⁻2");
    assert_eq!(field_value(&calculator, "Xmax"), "6.283185307");
}

#[test]
fn a_window_whose_ranges_do_not_increase_cannot_be_graphed() {
    let calculator = pressed(&[
        Key::Window,
        Key::Clear,
        Key::Digit2,
        Key::Digit0,
        Key::Graph,
    ]);
    assert_eq!(error_message(&calculator), "ERR:WINDOW RANGE");
}

#[test]
fn zoom_menu_items_change_the_window_and_show_the_graph() {
    let mut calculator = graphing_x_squared();
    press(&mut calculator, &[Key::Zoom, Key::Digit2]);
    assert_eq!(graph(&calculator).window.x_max, 2.5);
    press(
        &mut calculator,
        &[
            Key::Zoom,
            Key::Down,
            Key::Down,
            Key::Down,
            Key::Down,
            Key::Enter,
        ],
    );
    assert_eq!(graph(&calculator).window.x_max, 10.0);
}

#[test]
fn zoom_fit_fits_y_to_the_functions() {
    let mut calculator = graphing_x_squared();
    press(&mut calculator, &[Key::Zoom, Key::Digit0]);
    let window = graph(&calculator).window;
    assert_eq!((window.y_min, window.y_max), (0.0, 100.0));
}

#[test]
fn trace_rides_a_function_and_reports_the_point() {
    let mut calculator = pressed(&[Key::YEquals, XT, Key::Square, Key::Trace]);
    let trace = graph(&calculator).trace.unwrap();
    assert_eq!(
        (trace.equation.as_str(), trace.x, trace.y),
        ("Y1=X²", 0.0, Some(0.0))
    );
    press(&mut calculator, &[Key::Right]);
    let trace = graph(&calculator).trace.unwrap();
    assert!(trace.x > 0.0);
    assert!(trace.y_text.starts_with("Y=."));
}

#[test]
fn gestures_move_zoom_and_trace_the_graph() {
    let mut calculator = graphing_x_squared();
    calculator.pan_graph(0.5, 0.0);
    assert_eq!(graph(&calculator).window.x_min, 0.0);
    calculator.zoom_graph(2.0, (0.0, 0.5));
    assert_eq!(graph(&calculator).window.x_max, 10.0);
    calculator.trace_at(1.0);
    assert_eq!(graph(&calculator).trace.unwrap().x, 10.0);
}

#[test]
fn gestures_do_nothing_off_the_graph() {
    let mut calculator = Calculator::default();
    calculator.pan_graph(0.5, 0.0);
    press(&mut calculator, &[Key::Graph]);
    assert_eq!(graph(&calculator).window.x_min, -10.0);
}

#[test]
fn clear_leaves_the_graph_for_home() {
    let mut calculator = graphing_x_squared();
    press(&mut calculator, &[Key::Clear]);
    assert!(calculator.screen().home().is_some());
}
