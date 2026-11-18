use std::borrow::Cow;

use super::{Calculator, HOME, OpenMenu, View};
use crate::{
    editor::Editor,
    keymap::{Action, Command, Edit},
    menus::{MenuCursor, MenuKind, menu_key},
    program::{self, LONGEST_NAME, Program},
    screen::{FieldRowView, FieldsView, MenuItemView, MenuView, ProgramLineView, ProgramView},
    token::{Digit, Letter, Token},
};

/// PRGM's tabs on the home screen.
const TABS: [&str; 3] = ["EXEC", "EDIT", "NEW"];
const NEW: usize = 2;

/// PRGM on the home screen: EXEC types a program's name to run it, EDIT
/// opens it, NEW names a new one.
#[derive(Debug, Clone, Copy, Default)]
pub(super) struct ProgramMenu {
    tab: usize,
    highlighted: usize,
}

/// The program editor: the program's name, the line the cursor is on, and
/// that line being edited.
#[derive(Debug, Clone)]
pub(super) struct ProgramCursor {
    name: String,
    line: usize,
    editor: Editor,
}

/// The key that picks the `index`th item of a list as long as the
/// programs: 1 to 9, 0, then A to Z.
fn item_key(index: usize) -> char {
    const KEYS: &str = "1234567890ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    KEYS.chars().nth(index).unwrap_or(' ')
}

impl Calculator {
    /// The items on `tab`: the programs, or NEW's one item.
    fn program_items(&self, tab: usize) -> usize {
        if tab == NEW { 1 } else { self.programs.len() }
    }

    /// ◀ ▶ change tabs, ▲ ▼ items; ENTER or an item's key picks; CLEAR
    /// returns home.
    pub(super) fn perform_program_menu(&mut self, menu: ProgramMenu, action: Action) -> View {
        let count = self.program_items(menu.tab).max(1);
        let picked = match action {
            Action::Edit(Edit::Clear) => return HOME,
            Action::Edit(Edit::Left | Edit::Right) => {
                let step = if action == Action::Edit(Edit::Left) {
                    2
                } else {
                    1
                };
                return View::Programs(ProgramMenu {
                    tab: (menu.tab + step) % TABS.len(),
                    highlighted: 0,
                });
            }
            Action::Command(Command::Up | Command::Down) => {
                let step = if action == Action::Command(Command::Up) {
                    count - 1
                } else {
                    1
                };
                return View::Programs(ProgramMenu {
                    highlighted: (menu.highlighted + step) % count,
                    ..menu
                });
            }
            Action::Command(Command::Enter) => Some(menu.highlighted),
            Action::Insert(token) => {
                menu_key(token).and_then(|key| (0..count).find(|&index| item_key(index) == key))
            }
            _ => None,
        };
        match picked {
            Some(index) if index < self.program_items(menu.tab) => {
                self.pick_program(menu.tab, index)
            }
            _ => View::Programs(menu),
        }
    }

    fn pick_program(&mut self, tab: usize, index: usize) -> View {
        match tab {
            NEW => {
                self.modifiers.lock_alpha();
                View::ProgramName(Editor::default())
            }
            1 => self.program_editor(self.programs[index].name.clone(), 0),
            _ => {
                let name = self.programs[index].name.clone();
                self.editor.type_token(Token::Prgm);
                for token in program_name_tokens(&name) {
                    self.editor.type_token(token);
                }
                HOME
            }
        }
    }

    /// NEW's name prompt, ALPHA locked: letters, and digits after the
    /// first, up to eight; ENTER (ALPHA ENTER while locked) opens the program
    /// of that name, made empty when new; CLEAR on an empty name leaves.
    pub(super) fn perform_program_name(&mut self, mut editor: Editor, action: Action) -> View {
        match action {
            Action::Insert(token @ (Token::Letter(_) | Token::Digit(_)))
                if editor.tokens().len() < LONGEST_NAME =>
            {
                editor.type_token(token);
            }
            Action::Edit(Edit::Clear) if editor.tokens().is_empty() => {
                self.modifiers = Default::default();
                return HOME;
            }
            Action::Edit(edit) => editor.apply(edit),
            Action::Command(Command::Enter | Command::Solve) => {
                if let Some(name) = program::name(editor.tokens()) {
                    return self.open_program(name);
                }
            }
            _ => {}
        }
        View::ProgramName(editor)
    }

    fn open_program(&mut self, name: String) -> View {
        self.modifiers = Default::default();
        if let Err(at) = self
            .programs
            .binary_search_by(|program| program.name.cmp(&name))
        {
            let lines = vec![Vec::new()];
            self.programs.insert(
                at,
                Program {
                    name: name.clone(),
                    lines,
                },
            );
        }
        self.program_editor(name, 0)
    }

    /// The editor on line `line` of the program named `name`.
    pub(super) fn program_editor(&self, name: String, line: usize) -> View {
        let mut editor = Editor::default();
        if let Some(tokens) = self
            .program(&name)
            .and_then(|program| program.lines.get(line))
        {
            editor.replace(tokens.clone());
            editor.place_cursor(0);
        }
        View::ProgramEditor(ProgramCursor { name, line, editor })
    }

    pub(super) fn program(&self, name: &str) -> Option<&Program> {
        self.programs.iter().find(|program| program.name == name)
    }

    /// ▲ ▼ change lines, ENTER starts a new line below, DEL on an empty line
    /// removes it, PRGM opens the commands; the rest edits the line.
    pub(super) fn perform_program_editor(
        &mut self,
        mut cursor: ProgramCursor,
        action: Action,
    ) -> View {
        let line = cursor.line;
        match action {
            Action::Command(Command::Up) => return self.move_line(cursor, line.saturating_sub(1)),
            Action::Command(Command::Down) => return self.move_line(cursor, line + 1),
            Action::Command(Command::Enter) => return self.new_line(cursor),
            Action::Command(Command::Prgm) => {
                return View::Menu(OpenMenu {
                    cursor: MenuCursor::new(MenuKind::Program),
                    under: Box::new(View::ProgramEditor(cursor)),
                });
            }
            Action::Edit(Edit::Delete) if cursor.editor.tokens().is_empty() => {
                return self.remove_line(cursor);
            }
            Action::Insert(token) => cursor.editor.type_token(token),
            Action::Edit(edit) => cursor.editor.apply(edit),
            Action::Command(_) => {}
        }
        self.commit_line(&cursor);
        View::ProgramEditor(cursor)
    }

    /// Keeps the line being edited in the program, as each key edits it.
    fn commit_line(&mut self, cursor: &ProgramCursor) {
        let program = self
            .programs
            .iter_mut()
            .find(|program| program.name == cursor.name);
        if let Some(line) = program.and_then(|program| program.lines.get_mut(cursor.line)) {
            *line = cursor.editor.tokens().to_vec();
        }
    }

    fn line_count(&self, name: &str) -> usize {
        self.program(name).map_or(0, |program| program.lines.len())
    }

    fn move_line(&mut self, cursor: ProgramCursor, line: usize) -> View {
        let last = self.line_count(&cursor.name).saturating_sub(1);
        self.program_editor(cursor.name, line.min(last))
    }

    fn new_line(&mut self, cursor: ProgramCursor) -> View {
        let at = cursor.line + 1;
        if let Some(program) = self.programs.iter_mut().find(|p| p.name == cursor.name) {
            program.lines.insert(at, Vec::new());
        }
        self.program_editor(cursor.name, at)
    }

    fn remove_line(&mut self, cursor: ProgramCursor) -> View {
        let program = self.programs.iter_mut().find(|p| p.name == cursor.name);
        if let Some(program) = program.filter(|program| program.lines.len() > 1) {
            program.lines.remove(cursor.line);
        }
        self.program_editor(cursor.name, cursor.line.saturating_sub(1))
    }

    pub(super) fn program_menu_view(&self, menu: ProgramMenu) -> MenuView {
        let items: Vec<MenuItemView> = match menu.tab {
            NEW => vec![MenuItemView {
                key: '1',
                label: Cow::Borrowed("Create New"),
            }],
            _ => (self.programs.iter().enumerate())
                .map(|(index, program)| MenuItemView {
                    key: item_key(index),
                    label: Cow::Owned(program.name.clone()),
                })
                .collect(),
        };
        MenuView {
            tabs: TABS.to_vec(),
            tab: menu.tab,
            items,
            highlighted: menu.highlighted,
        }
    }

    pub(super) fn program_name_view(editor: &Editor) -> FieldsView<'_> {
        FieldsView {
            title: "PROGRAM",
            rows: vec![FieldRowView {
                label: "Name=",
                value: String::new(),
                editing: Some((editor.tokens(), editor.cursor())),
            }],
        }
    }

    /// The program's lines, the one being edited with its cursor.
    pub(super) fn program_view<'a>(&'a self, cursor: &'a ProgramCursor) -> ProgramView<'a> {
        let lines = self
            .program(&cursor.name)
            .map_or(&[][..], |program| &program.lines);
        let line = |(index, tokens): (usize, &'a Vec<Token>)| match index == cursor.line {
            true => ProgramLineView {
                entry: cursor.editor.tokens(),
                cursor: Some(cursor.editor.cursor()),
            },
            false => ProgramLineView {
                entry: tokens,
                cursor: None,
            },
        };
        ProgramView {
            title: format!("PROGRAM:{}", cursor.name),
            lines: lines.iter().enumerate().map(line).collect(),
        }
    }
}

/// The tokens that spell a program's name.
fn program_name_tokens(name: &str) -> Vec<Token> {
    let letters = Letter::ALL.map(Token::Letter);
    let digits = Digit::ALL.map(Token::Digit);
    let spelled = |c: char| {
        let mut buffer = [0; 4];
        let text: &str = c.encode_utf8(&mut buffer);
        letters
            .iter()
            .chain(&digits)
            .find(|token| token.text() == text)
            .copied()
    };
    name.chars().filter_map(spelled).collect()
}
