use super::{Calculator, HOME, View, error_view};
use crate::{
    error::CalcError,
    error_screen::ErrorOrigin,
    eval::evaluate,
    format::format_real,
    home::Output,
    keymap::{Action, Command, Edit},
    latex,
    parse::parse,
    token::Token,
};

impl Calculator {
    pub(super) fn perform_home(&mut self, action: Action, recalled: Option<usize>) -> View {
        match action {
            Action::Insert(token) => self.insert(token),
            Action::Edit(Edit::Clear) if self.editor.tokens().is_empty() => self.history.clear(),
            Action::Edit(edit) => self.editor.apply(edit),
            Action::Command(Command::Enter) => return self.enter(),
            Action::Command(Command::Entry) => return View::Home(self.recall_entry(recalled)),
            Action::Command(_) => {}
        }
        HOME
    }

    /// Types `token`; an operator typed first on the line applies to `Ans`.
    fn insert(&mut self, token: Token) {
        if self.editor.tokens().is_empty() && token.takes_left_operand() {
            self.editor.type_token(Token::Ans);
        }
        self.editor.type_token(token);
    }

    /// Evaluates the entry line, or the newest entry again when the line is
    /// empty. An error opens the error screen.
    fn enter(&mut self) -> View {
        let entry = match (self.editor.tokens(), self.entries.recall(0)) {
            ([], Some(previous)) => previous.to_vec(),
            ([], None) => return HOME,
            (tokens, _) => tokens.to_vec(),
        };
        self.entries.push(entry.clone());
        self.editor.apply(Edit::Clear);
        match self.run(&entry) {
            Ok(output) => {
                self.history.push(output);
                HOME
            }
            Err(error) => error_view(error, ErrorOrigin::Home(entry)),
        }
    }

    fn run(&mut self, entry: &[Token]) -> Result<Output, CalcError> {
        let statement = parse(entry)?;
        let value = evaluate(&statement.expr, &self.memory, self.settings.angle)?;
        if let Some(letter) = statement.store {
            self.memory.set(letter, value);
        }
        self.memory.set_ans(value);
        Ok(Output {
            latex: latex::statement(&statement),
            result: format_real(value, self.settings),
        })
    }

    /// 2nd ENTRY: the newest entry, then each older one on every further
    /// press; how far back it reached.
    fn recall_entry(&mut self, recalled: Option<usize>) -> Option<usize> {
        let back = recalled.map_or(0, |back| back + 1);
        let entry = self.entries.recall(back)?;
        self.editor.replace(entry.to_vec());
        Some(back)
    }
}
