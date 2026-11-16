use crate::{
    editor::Editor,
    error::{CalcError, ErrorKind},
    eval::{Memory, evaluate},
    format::format_real,
    keymap::{Action, Command},
    parse::parse,
    settings::{Angle, Settings},
    token::{Digit, Token},
};

/// A line of a settings screen (WINDOW, TBLSET) whose value is typed as an
/// expression.
pub trait Field: Copy + Eq + 'static {
    /// The screen's title.
    const TITLE: &'static str;
    /// The screen's fields, top to bottom.
    const ALL: &'static [Self];

    fn label(self) -> &'static str;
}

/// A settings screen's cursor: the focused field and what is typed into it.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldCursor<F> {
    pub field: F,
    pub editor: Editor,
}

impl<F: Field> FieldCursor<F> {
    /// On `field`, its entry showing `value` as the calculator writes it.
    pub fn new(field: F, value: f64, settings: Settings) -> Self {
        let mut editor = Editor::default();
        editor.replace(number_tokens(&format_real(value, settings)));
        editor.place_cursor(0);
        FieldCursor { field, editor }
    }

    /// The field above or below, stopping at the first and last.
    pub fn neighbor(&self, down: bool) -> F {
        let index = F::ALL.iter().position(|&f| f == self.field).unwrap_or(0);
        let next = if down {
            (index + 1).min(F::ALL.len() - 1)
        } else {
            index.saturating_sub(1)
        };
        F::ALL[next]
    }

    /// Applies typing and editing keys to the entry; for ▲, ▼ and ENTER,
    /// which commit the entry and move, whether the move is down.
    pub fn perform(&mut self, action: Action) -> Option<bool> {
        match action {
            Action::Insert(token) => self.editor.type_token(token),
            Action::Edit(edit) => self.editor.apply(edit),
            Action::Command(Command::Up) => return Some(false),
            Action::Command(Command::Down | Command::Enter) => return Some(true),
            Action::Command(_) => {}
        }
        None
    }

    /// The entry's value, angles in `angle` units; `None` for an empty
    /// entry, which keeps the field's value.
    pub fn value(&self, memory: &Memory, angle: Angle) -> Result<Option<f64>, CalcError> {
        let tokens = self.editor.tokens();
        if tokens.is_empty() {
            return Ok(None);
        }
        let statement = parse(tokens)?;
        if statement.store.is_some() {
            return Err(CalcError::new(ErrorKind::Syntax));
        }
        evaluate(&statement.expr, memory, angle).map(Some)
    }
}

/// The tokens that type a formatted number (`⁻1.5ᴇ3`).
fn number_tokens(text: &str) -> Vec<Token> {
    const DIGITS: [Digit; 10] = [
        Digit::Zero,
        Digit::One,
        Digit::Two,
        Digit::Three,
        Digit::Four,
        Digit::Five,
        Digit::Six,
        Digit::Seven,
        Digit::Eight,
        Digit::Nine,
    ];
    text.chars()
        .filter_map(|c| match c {
            '.' => Some(Token::DecimalPoint),
            '⁻' => Some(Token::Negate),
            'ᴇ' => Some(Token::Exponent),
            c => c.to_digit(10).map(|d| Token::Digit(DIGITS[d as usize])),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Probe {
        First,
        Last,
    }

    impl Field for Probe {
        const TITLE: &'static str = "PROBE";
        const ALL: &'static [Self] = &[Probe::First, Probe::Last];

        fn label(self) -> &'static str {
            "probe"
        }
    }

    fn text(cursor: &FieldCursor<Probe>) -> String {
        cursor.editor.tokens().iter().map(|t| t.text()).collect()
    }

    #[test]
    fn a_field_opens_with_its_value_typed_out() {
        let cursor = FieldCursor::new(Probe::First, -1.5e-3, Settings::default());
        assert_eq!(text(&cursor), "⁻.0015");
        let tiny = FieldCursor::new(Probe::First, -1.5e-9, Settings::default());
        assert_eq!(text(&tiny), "⁻1.5ᴇ⁻9");
    }

    #[test]
    fn the_entry_evaluates_as_an_expression_and_empty_keeps_the_value() {
        let mut cursor = FieldCursor::new(Probe::First, 0.0, Settings::default());
        cursor
            .editor
            .replace(vec![Token::Pi, Token::Divide, Token::Digit(Digit::Two)]);
        let value = cursor
            .value(&Memory::default(), Angle::Radian)
            .unwrap()
            .unwrap();
        assert!((value - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
        cursor.editor.replace(Vec::new());
        assert_eq!(cursor.value(&Memory::default(), Angle::Radian), Ok(None));
    }

    #[test]
    fn fields_stop_at_the_ends() {
        let cursor = FieldCursor::new(Probe::Last, 0.0, Settings::default());
        assert_eq!(cursor.neighbor(true), Probe::Last);
        assert_eq!(cursor.neighbor(false), Probe::First);
    }
}
