use std::collections::VecDeque;

use crate::{
    editor::Editor,
    error::CalcError,
    eval::{Memory, evaluate},
    format::format_real,
    home::{History, HistoryLine, Output},
    key::Key,
    keymap::{self, Action, Command, Edit, Layer},
    latex,
    modifiers::{AlphaState, Modifiers},
    parse::parse,
    token::Token,
};

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Screen<'a> {
    pub history: &'a VecDeque<HistoryLine>,
    pub entry: &'a [Token],
    pub cursor: usize,
    pub cursor_style: CursorStyle,
    /// The error the last ENTER raised, until the next key press.
    pub error: Option<CalcError>,
}

/// The calculator: key presses in, a screen out.
#[derive(Debug, Clone, Default)]
pub struct Calculator {
    modifiers: Modifiers,
    editor: Editor,
    memory: Memory,
    history: History,
    error: Option<CalcError>,
    /// How far back the last 2nd ENTRY reached, while ENTRY keeps being pressed.
    recall: Option<usize>,
}

impl Calculator {
    pub fn press(&mut self, key: Key) {
        self.error = None;
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
            history: self.history.lines(),
            entry: self.editor.tokens(),
            cursor: self.editor.cursor(),
            cursor_style: self.cursor_style(),
            error: self.error,
        }
    }

    fn perform(&mut self, action: Action, recalled: Option<usize>) {
        match action {
            Action::Insert(token) => self.insert(token),
            Action::Edit(Edit::Clear) if self.editor.tokens().is_empty() => self.history.clear(),
            Action::Edit(edit) => self.editor.apply(edit),
            Action::Command(Command::Enter) => self.enter(),
            Action::Command(Command::Entry) => self.recall_entry(recalled),
            Action::Command(_) => {}
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
    /// empty. An error leaves the entry on the line with the cursor at it.
    fn enter(&mut self) {
        let entry = match (self.editor.tokens(), self.history.recall(0)) {
            ([], Some(previous)) => previous.to_vec(),
            ([], None) => return,
            (tokens, _) => tokens.to_vec(),
        };
        match self.run(&entry) {
            Ok(output) => {
                self.history.push(entry, output);
                self.editor.apply(Edit::Clear);
            }
            Err(error) => {
                self.editor.replace(entry);
                if let Some(position) = error.position {
                    self.editor.place_cursor(position);
                }
                self.error = Some(error);
            }
        }
    }

    fn run(&mut self, entry: &[Token]) -> Result<Output, CalcError> {
        let statement = parse(entry)?;
        let value = evaluate(&statement.expr, &self.memory)?;
        if let Some(letter) = statement.store {
            self.memory.set(letter, value);
        }
        self.memory.set_ans(value);
        Ok(Output {
            latex: latex::statement(&statement),
            result: format_real(value),
        })
    }

    /// 2nd ENTRY: the newest entry, then each older one on every further press.
    fn recall_entry(&mut self, recalled: Option<usize>) {
        let back = recalled.map_or(0, |back| back + 1);
        if let Some(entry) = self.history.recall(back) {
            self.editor.replace(entry.to_vec());
            self.recall = Some(back);
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
