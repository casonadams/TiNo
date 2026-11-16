use crate::{
    editor::Editor,
    error::CalcError,
    error_screen::{ErrorChoice, ErrorMenu},
    eval::{Memory, evaluate},
    format::format_real,
    home::{EntryStack, History, Output},
    key::Key,
    keymap::{self, Action, Command, Edit, Layer},
    latex,
    mode_screen::{ModeCursor, ModeRow},
    modifiers::{AlphaState, Modifiers},
    parse::parse,
    screen::{CursorStyle, ErrorView, HomeView, ModeRowView, ModeView, Screen, ScreenView},
    settings::Settings,
    token::{Digit, Token},
};

/// The screen being shown, with its own state.
#[derive(Debug, Clone, Default)]
enum View {
    #[default]
    Home,
    Mode(ModeCursor),
    Error(ErrorMenu),
}

/// The calculator: key presses in, a screen out.
#[derive(Debug, Clone, Default)]
pub struct Calculator {
    modifiers: Modifiers,
    view: View,
    editor: Editor,
    settings: Settings,
    memory: Memory,
    history: History,
    entries: EntryStack,
    /// How far back the last 2nd ENTRY reached, while ENTRY keeps being pressed.
    recall: Option<usize>,
}

impl Calculator {
    pub fn press(&mut self, key: Key) {
        match key {
            Key::Second => self.modifiers.press_second(),
            Key::Alpha => self.modifiers.press_alpha(),
            _ => {
                let action = keymap::action(key, self.modifiers.layer());
                self.modifiers.release();
                let recalled = self.recall.take();
                if let Some(action) = action {
                    self.perform(action, recalled);
                }
            }
        }
    }

    pub fn screen(&self) -> Screen<'_> {
        Screen {
            status: self.settings.labels(),
            cursor_style: self.cursor_style(),
            view: match &self.view {
                View::Home => ScreenView::Home(HomeView {
                    history: self.history.lines(),
                    entry: self.editor.tokens(),
                    cursor: self.editor.cursor(),
                }),
                View::Mode(cursor) => ScreenView::Mode(self.mode_view(*cursor)),
                View::Error(menu) => ScreenView::Error(ErrorView {
                    message: menu.error.message(),
                    choices: ErrorChoice::ALL,
                    highlighted: menu.highlighted,
                }),
            },
        }
    }

    fn perform(&mut self, action: Action, recalled: Option<usize>) {
        match std::mem::take(&mut self.view) {
            View::Home => self.perform_home(action, recalled),
            View::Mode(cursor) => self.view = self.perform_mode(cursor, action),
            View::Error(menu) => self.view = self.perform_error(menu, action),
        }
    }

    fn perform_home(&mut self, action: Action, recalled: Option<usize>) {
        match action {
            Action::Insert(token) => self.insert(token),
            Action::Edit(Edit::Clear) if self.editor.tokens().is_empty() => self.history.clear(),
            Action::Edit(edit) => self.editor.apply(edit),
            Action::Command(Command::Enter) => self.enter(),
            Action::Command(Command::Entry) => self.recall_entry(recalled),
            Action::Command(Command::Mode) => {
                self.view = View::Mode(ModeCursor::new(self.settings))
            }
            Action::Command(_) => {}
        }
    }

    fn perform_mode(&mut self, cursor: ModeCursor, action: Action) -> View {
        let cursor = match action {
            Action::Command(Command::Quit) | Action::Edit(Edit::Clear) => return View::Home,
            Action::Command(Command::Up) => cursor.step_row(false, self.settings),
            Action::Command(Command::Down) => cursor.step_row(true, self.settings),
            Action::Edit(Edit::Left) => cursor.step_column(false),
            Action::Edit(Edit::Right) => cursor.step_column(true),
            Action::Command(Command::Enter) => {
                cursor.choose(&mut self.settings);
                cursor
            }
            _ => cursor,
        };
        View::Mode(cursor)
    }

    fn perform_error(&mut self, menu: ErrorMenu, action: Action) -> View {
        let choice = match action {
            Action::Command(Command::Up | Command::Down) => return View::Error(menu.toggled()),
            Action::Command(Command::Enter) => menu.highlighted,
            Action::Command(Command::Quit) | Action::Insert(Token::Digit(Digit::One)) => {
                ErrorChoice::Quit
            }
            Action::Insert(Token::Digit(Digit::Two)) => ErrorChoice::Goto,
            _ => return View::Error(menu),
        };
        if choice == ErrorChoice::Goto {
            self.editor.replace(menu.entry);
            self.editor
                .place_cursor(menu.error.position.unwrap_or(usize::MAX));
        }
        View::Home
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
    fn enter(&mut self) {
        let entry = match (self.editor.tokens(), self.entries.recall(0)) {
            ([], Some(previous)) => previous.to_vec(),
            ([], None) => return,
            (tokens, _) => tokens.to_vec(),
        };
        self.entries.push(entry.clone());
        self.editor.apply(Edit::Clear);
        match self.run(&entry) {
            Ok(output) => self.history.push(output),
            Err(error) => self.view = View::Error(ErrorMenu::new(error, entry)),
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

    /// 2nd ENTRY: the newest entry, then each older one on every further press.
    fn recall_entry(&mut self, recalled: Option<usize>) {
        let back = recalled.map_or(0, |back| back + 1);
        if let Some(entry) = self.entries.recall(back) {
            self.editor.replace(entry.to_vec());
            self.recall = Some(back);
        }
    }

    fn mode_view(&self, cursor: ModeCursor) -> ModeView {
        let rows = ModeRow::ALL.map(|row| ModeRowView {
            row,
            options: row.options(),
            selected: row.selected(self.settings),
            highlighted: (row == cursor.row()).then_some(cursor.column()),
        });
        ModeView {
            rows: rows.to_vec(),
        }
    }

    fn cursor_style(&self) -> CursorStyle {
        match (self.modifiers.layer(), self.modifiers.alpha()) {
            (Layer::Second, _) => CursorStyle::Second,
            (Layer::Alpha, AlphaState::Locked) => CursorStyle::AlphaLock,
            (Layer::Alpha, _) => CursorStyle::Alpha,
            (Layer::Normal, _) if self.editor.is_inserting() => CursorStyle::Insert,
            (Layer::Normal, _) => CursorStyle::Overwrite,
        }
    }
}
