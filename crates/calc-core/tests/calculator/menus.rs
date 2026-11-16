use crate::support::{entry_text, field_value, press, pressed, results};
use calc_core::{Key, ScreenView};

const TEST: [Key; 2] = [Key::Second, Key::Math];

fn menu_tab(calculator: &calc_core::Calculator) -> Option<(&'static str, Vec<String>)> {
    let ScreenView::Menu(menu) = calculator.screen().view else {
        return None;
    };
    let items = menu.items.iter();
    let labels = items.map(|item| format!("{}:{}", item.key, item.label));
    Some((menu.tabs[menu.tab], labels.collect()))
}

#[test]
fn math_items_type_into_the_entry_and_evaluate() {
    let mut calculator = pressed(&[Key::Math, Key::Right, Key::Digit7]);
    press(
        &mut calculator,
        &[Key::Digit3, Key::Comma, Key::Digit8, Key::RightParen],
    );
    assert_eq!(entry_text(&calculator), "max(3,8)");
    press(&mut calculator, &[Key::Enter]);
    assert_eq!(results(&calculator), ["8"]);
}

#[test]
fn arrows_move_through_tabs_and_enter_picks_the_highlighted_item() {
    let mut calculator = pressed(&[Key::Math, Key::Left]);
    let (title, items) = menu_tab(&calculator).expect("the menu is shown");
    assert_eq!(title, "PROB");
    assert_eq!(items[2], "3:nCr");
    press(
        &mut calculator,
        &[Key::Down, Key::Down, Key::Up, Key::Enter],
    );
    assert_eq!(entry_text(&calculator), "Ans nPr ");
}

#[test]
fn test_menu_relations_give_one_or_zero() {
    let mut calculator = pressed(&[Key::Digit2]);
    press(&mut calculator, &TEST);
    press(&mut calculator, &[Key::Digit3, Key::Digit1, Key::Enter]);
    press(&mut calculator, &[Key::Digit2]);
    press(&mut calculator, &TEST);
    press(
        &mut calculator,
        &[Key::Right, Key::Digit1, Key::Digit0, Key::Enter],
    );
    assert_eq!(results(&calculator), ["1", "0"]);
}

#[test]
fn clear_closes_the_menu_without_typing() {
    let calculator = pressed(&[Key::Digit4, Key::Math, Key::Clear]);
    assert_eq!(entry_text(&calculator), "4");
}

#[test]
fn a_menu_types_into_the_window_field_it_was_opened_from() {
    let mut calculator = pressed(&[Key::Window, Key::Clear, Key::Digit8]);
    press(&mut calculator, &[Key::Math, Key::Digit3, Key::Down]);
    press(&mut calculator, &[Key::Up]);
    assert_eq!(field_value(&calculator, "Xmin"), "512");
}

#[test]
fn a_menu_opened_from_the_graph_types_on_the_home_screen() {
    let calculator = pressed(&[Key::Graph, Key::Math, Key::Right, Key::Digit1]);
    assert_eq!(entry_text(&calculator), "abs(");
}

#[test]
fn vars_reads_window_settings_and_y_equals_functions() {
    let mut calculator = pressed(&[
        Key::YEquals,
        Key::XtThetaN,
        Key::Square,
        Key::Second,
        Key::Mode,
    ]);
    press(&mut calculator, &[Key::Vars, Key::Left, Key::Digit1]);
    press(
        &mut calculator,
        &[Key::LeftParen, Key::Digit3, Key::RightParen],
    );
    assert_eq!(entry_text(&calculator), "Y₁(3)");
    press(
        &mut calculator,
        &[Key::Enter, Key::Vars, Key::Digit1, Key::Enter],
    );
    assert_eq!(results(&calculator), ["9", "⁻10"]);
}

#[test]
fn a_long_menu_scrolls_to_keep_the_highlighted_item_shown() {
    let mut calculator = pressed(&[Key::Math]);
    let (_, items) = menu_tab(&calculator).expect("the menu is shown");
    assert_eq!(items.first().map(String::as_str), Some("1:►Frac"));
    press(&mut calculator, &[Key::Up]);
    let (_, items) = menu_tab(&calculator).expect("the menu is shown");
    assert_eq!(items.last().map(String::as_str), Some("A:logBASE("));
    assert_eq!(items.first().map(String::as_str), Some("3:³"));
}
