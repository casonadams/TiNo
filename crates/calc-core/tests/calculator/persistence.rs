use calc_core::{Calculator, Key, LoadError, ScreenView};

use crate::support::{field_value, home, press, pressed, results};

/// A calculator with something in every kept part: A=5 in history, Degree
/// and Fix 2, Y1=X², Xmax=20 and TblStart=3.
fn used_calculator() -> Calculator {
    let mut calculator = pressed(&[Key::Digit5, Key::Store, Key::Alpha, Key::Math, Key::Enter]);
    press(
        &mut calculator,
        &[
            Key::Mode,
            Key::Down,
            Key::Right,
            Key::Right,
            Key::Right,
            Key::Enter,
        ],
    );
    press(
        &mut calculator,
        &[Key::Down, Key::Right, Key::Enter, Key::Second, Key::Mode],
    );
    press(&mut calculator, &[Key::YEquals, Key::XtThetaN, Key::Square]);
    press(
        &mut calculator,
        &[
            Key::Window,
            Key::Down,
            Key::Clear,
            Key::Digit2,
            Key::Digit0,
            Key::Enter,
        ],
    );
    press(
        &mut calculator,
        &[
            Key::Second,
            Key::Window,
            Key::Clear,
            Key::Digit3,
            Key::Enter,
        ],
    );
    press(&mut calculator, &[Key::Second, Key::Mode]);
    calculator
}

fn reloaded() -> Calculator {
    Calculator::load(&used_calculator().save()).expect("a saved calculator loads")
}

#[test]
fn a_reloaded_calculator_keeps_its_home_screen_settings_and_variables() {
    let mut loaded = reloaded();
    assert_eq!(results(&loaded), ["5"]);
    assert_eq!(
        loaded.screen().status,
        ["NORMAL", "FIX2", "DEGREE"].map(String::from)
    );
    press(&mut loaded, &[Key::Alpha, Key::Math, Key::Enter]);
    assert_eq!(results(&loaded)[1], "5.00");
}

#[test]
fn a_reloaded_calculator_keeps_its_functions_window_and_table_settings() {
    let mut loaded = reloaded();
    press(&mut loaded, &[Key::Graph]);
    let ScreenView::Graph(graph) = loaded.screen().view else {
        panic!("expected the graph");
    };
    assert_eq!((graph.window.x_max, graph.plot.curves.len()), (20.0, 1));
    press(&mut loaded, &[Key::Second, Key::Window]);
    assert_eq!(field_value(&loaded, "TblStart"), "3.00");
}

#[test]
fn a_reloaded_calculator_recalls_its_entries() {
    let mut loaded = reloaded();
    press(&mut loaded, &[Key::Second, Key::Enter]);
    let entry: String = home(&loaded).entry.iter().map(|t| t.text()).collect();
    assert_eq!(entry, "5→A");
}

#[test]
fn unreadable_snapshots_are_errors() {
    let saved = used_calculator().save();
    let cases = [
        ("not json".to_owned(), "malformed"),
        (
            saved.replacen("\"version\":1", "\"version\":2", 1),
            "version",
        ),
        (saved.replacen("\"Fix\":2", "\"Fix\":12", 1), "malformed"),
    ];
    for (text, expected) in cases {
        let kind = match Calculator::load(&text) {
            Err(LoadError::Malformed(_)) => "malformed",
            Err(LoadError::Version(_)) => "version",
            other => panic!("unexpected {other:?}"),
        };
        assert_eq!(kind, expected, "{text}");
    }
}
