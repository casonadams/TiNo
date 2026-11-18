use super::{Calculator, HOME, HomeState, View, error_view};
use crate::{
    error::CalcError,
    error_screen::ErrorOrigin,
    expr::StoreTarget,
    format::format_value,
    home::{HistorySelection, Output},
    keymap::{Action, Command, Edit},
    latex,
    parse::parse,
    token::Token,
};

impl Calculator {
    pub(super) fn perform_home(&mut self, action: Action, state: HomeState) -> View {
        if let Some(selected) = state.selected {
            return self.perform_selection(selected, action);
        }
        match action {
            Action::Insert(token) => self.insert(token),
            Action::Edit(Edit::Clear) if self.editor.tokens().is_empty() => self.history.clear(),
            Action::Edit(edit) => self.editor.apply(edit),
            Action::Command(Command::Enter) => return self.enter(),
            Action::Command(Command::Entry) => {
                let recalled = self.recall_entry(state.recalled);
                return View::Home(HomeState {
                    recalled,
                    selected: None,
                });
            }
            Action::Command(Command::Up) => {
                return selecting(HistorySelection::newest(self.history.lines().len()));
            }
            Action::Command(_) => {}
        }
        HOME
    }

    /// With a history entry or result highlighted: ▲ ▼ move the highlight,
    /// ENTER types it at the cursor, CLEAR drops the highlight; any other
    /// key drops it and acts as usual.
    fn perform_selection(&mut self, selected: HistorySelection, action: Action) -> View {
        match action {
            Action::Command(Command::Up) => selecting(Some(selected.up())),
            Action::Command(Command::Down) => selecting(selected.down(self.history.lines().len())),
            Action::Command(Command::Enter) => {
                for token in self.history.pasted(selected) {
                    self.editor.type_token(token);
                }
                HOME
            }
            Action::Edit(Edit::Clear) => HOME,
            _ => self.perform_home(action, HomeState::default()),
        }
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
        let value = self.context().evaluate(&statement.expr)?;
        match statement.store {
            Some(StoreTarget::Variable(letter)) => self.memory.set(letter, value.clone().real()?),
            Some(StoreTarget::List(name)) => self.memory.set_list(name, value.clone().list()?),
            None => {}
        }
        let result = format_value(&value, self.settings, statement.display);
        self.memory.set_ans(value);
        Ok(Output {
            entry: entry.to_vec(),
            latex: latex::statement(&statement),
            result,
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

fn selecting(selected: Option<HistorySelection>) -> View {
    View::Home(HomeState {
        recalled: None,
        selected,
    })
}
