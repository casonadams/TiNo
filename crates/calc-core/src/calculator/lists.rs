use super::{Calculator, View, error_view, free::arrow_step};
use crate::{
    editor::Editor,
    error::CalcError,
    error_screen::ErrorOrigin,
    format::format_real,
    keymap::{Action, Command, Edit},
    list_editor::{EDITOR_COLUMNS, EDITOR_ROWS, ListEditor},
    parse::parse,
    screen::TableView,
    token::{ListName, Token},
};

impl Calculator {
    /// STAT EDIT: arrows move between cells (ending what is typed), typing
    /// edits the selected cell, ENTER stores it and moves down, DEL removes
    /// the element, CLEAR empties what is typed.
    pub(super) fn perform_list_editor(&mut self, mut editor: ListEditor, action: Action) -> View {
        if let Some(step) = arrow_step(action) {
            let length = |name: ListName| self.memory.list(name).len();
            return View::Lists(editor.stepped((step.0, -step.1), length));
        }
        match action {
            Action::Command(Command::Enter) => return self.store_cell(editor),
            Action::Insert(token) => editor
                .entry
                .get_or_insert_with(Editor::default)
                .type_token(token),
            Action::Edit(Edit::Delete) if editor.entry.is_none() => self.delete_cell(&editor),
            Action::Edit(edit) => {
                if let Some(entry) = editor.entry.as_mut() {
                    entry.apply(edit);
                }
            }
            Action::Command(_) => {}
        }
        View::Lists(editor)
    }

    /// ENTER: the typed value into the selected element (a new last element
    /// in the row after the last), then the row below.
    fn store_cell(&mut self, editor: ListEditor) -> View {
        let Some(entry) = &editor.entry else {
            return View::Lists(editor.stepped((0, 1), |name| self.memory.list(name).len()));
        };
        let value = match self.cell_value(entry.tokens()) {
            Ok(value) => value,
            Err(error) => return error_view(error, ErrorOrigin::ListEditor(editor)),
        };
        let name = editor.list();
        let mut list = self.memory.list(name).to_vec();
        match list.get_mut(editor.row) {
            Some(element) => *element = value,
            None => list.push(value),
        }
        self.memory.set_list(name, list);
        View::Lists(editor.stepped((0, 1), |name| self.memory.list(name).len()))
    }

    fn cell_value(&self, tokens: &[Token]) -> Result<f64, CalcError> {
        self.context().evaluate_real(&parse(tokens)?.expr)
    }

    fn delete_cell(&mut self, editor: &ListEditor) {
        let name = editor.list();
        let mut list = self.memory.list(name).to_vec();
        if editor.row < list.len() {
            list.remove(editor.row);
            self.memory.set_list(name, list);
        }
    }

    /// The lists shown as a table: their names, their elements, the
    /// selected cell, and `L₁(3)=` with the element or what is typed.
    pub(super) fn list_editor_view(&self, editor: &ListEditor) -> TableView {
        let names = &ListName::ALL[editor.first_column..editor.first_column + EDITOR_COLUMNS];
        let cell = |name: &ListName, row: usize| {
            let list = self.memory.list(*name);
            list.get(row)
                .map_or_else(String::new, |&x| format_real(x, self.settings))
        };
        let rows = (editor.first_row..editor.first_row + EDITOR_ROWS)
            .map(|row| names.iter().map(|name| cell(name, row)).collect());
        let name = Token::List(editor.list()).text();
        let label = format!("{name}({})=", editor.row + 1);
        let shown = cell(&editor.list(), editor.row);
        TableView {
            headers: names
                .iter()
                .map(|&name| Token::List(name).text().to_owned())
                .collect(),
            rows: rows.collect(),
            selected_row: editor.row - editor.first_row,
            selected_column: editor.column - editor.first_column,
            footer: match editor.entry {
                Some(_) => label,
                None => format!("{label}{shown}"),
            },
            editing: editor
                .entry
                .as_ref()
                .map(|entry| (entry.tokens().to_vec(), entry.cursor())),
        }
    }
}
