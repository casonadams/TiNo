use crate::{keymap::Edit, token::Token};

/// The entry line: tokens and a cursor that sits on a token or after the
/// last one. Typing replaces the token under the cursor unless insert mode
/// (2nd INS) is on; moving the cursor turns insert mode off.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Editor {
    tokens: Vec<Token>,
    cursor: usize,
    inserting: bool,
}

impl Editor {
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// The index of the token under the cursor; `tokens().len()` after the
    /// last one.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn is_inserting(&self) -> bool {
        self.inserting
    }

    pub fn type_token(&mut self, token: Token) {
        if self.inserting || self.cursor == self.tokens.len() {
            self.tokens.insert(self.cursor, token);
        } else {
            self.tokens[self.cursor] = token;
        }
        self.cursor += 1;
    }

    pub fn apply(&mut self, edit: Edit) {
        match edit {
            Edit::Delete => self.delete(),
            Edit::ToggleInsert => self.inserting = !self.inserting,
            Edit::Clear => *self = Editor::default(),
            Edit::Left => self.move_to(self.cursor.saturating_sub(1)),
            Edit::Right => self.move_to((self.cursor + 1).min(self.tokens.len())),
        }
    }

    /// Removes the token under the cursor, or the last token when the cursor
    /// is after it.
    fn delete(&mut self) {
        if self.cursor < self.tokens.len() {
            self.tokens.remove(self.cursor);
        } else if self.tokens.pop().is_some() {
            self.cursor = self.tokens.len();
        }
    }

    fn move_to(&mut self, cursor: usize) {
        self.cursor = cursor;
        self.inserting = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Digit;

    const ONE: Token = Token::Digit(Digit::One);
    const TWO: Token = Token::Digit(Digit::Two);
    const THREE: Token = Token::Digit(Digit::Three);

    fn typed(tokens: &[Token]) -> Editor {
        let mut editor = Editor::default();
        tokens.iter().for_each(|&t| editor.type_token(t));
        editor
    }

    #[test]
    fn typing_appends_at_the_end() {
        let editor = typed(&[ONE, Token::Add, TWO]);
        assert_eq!(editor.tokens(), [ONE, Token::Add, TWO]);
        assert_eq!(editor.cursor(), 3);
    }

    #[test]
    fn typing_over_a_token_replaces_it() {
        let mut editor = typed(&[ONE, TWO]);
        editor.apply(Edit::Left);
        editor.apply(Edit::Left);
        editor.type_token(THREE);
        assert_eq!(editor.tokens(), [THREE, TWO]);
        assert_eq!(editor.cursor(), 1);
    }

    #[test]
    fn insert_mode_inserts_until_the_cursor_moves() {
        let mut editor = typed(&[ONE, TWO]);
        editor.apply(Edit::Left);
        editor.apply(Edit::ToggleInsert);
        editor.type_token(THREE);
        assert_eq!(editor.tokens(), [ONE, THREE, TWO]);
        assert!(editor.is_inserting());
        editor.apply(Edit::Right);
        assert!(!editor.is_inserting());
    }

    #[test]
    fn delete_removes_under_the_cursor_or_the_last_token() {
        let mut editor = typed(&[ONE, TWO, THREE]);
        editor.apply(Edit::Delete);
        assert_eq!(editor.tokens(), [ONE, TWO]);
        assert_eq!(editor.cursor(), 2);
        editor.apply(Edit::Left);
        editor.apply(Edit::Left);
        editor.apply(Edit::Delete);
        assert_eq!(editor.tokens(), [TWO]);
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn the_cursor_stays_within_the_line() {
        let mut editor = typed(&[ONE]);
        editor.apply(Edit::Right);
        assert_eq!(editor.cursor(), 1);
        editor.apply(Edit::Left);
        editor.apply(Edit::Left);
        assert_eq!(editor.cursor(), 0);
    }

    #[test]
    fn clear_empties_the_line_and_leaves_insert_mode() {
        let mut editor = typed(&[ONE, TWO]);
        editor.apply(Edit::ToggleInsert);
        editor.apply(Edit::Clear);
        assert_eq!(editor, Editor::default());
    }
}
