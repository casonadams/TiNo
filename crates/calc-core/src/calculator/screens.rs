use super::{Calculator, HOME, OpenMenu, View, error_view};
use crate::{
    error_screen::{ErrorChoice, ErrorMenu, ErrorOrigin},
    field_editor::FieldCursor,
    keymap::{Action, Command, Edit},
    list_menu::ListMenu,
    menus::{MenuCursor, menu_key},
    token::{Digit, Token},
    window_editor::WindowField,
    y_editor::YCursor,
    zoom::ZoomItem,
};

impl Calculator {
    pub(super) fn perform_error(&mut self, menu: ErrorMenu, action: Action) -> View {
        let choice = match action {
            Action::Command(Command::Up | Command::Down) => return View::Error(menu.toggled()),
            Action::Command(Command::Enter) => menu.highlighted,
            Action::Command(Command::Quit) | Action::Insert(Token::Digit(Digit::One)) => {
                ErrorChoice::Quit
            }
            Action::Insert(Token::Digit(Digit::Two)) => ErrorChoice::Goto,
            _ => return View::Error(menu),
        };
        match choice {
            ErrorChoice::Quit => HOME,
            ErrorChoice::Goto => self.goto(menu),
        }
    }

    /// Goto: where the error was raised, the cursor at the fault.
    fn goto(&mut self, menu: ErrorMenu) -> View {
        let position = menu.error.position.unwrap_or(usize::MAX);
        match menu.origin {
            ErrorOrigin::Home(entry) => {
                self.editor.replace(entry);
                self.editor.place_cursor(position);
                HOME
            }
            ErrorOrigin::Function(slot) => {
                View::Functions(YCursor::at(slot, position, &mut self.functions))
            }
            ErrorOrigin::WindowField(field, entry) => {
                let mut cursor = self.window_cursor(field);
                cursor.editor.replace(entry);
                cursor.editor.place_cursor(position);
                View::Window(cursor)
            }
            ErrorOrigin::Window => View::Window(self.window_cursor(WindowField::XMin)),
            ErrorOrigin::Graph => self.graph(false),
            ErrorOrigin::TableField(field, entry) => {
                let mut cursor = self.table_setup_cursor(field);
                cursor.editor.replace(entry);
                cursor.editor.place_cursor(position);
                View::TableSetup(cursor)
            }
        }
    }

    /// The WINDOW cursor on `field`, showing its value.
    pub(super) fn window_cursor(&self, field: WindowField) -> FieldCursor<WindowField> {
        FieldCursor::new(field, field.get(&self.window))
    }

    pub(super) fn perform_window(
        &mut self,
        mut cursor: FieldCursor<WindowField>,
        action: Action,
    ) -> View {
        let Some(down) = cursor.perform(action) else {
            return View::Window(cursor);
        };
        match self.commit_window(&cursor) {
            Ok(()) => View::Window(self.window_cursor(cursor.neighbor(down))),
            Err(error) => error,
        }
    }

    /// Writes the WINDOW field's entry into the window; the error screen
    /// when it does not evaluate.
    pub(super) fn commit_window(&mut self, cursor: &FieldCursor<WindowField>) -> Result<(), View> {
        let value = cursor.value(&self.context());
        match value {
            Ok(Some(value)) => cursor.field.set(&mut self.window, value),
            Ok(None) => {}
            Err(error) => {
                let origin =
                    ErrorOrigin::WindowField(cursor.field, cursor.editor.tokens().to_vec());
                return Err(error_view(error, origin));
            }
        }
        Ok(())
    }

    pub(super) fn perform_zoom(&mut self, menu: ListMenu, action: Action) -> View {
        let pick = match action {
            Action::Command(Command::Up) => return View::Zoom(menu.step(false)),
            Action::Command(Command::Down) => return View::Zoom(menu.step(true)),
            Action::Edit(Edit::Clear) => return HOME,
            Action::Command(Command::Enter) => Some(ZoomItem::ALL[menu.highlighted()]),
            Action::Insert(Token::Digit(digit)) => ZoomItem::numbered(digit as u8),
            _ => None,
        };
        match pick {
            Some(item) => self.zoom(item),
            None => View::Zoom(menu),
        }
    }

    /// A typing menu: arrows move through tabs and items; ENTER or an
    /// item's key types its token into the screen under the menu and
    /// returns there; CLEAR returns without typing.
    pub(super) fn perform_menu(&mut self, menu: OpenMenu, action: Action) -> View {
        let OpenMenu { cursor, under } = menu;
        let moved = |cursor: MenuCursor, under: Box<View>| View::Menu(OpenMenu { cursor, under });
        let token = match action {
            Action::Edit(Edit::Clear) => return *under,
            Action::Edit(Edit::Left) => return moved(cursor.step_tab(false), under),
            Action::Edit(Edit::Right) => return moved(cursor.step_tab(true), under),
            Action::Command(Command::Up) => return moved(cursor.step_item(false), under),
            Action::Command(Command::Down) => return moved(cursor.step_item(true), under),
            Action::Command(Command::Enter) => Some(cursor.highlighted_token()),
            Action::Insert(typed) => menu_key(typed).and_then(|key| cursor.token_for_key(key)),
            _ => None,
        };
        match token {
            Some(token) => self.perform_in(*under, Action::Insert(token)),
            None => moved(cursor, under),
        }
    }
}
