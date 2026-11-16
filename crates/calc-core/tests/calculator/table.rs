use calc_core::{Calculator, Key, ScreenView, TableView};

use crate::support::{field_value, press, pressed};

const TBLSET: [Key; 2] = [Key::Second, Key::Window];
const TABLE: [Key; 2] = [Key::Second, Key::Graph];

fn table(calculator: &Calculator) -> TableView {
    match calculator.screen().view {
        ScreenView::Table(table) => table,
        other => panic!("expected the table, got {other:?}"),
    }
}

/// Y1 = X², Y2 = 1/X.
fn with_two_functions() -> Calculator {
    pressed(&[
        Key::YEquals,
        Key::XtThetaN,
        Key::Square,
        Key::Down,
        Key::Digit1,
        Key::Divide,
        Key::XtThetaN,
    ])
}

#[test]
fn the_table_lists_x_from_tbl_start_beside_each_graphed_function() {
    let mut calculator = with_two_functions();
    press(&mut calculator, &TABLE);
    let table = table(&calculator);
    assert_eq!(table.headers, ["X", "Y1", "Y2"]);
    assert_eq!(table.rows[0], ["0", "0", "ERROR"]);
    assert_eq!(table.rows[2], ["2", "4", ".5"]);
    assert_eq!(table.footer, "X=0");
}

#[test]
fn tblset_changes_where_the_table_starts_and_its_step() {
    let mut calculator = with_two_functions();
    press(&mut calculator, &TBLSET);
    press(
        &mut calculator,
        &[Key::Clear, Key::Negate, Key::Digit1, Key::Enter],
    );
    press(
        &mut calculator,
        &[Key::Clear, Key::Decimal, Key::Digit5, Key::Enter],
    );
    assert_eq!(field_value(&calculator, "TblStart"), "⁻1");
    assert_eq!(field_value(&calculator, "ΔTbl"), ".5");
    press(&mut calculator, &TABLE);
    let first: Vec<_> = table(&calculator)
        .rows
        .iter()
        .map(|row| row[0].clone())
        .take(3)
        .collect();
    assert_eq!(first, ["⁻1", "⁻.5", "0"]);
}

#[test]
fn a_zero_step_is_refused_with_the_error_screen() {
    let mut calculator = pressed(&TBLSET);
    press(
        &mut calculator,
        &[Key::Down, Key::Clear, Key::Digit0, Key::Enter],
    );
    let ScreenView::Error(error) = calculator.screen().view else {
        panic!("expected the error screen");
    };
    assert_eq!(error.message, "ERR:DOMAIN");
    press(&mut calculator, &[Key::Digit2]);
    assert_eq!(field_value(&calculator, "ΔTbl"), "1");
}

#[test]
fn moving_the_selection_reports_the_cell_and_scrolls_past_the_edge() {
    let mut calculator = with_two_functions();
    press(&mut calculator, &TABLE);
    press(&mut calculator, &[Key::Down, Key::Down, Key::Right]);
    assert_eq!(table(&calculator).footer, "Y1=4");
    (0..8).for_each(|_| calculator.press(Key::Down));
    let table = table(&calculator);
    assert_eq!(
        (table.rows[0][0].as_str(), table.footer.as_str()),
        ("3", "Y1=100")
    );
}

#[test]
fn scrolling_moves_the_rows_and_works_only_on_the_table() {
    let mut calculator = with_two_functions();
    calculator.scroll_table(5);
    press(&mut calculator, &TABLE);
    assert_eq!(table(&calculator).rows[0][0], "0");
    calculator.scroll_table(-3);
    assert_eq!(table(&calculator).rows[0][0], "⁻3");
}

#[test]
fn clear_leaves_the_table_for_home() {
    let mut calculator = pressed(&TABLE);
    press(&mut calculator, &[Key::Clear]);
    assert!(calculator.screen().home().is_some());
}
