//! The calculator: key presses in, a screen out. Each screen's keys are
//! handled in its own module; the screen keys (Y=, WINDOW, ZOOM, TRACE,
//! GRAPH, MODE, 2nd QUIT) switch screens from anywhere but the error screen.

mod graph;
mod home;
mod render;
mod screens;

use crate::{
    editor::Editor,
    error::CalcError,
    error_screen::{ErrorMenu, ErrorOrigin},
    eval::Memory,
    functions::Functions,
    home::{EntryStack, History},
    key::Key,
    keymap::{self, Action, Command},
    list_menu::ListMenu,
    mode_screen::ModeCursor,
    modifiers::Modifiers,
    settings::Settings,
    trace::TraceCursor,
    window::Window,
    window_editor::{WindowCursor, WindowField},
    y_editor::YCursor,
    zoom::ZoomItem,
};

/// The screen being shown, with its own state.
#[derive(Debug, Clone)]
enum View {
    /// The home screen, with how far back 2nd ENTRY last reached while it
    /// keeps being pressed.
    Home(Option<usize>),
    Mode(ModeCursor),
    Error(ErrorMenu),
    Functions(YCursor),
    Window(WindowCursor),
    /// The graph, with the trace cursor while tracing.
    Graph(Option<TraceCursor>),
    Zoom(ListMenu),
}

const HOME: View = View::Home(None);

impl Default for View {
    fn default() -> Self {
        HOME
    }
}

/// A screen the screen keys open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Destination {
    Home,
    Mode,
    Functions,
    Window,
    Zoom,
    Graph,
    Trace,
}

impl Destination {
    fn of(command: Command) -> Option<Self> {
        Some(match command {
            Command::Quit => Destination::Home,
            Command::Mode => Destination::Mode,
            Command::YEquals => Destination::Functions,
            Command::Window => Destination::Window,
            Command::Zoom => Destination::Zoom,
            Command::Graph => Destination::Graph,
            Command::Trace => Destination::Trace,
            _ => return None,
        })
    }
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
    functions: Functions,
    window: Window,
}

impl Calculator {
    pub fn press(&mut self, key: Key) {
        match key {
            Key::Second => self.modifiers.press_second(),
            Key::Alpha => self.modifiers.press_alpha(),
            _ => {
                let action = keymap::action(key, self.modifiers.layer());
                self.modifiers.release();
                if let Some(action) = action {
                    self.perform(action);
                }
            }
        }
    }

    fn perform(&mut self, action: Action) {
        let view = std::mem::take(&mut self.view);
        let destination = match (&view, action) {
            (View::Error(_), _) => None,
            (_, Action::Command(command)) => Destination::of(command),
            _ => None,
        };
        self.view = match destination {
            Some(destination) => self.leave(view).unwrap_or_else(|| self.open(destination)),
            None => self.perform_in(view, action),
        };
    }

    fn perform_in(&mut self, view: View, action: Action) -> View {
        match view {
            View::Home(recalled) => self.perform_home(action, recalled),
            View::Mode(cursor) => self.perform_mode(cursor, action),
            View::Error(menu) => self.perform_error(menu, action),
            View::Functions(cursor) => View::Functions(cursor.perform(action, &mut self.functions)),
            View::Window(cursor) => self.perform_window(cursor, action),
            View::Graph(trace) => self.perform_graph(trace, action),
            View::Zoom(menu) => self.perform_zoom(menu, action),
        }
    }

    /// Leaving `view` for another screen; the error screen instead when the
    /// WINDOW field being typed does not evaluate.
    fn leave(&mut self, view: View) -> Option<View> {
        match view {
            View::Window(cursor) => self.commit_window(&cursor).err(),
            _ => None,
        }
    }

    fn open(&mut self, destination: Destination) -> View {
        match destination {
            Destination::Home => HOME,
            Destination::Mode => View::Mode(ModeCursor::new(self.settings)),
            Destination::Functions => {
                View::Functions(YCursor::at(0, usize::MAX, &mut self.functions))
            }
            Destination::Window => View::Window(WindowCursor::new(
                WindowField::XMin,
                &self.window,
                self.settings,
            )),
            Destination::Zoom => View::Zoom(ListMenu::new(ZoomItem::ALL.len())),
            Destination::Graph => self.graph(false),
            Destination::Trace => self.graph(true),
        }
    }
}

fn error_view(error: CalcError, origin: ErrorOrigin) -> View {
    View::Error(ErrorMenu::new(error, origin))
}
