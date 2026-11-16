use crate::{
    functions::{FUNCTION_COUNT, Functions},
    keymap::{Action, Command, Edit},
};

/// The Y= screen's cursor: the focused function, in its entry or on its
/// `=` sign, where ENTER turns graphing the function on or off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct YCursor {
    pub row: usize,
    pub on_selector: bool,
}

impl YCursor {
    /// On `row`'s entry, its cursor at `position` (clamped to the entry).
    pub fn at(row: usize, position: usize, functions: &mut Functions) -> Self {
        functions.slot_mut(row).editor.place_cursor(position);
        YCursor {
            row,
            on_selector: false,
        }
    }

    /// What `action` does on Y=; the cursor after it.
    pub fn perform(self, action: Action, functions: &mut Functions) -> Self {
        match action {
            Action::Edit(Edit::Clear) => {
                functions.slot_mut(self.row).editor.apply(Edit::Clear);
                self
            }
            Action::Command(Command::Up) => self.to_row(self.row.saturating_sub(1), functions),
            Action::Command(Command::Down) => self.to_row(self.row + 1, functions),
            _ if self.on_selector => self.perform_on_selector(action, functions),
            _ => self.perform_in_entry(action, functions),
        }
    }

    fn perform_on_selector(self, action: Action, functions: &mut Functions) -> Self {
        match action {
            Action::Command(Command::Enter) => {
                let slot = functions.slot_mut(self.row);
                slot.enabled = !slot.enabled;
                self
            }
            Action::Edit(Edit::Right) => Self::at(self.row, 0, functions),
            _ => self,
        }
    }

    /// Typing into an empty function turns graphing it on.
    fn perform_in_entry(self, action: Action, functions: &mut Functions) -> Self {
        let slot = functions.slot_mut(self.row);
        match action {
            Action::Insert(token) => {
                slot.enabled |= slot.editor.tokens().is_empty();
                slot.editor.type_token(token);
            }
            Action::Edit(Edit::Left) if slot.editor.cursor() == 0 => {
                return YCursor {
                    on_selector: true,
                    ..self
                };
            }
            Action::Edit(edit) => slot.editor.apply(edit),
            Action::Command(Command::Enter) => return self.to_row(self.row + 1, functions),
            Action::Command(_) => {}
        }
        self
    }

    /// Onto `row` (clamped to the last function), at the end of its entry.
    fn to_row(self, row: usize, functions: &mut Functions) -> Self {
        let row = row.min(FUNCTION_COUNT - 1);
        Self::at(row, usize::MAX, functions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Letter, Token};

    fn typed(functions: &mut Functions, cursor: YCursor, token: Token) -> YCursor {
        cursor.perform(Action::Insert(token), functions)
    }

    #[test]
    fn typing_into_an_empty_function_turns_it_on() {
        let mut functions = Functions::default();
        typed(&mut functions, YCursor::default(), Token::Letter(Letter::X));
        assert_eq!(functions.graphed().collect::<Vec<_>>(), [0]);
    }

    #[test]
    fn enter_on_the_equals_sign_toggles_graphing() {
        let mut functions = Functions::default();
        let cursor = typed(&mut functions, YCursor::default(), Token::Letter(Letter::X));
        let cursor = cursor.perform(Action::Edit(Edit::Left), &mut functions);
        let cursor = cursor.perform(Action::Edit(Edit::Left), &mut functions);
        assert!(cursor.on_selector);
        cursor.perform(Action::Command(Command::Enter), &mut functions);
        assert_eq!(functions.graphed().count(), 0);
    }

    #[test]
    fn enter_in_an_entry_moves_to_the_next_function_and_stops_at_the_last() {
        let mut functions = Functions::default();
        let enter = Action::Command(Command::Enter);
        let last = (0..12).fold(YCursor::default(), |c, _| c.perform(enter, &mut functions));
        assert_eq!(last.row, FUNCTION_COUNT - 1);
    }
}
