use crate::support::{entry_text, home, pressed};
use calc_core::{CursorStyle, Key, Token, token::Letter};

#[test]
fn second_applies_to_one_key() {
    let calculator = pressed(&[Key::Second, Key::Sin, Key::Sin]);
    assert_eq!(entry_text(&calculator), "sin⁻¹(sin(");
    assert_eq!(calculator.screen().cursor_style, CursorStyle::Overwrite);
}

#[test]
fn pressing_second_twice_cancels_it() {
    let calculator = pressed(&[Key::Second, Key::Second, Key::Digit7, Key::Power]);
    assert_eq!(entry_text(&calculator), "7^");
}

#[test]
fn alpha_applies_to_one_key() {
    let calculator = pressed(&[Key::Alpha, Key::Math, Key::Digit1]);
    assert_eq!(home(&calculator).entry[0], Token::Letter(Letter::A));
    assert_eq!(entry_text(&calculator), "A1");
    assert_eq!(calculator.screen().cursor_style, CursorStyle::Overwrite);
}

#[test]
fn second_alpha_locks_alpha_until_alpha() {
    let mut calculator = pressed(&[Key::Second, Key::Alpha, Key::Math, Key::Apps]);
    assert_eq!(entry_text(&calculator), "AB");
    assert_eq!(calculator.screen().cursor_style, CursorStyle::AlphaLock);
    calculator.press(Key::Alpha);
    calculator.press(Key::Digit1);
    assert_eq!(entry_text(&calculator), "AB1");
}

#[test]
fn the_cursor_shows_the_pending_modifier() {
    let cases = [
        (vec![], CursorStyle::Overwrite),
        (vec![Key::Second], CursorStyle::Second),
        (vec![Key::Alpha], CursorStyle::Alpha),
        (vec![Key::Second, Key::Del], CursorStyle::Insert),
    ];
    for (keys, style) in cases {
        assert_eq!(pressed(&keys).screen().cursor_style, style, "{keys:?}");
    }
}

#[test]
fn editing_keys_work_while_alpha_is_locked() {
    let calculator = pressed(&[Key::Second, Key::Alpha, Key::Math, Key::Left, Key::Apps]);
    assert_eq!(entry_text(&calculator), "B");
    assert_eq!(calculator.screen().cursor_style, CursorStyle::AlphaLock);
}
