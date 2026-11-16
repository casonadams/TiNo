use crate::{
    editor::Editor,
    error::{CalcError, ErrorKind},
    eval::Context,
    format::{format_real, result_tokens},
    keymap::{Action, Command},
    parse::parse,
    settings::Settings,
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
    /// On `field`, its entry showing `value` in Normal Float, whatever the
    /// mode, so the field holds every digit it has.
    pub fn new(field: F, value: f64) -> Self {
        let mut editor = Editor::default();
        editor.replace(result_tokens(&format_real(value, Settings::default())));
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

    /// The entry's value in `context`; `None` for an empty entry, which
    /// keeps the field's value.
    pub fn value(&self, context: &Context<'_>) -> Result<Option<f64>, CalcError> {
        let tokens = self.editor.tokens();
        if tokens.is_empty() {
            return Ok(None);
        }
        let statement = parse(tokens)?;
        if statement.store.is_some() {
            return Err(CalcError::new(ErrorKind::Syntax));
        }
        context.evaluate(&statement.expr).map(Some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

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
        let cursor = FieldCursor::new(Probe::First, -1.5e-3);
        assert_eq!(text(&cursor), "⁻.0015");
        let tiny = FieldCursor::new(Probe::First, -1.5e-9);
        assert_eq!(text(&tiny), "⁻1.5ᴇ⁻9");
    }

    #[test]
    fn the_entry_evaluates_as_an_expression_and_empty_keeps_the_value() {
        let mut cursor = FieldCursor::new(Probe::First, 0.0);
        cursor.editor.replace(vec![
            Token::Pi,
            Token::Op(crate::vocabulary::Operator::Divide),
            Token::Digit(crate::token::Digit::Two),
        ]);
        let context = Context {
            memory: &crate::eval::Memory::default(),
            angle: crate::settings::Angle::Radian,
            window: &crate::window::Window::standard(),
            table: &crate::table::TableSettings::default(),
            functions: Vec::new(),
        };
        let value = cursor.value(&context).unwrap().unwrap();
        assert!((value - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
        cursor.editor.replace(Vec::new());
        assert_eq!(cursor.value(&context), Ok(None));
    }

    #[test]
    fn fields_stop_at_the_ends() {
        let cursor = FieldCursor::new(Probe::Last, 0.0);
        assert_eq!(cursor.neighbor(true), Probe::Last);
        assert_eq!(cursor.neighbor(false), Probe::First);
    }
}
