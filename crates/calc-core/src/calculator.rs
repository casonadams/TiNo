use crate::{
    editor::Editor,
    key::Key,
    keymap::{self, Action, Layer},
    token::Token,
};

/// ALPHA's state: off, for the next key, or locked (2nd ALPHA) until ALPHA
/// is pressed again.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum AlphaState {
    #[default]
    Off,
    Once,
    Locked,
}

/// The modifier keys' state, which decides the layer of the next press.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Modifiers {
    second: bool,
    alpha: AlphaState,
}

impl Modifiers {
    fn layer(self) -> Layer {
        match (self.second, self.alpha) {
            (true, _) => Layer::Second,
            (false, AlphaState::Off) => Layer::Normal,
            (false, _) => Layer::Alpha,
        }
    }

    fn press_second(&mut self) {
        self.second = !self.second;
    }

    fn press_alpha(&mut self) {
        self.alpha = match (self.second, self.alpha) {
            (true, _) => AlphaState::Locked,
            (false, AlphaState::Off) => AlphaState::Once,
            (false, _) => AlphaState::Off,
        };
        self.second = false;
    }

    /// After any other key: 2nd and a single ALPHA are used up, a lock stays.
    fn release(&mut self) {
        self.second = false;
        if self.alpha == AlphaState::Once {
            self.alpha = AlphaState::Off;
        }
    }
}

/// What the cursor shows: the mode the next key press is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Overwrite,
    Insert,
    Second,
    Alpha,
    AlphaLock,
}

/// What the screen shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Screen<'a> {
    pub entry: &'a [Token],
    pub cursor: usize,
    pub cursor_style: CursorStyle,
}

/// The calculator: key presses in, a screen out.
#[derive(Debug, Clone, Default)]
pub struct Calculator {
    modifiers: Modifiers,
    editor: Editor,
}

impl Calculator {
    pub fn press(&mut self, key: Key) {
        match key {
            Key::Second => self.modifiers.press_second(),
            Key::Alpha => self.modifiers.press_alpha(),
            _ => {
                let action = keymap::action(key, self.modifiers.layer());
                self.modifiers.release();
                if let Some(action) = action {
                    self.perform(action);
                }
            }
        }
    }

    pub fn screen(&self) -> Screen<'_> {
        Screen {
            entry: self.editor.tokens(),
            cursor: self.editor.cursor(),
            cursor_style: self.cursor_style(),
        }
    }

    fn perform(&mut self, action: Action) {
        match action {
            Action::Insert(token) => self.editor.type_token(token),
            Action::Edit(edit) => self.editor.apply(edit),
            Action::Command(_) => {}
        }
    }

    fn cursor_style(&self) -> CursorStyle {
        match (self.modifiers.layer(), self.modifiers.alpha) {
            (Layer::Second, _) => CursorStyle::Second,
            (Layer::Alpha, AlphaState::Locked) => CursorStyle::AlphaLock,
            (Layer::Alpha, _) => CursorStyle::Alpha,
            (Layer::Normal, _) if self.editor.is_inserting() => CursorStyle::Insert,
            (Layer::Normal, _) => CursorStyle::Overwrite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Letter;

    fn pressed(keys: &[Key]) -> Calculator {
        let mut calculator = Calculator::default();
        keys.iter().for_each(|&key| calculator.press(key));
        calculator
    }

    fn entry_text(calculator: &Calculator) -> String {
        calculator.screen().entry.iter().map(|t| t.text()).collect()
    }

    #[test]
    fn second_applies_to_one_key() {
        let calculator = pressed(&[Key::Second, Key::Sin, Key::Sin]);
        assert_eq!(entry_text(&calculator), "sin⁻¹(sin(");
        assert_eq!(calculator.screen().cursor_style, CursorStyle::Overwrite);
    }

    #[test]
    fn pressing_second_twice_cancels_it() {
        let calculator = pressed(&[Key::Second, Key::Second, Key::Power]);
        assert_eq!(entry_text(&calculator), "^");
    }

    #[test]
    fn alpha_applies_to_one_key() {
        let calculator = pressed(&[Key::Alpha, Key::Math, Key::Math]);
        assert_eq!(calculator.screen().entry[0], Token::Letter(Letter::A));
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
}
