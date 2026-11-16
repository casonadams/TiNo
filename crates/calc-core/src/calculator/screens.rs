use super::{Calculator, HOME, View, error_view};
use crate::{
    error_screen::{ErrorChoice, ErrorMenu, ErrorOrigin},
    keymap::{Action, Command, Edit},
    list_menu::ListMenu,
    mode_screen::ModeCursor,
    token::{Digit, Token},
    window_editor::{WindowCursor, WindowField},
    y_editor::YCursor,
    zoom::ZoomItem,
};

impl Calculator {
    pub(super) fn perform_mode(&mut self, cursor: ModeCursor, action: Action) -> View {
        let cursor = match action {
            Action::Edit(Edit::Clear) => return HOME,
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
                let mut cursor = WindowCursor::new(field, &self.window, self.settings);
                cursor.editor.replace(entry);
                cursor.editor.place_cursor(position);
                View::Window(cursor)
            }
            ErrorOrigin::Window => View::Window(WindowCursor::new(
                WindowField::XMin,
                &self.window,
                self.settings,
            )),
        }
    }

    pub(super) fn perform_window(&mut self, mut cursor: WindowCursor, action: Action) -> View {
        let down = match action {
            Action::Insert(token) => {
                cursor.editor.type_token(token);
                return View::Window(cursor);
            }
            Action::Edit(edit) => {
                cursor.editor.apply(edit);
                return View::Window(cursor);
            }
            Action::Command(Command::Up) => false,
            Action::Command(Command::Down | Command::Enter) => true,
            Action::Command(_) => return View::Window(cursor),
        };
        if let Err(error) = self.commit_window(&cursor) {
            return error;
        }
        View::Window(WindowCursor::new(
            cursor.neighbor(down),
            &self.window,
            self.settings,
        ))
    }

    /// Writes the WINDOW field's entry into the window; the error screen
    /// when it does not evaluate.
    pub(super) fn commit_window(&mut self, cursor: &WindowCursor) -> Result<(), View> {
        match cursor.value(&self.memory, self.settings.angle) {
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
}
