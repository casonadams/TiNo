//! The calculator: key presses in, a screen out. Each screen's keys are
//! handled in its own module; the screen keys (Y=, WINDOW, ZOOM, FORMAT,
//! TRACE, GRAPH, TBLSET, TABLE, MODE, 2nd QUIT) switch screens from anywhere
//! but the error screen.

mod calc;
mod free;
mod graph;
mod grids;
mod home;
mod persist;
mod render;
mod screens;
mod table;

pub use persist::LoadError;

use crate::{
    choice_screen::{ChoiceCursor, ChoiceRow},
    editor::Editor,
    error::CalcError,
    error_screen::{ErrorMenu, ErrorOrigin},
    eval::{Context, Memory},
    field_editor::FieldCursor,
    free_cursor::FreeCursor,
    functions::Functions,
    graph_calc::CalcItem,
    graph_format::{FormatCursor, GraphFormat},
    grid_editor::{Grid, GridEditor},
    home::{EntryStack, History, HistorySelection},
    key::Key,
    keymap::{self, Action, Command, Edit},
    list_menu::ListMenu,
    menus::{MenuCursor, MenuKind},
    mode_screen::ModeCursor,
    modifiers::Modifiers,
    settings::{NumberMode, Settings},
    table::{TableCursor, TableField, TableSettings},
    trace::TraceCursor,
    window::Window,
    window_editor::WindowField,
    y_editor::YCursor,
    zoom::ZoomItem,
};

/// The screen being shown, with its own state.
#[derive(Debug, Clone)]
enum View {
    Home(HomeState),
    Mode(ModeCursor),
    Error(ErrorMenu),
    Functions(YCursor),
    Window(FieldCursor<WindowField>),
    /// The graph, with the trace cursor while tracing.
    Graph(Option<TraceCursor>),
    /// The graph with the free-moving cursor out.
    FreeGraph(FreeCursor),
    /// ZOOM ZBox: corners being marked on the graph.
    ZBox(free::BoxMarking),
    /// 2nd FORMAT: how the graph is drawn.
    Format(FormatCursor),
    /// 2nd CALC: the items that compute on the graph.
    CalcMenu(ListMenu),
    /// A CALC item under way on the graph.
    GraphCalc(calc::CalcSession),
    Zoom(ListMenu),
    TableSetup(FieldCursor<TableField>),
    Table(TableCursor),
    /// STAT EDIT's lists or MATRIX EDIT's matrix, a grid of cells.
    Grid(Grid, GridEditor),
    /// A menu that types into the screen under it.
    Menu(OpenMenu),
}

/// An open typing menu and the screen it types into.
#[derive(Debug, Clone)]
struct OpenMenu {
    cursor: MenuCursor,
    under: Box<View>,
}

/// The home screen's state between key presses.
#[derive(Debug, Clone, Copy, Default)]
struct HomeState {
    /// How far back 2nd ENTRY last reached while it keeps being pressed.
    recalled: Option<usize>,
    /// The history entry or result ▲ ▼ highlight.
    selected: Option<HistorySelection>,
}

const HOME: View = View::Home(HomeState {
    recalled: None,
    selected: None,
});

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
    Format,
    Calc,
    Graph,
    Trace,
    TableSetup,
    Table,
}

impl Destination {
    fn of(command: Command) -> Option<Self> {
        Some(match command {
            Command::Quit => Destination::Home,
            Command::Mode => Destination::Mode,
            Command::YEquals => Destination::Functions,
            Command::Window => Destination::Window,
            Command::Zoom => Destination::Zoom,
            Command::Format => Destination::Format,
            Command::Calc => Destination::Calc,
            Command::Graph => Destination::Graph,
            Command::Trace => Destination::Trace,
            Command::TableSetup => Destination::TableSetup,
            Command::Table => Destination::Table,
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
    table: TableSettings,
    format: GraphFormat,
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
        let (destination, menu) = match (&view, action) {
            (View::Error(_), _) => (None, None),
            (_, Action::Command(command)) => (Destination::of(command), menu_of(command)),
            _ => (None, None),
        };
        self.view = match (destination, menu) {
            (Some(destination), _) => self.leave(view).unwrap_or_else(|| self.open(destination)),
            (None, Some(kind)) => View::Menu(OpenMenu {
                cursor: MenuCursor::new(kind),
                under: Box::new(menu_base(view)),
            }),
            (None, None) => self.perform_in(view, action),
        };
    }

    fn perform_in(&mut self, view: View, action: Action) -> View {
        match view {
            View::Home(recalled) => self.perform_home(action, recalled),
            View::Mode(cursor) => {
                choice_step(cursor, action, &mut self.settings).map_or(HOME, View::Mode)
            }
            View::Format(cursor) => {
                choice_step(cursor, action, &mut self.format).map_or(HOME, View::Format)
            }
            View::Error(menu) => self.perform_error(menu, action),
            View::Functions(cursor) => View::Functions(cursor.perform(action, &mut self.functions)),
            View::Window(cursor) => self.perform_window(cursor, action),
            View::TableSetup(cursor) => self.perform_table_setup(cursor, action),
            View::Table(cursor) => self.perform_table(cursor, action),
            View::Grid(grid, editor) => self.perform_grid((grid, editor), action),
            View::Graph(trace) => self.perform_graph(trace, action),
            View::FreeGraph(cursor) => self.perform_free(cursor, action),
            View::ZBox(marking) => self.perform_zbox(marking, action),
            View::Zoom(menu) => self.perform_zoom(menu, action),
            View::CalcMenu(menu) => self.perform_calc_menu(menu, action),
            View::GraphCalc(session) => self.perform_graph_calc(session, action),
            View::Menu(menu) => self.perform_menu(menu, action),
        }
    }

    /// Leaving `view` for another screen; the error screen instead when the
    /// WINDOW or TBLSET field being typed does not evaluate.
    fn leave(&mut self, view: View) -> Option<View> {
        match view {
            View::Window(cursor) => self.commit_window(&cursor).err(),
            View::TableSetup(cursor) => self.commit_table_setup(&cursor).err(),
            View::Menu(menu) => self.leave(*menu.under),
            _ => None,
        }
    }

    fn open(&mut self, destination: Destination) -> View {
        match destination {
            Destination::Home => HOME,
            Destination::Mode => View::Mode(ModeCursor::new(&self.settings)),
            Destination::Format => View::Format(FormatCursor::new(&self.format)),
            Destination::Functions => {
                View::Functions(YCursor::at(0, usize::MAX, &mut self.functions))
            }
            Destination::Window => View::Window(self.window_cursor(WindowField::XMin)),
            Destination::Zoom => View::Zoom(ListMenu::new(ZoomItem::ALL.len())),
            Destination::Calc => View::CalcMenu(ListMenu::new(CalcItem::ALL.len())),
            Destination::Graph => self.graph(false),
            Destination::Trace => self.graph(true),
            Destination::TableSetup => View::TableSetup(self.table_setup_cursor(TableField::Start)),
            Destination::Table => self.table(),
        }
    }

    /// What expressions read: memory, angle units, whether nonreal answers
    /// are complex, WINDOW, TBLSET and Y=.
    fn context(&self) -> Context<'_> {
        Context {
            memory: &self.memory,
            angle: self.settings.angle,
            complex: self.settings.number != NumberMode::Real,
            window: &self.window,
            table: &self.table,
            functions: self.functions.parse_all(),
        }
    }
}

fn error_view(error: CalcError, origin: ErrorOrigin) -> View {
    View::Error(ErrorMenu::new(error, origin))
}

fn menu_of(command: Command) -> Option<MenuKind> {
    match command {
        Command::Math => Some(MenuKind::Math),
        Command::Test => Some(MenuKind::Test),
        Command::Vars => Some(MenuKind::Vars),
        Command::List => Some(MenuKind::List),
        Command::Stat => Some(MenuKind::Stat),
        Command::Matrix => Some(MenuKind::Matrix),
        _ => None,
    }
}

/// The screen a menu types into: the one shown when it has an entry
/// being typed, else the home screen.
fn menu_base(view: View) -> View {
    match view {
        View::Home(_) | View::Window(_) | View::TableSetup(_) | View::Grid(..) => view,
        View::Functions(cursor) if !cursor.on_selector => view,
        View::Menu(menu) => *menu.under,
        _ => HOME,
    }
}

/// An options screen's keys: ▲ ▼ change line, ◀ ▶ move along it, ENTER
/// chooses; `None` for CLEAR, which leaves it.
fn choice_step<R: ChoiceRow>(
    cursor: ChoiceCursor<R>,
    action: Action,
    settings: &mut R::Settings,
) -> Option<ChoiceCursor<R>> {
    Some(match action {
        Action::Edit(Edit::Clear) => return None,
        Action::Command(Command::Up) => cursor.step_row(false, settings),
        Action::Command(Command::Down) => cursor.step_row(true, settings),
        Action::Edit(Edit::Left) => cursor.step_column(false),
        Action::Edit(Edit::Right) => cursor.step_column(true),
        Action::Command(Command::Enter) => {
            cursor.choose(settings);
            cursor
        }
        _ => cursor,
    })
}
