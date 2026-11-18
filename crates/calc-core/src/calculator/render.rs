use std::borrow::Cow;

use super::{Calculator, View, calc::calc_menu};
use crate::{
    choice_screen::{ChoiceCursor, ChoiceRow},
    error_screen::{ErrorChoice, ErrorMenu},
    field_editor::{Field, FieldCursor},
    format::format_real,
    functions::{FUNCTION_COUNT, Functions},
    home::HistorySelection,
    keymap::Layer,
    list_menu::ListMenu,
    menus::MenuCursor,
    modifiers::AlphaState,
    plot::plot,
    screen::{
        CursorStyle, ErrorView, FieldRowView, FieldsView, FunctionRowView, FunctionsView,
        GraphView, HomeView, MenuItemView, MenuView, ModeRowView, ModeView, RowFocus, Screen,
        ScreenView, TraceView,
    },
    trace::TraceCursor,
    y_editor::YCursor,
    zoom::ZoomItem,
};

impl Calculator {
    pub fn screen(&self) -> Screen<'_> {
        Screen {
            status: self.settings.labels(),
            cursor_style: self.cursor_style(),
            running: matches!(&self.view, View::Running(_)),
            busy: matches!(&self.view, View::Running(run) if run.waiting.is_none()),
            view: self.view(),
        }
    }

    fn view(&self) -> ScreenView<'_> {
        match &self.view {
            View::Home(state) => ScreenView::Home(self.home_view(state.selected, None)),
            View::Running(run) => ScreenView::Home(self.home_view(None, Self::input_prompt(run))),
            View::Mode(cursor) => ScreenView::Mode(choice_view(*cursor, &self.settings)),
            View::Format(cursor) => ScreenView::Mode(choice_view(*cursor, &self.format)),
            View::Error(menu) => ScreenView::Error(error_view(menu)),
            View::Functions(cursor) => ScreenView::Functions(self.functions_view(*cursor)),
            View::Window(cursor) => {
                ScreenView::Fields(self.fields_view(cursor, |field| field.get(&self.window)))
            }
            View::TableSetup(cursor) => {
                ScreenView::Fields(self.fields_view(cursor, |field| field.get(&self.table)))
            }
            View::Table(cursor) => ScreenView::Table(self.table_view(*cursor)),
            View::Grid(grid, editor) => ScreenView::Table(self.grid_view(*grid, editor)),
            View::Graph(trace) => ScreenView::Graph(Box::new(self.graph_view(*trace))),
            View::FreeGraph(cursor) => {
                ScreenView::Graph(Box::new(self.free_graph_view(*cursor, None)))
            }
            View::ZBox(marking) => ScreenView::Graph(Box::new(
                self.free_graph_view(marking.cursor, marking.corner),
            )),
            View::Zoom(menu) => ScreenView::Menu(zoom_menu(*menu)),
            View::CalcMenu(menu) => ScreenView::Menu(calc_menu(*menu)),
            View::GraphCalc(session) => ScreenView::Graph(Box::new(self.graph_calc_view(session))),
            View::Menu(menu) => ScreenView::Menu(typing_menu(menu.cursor)),
            View::Programs(menu) => ScreenView::Menu(self.program_menu_view(*menu)),
            View::ProgramName(editor) => ScreenView::Fields(Self::program_name_view(editor)),
            View::ProgramEditor(cursor) => ScreenView::Program(self.program_view(cursor)),
        }
    }

    fn home_view<'a>(
        &'a self,
        selected: Option<HistorySelection>,
        prompt: Option<&'a str>,
    ) -> HomeView<'a> {
        HomeView {
            history: self.history.lines(),
            entry: self.editor.tokens(),
            cursor: self.editor.cursor(),
            selected,
            prompt,
        }
    }

    fn functions_view(&self, cursor: YCursor) -> FunctionsView<'_> {
        let rows = self
            .functions
            .slots()
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                let focus = (index == cursor.row).then(|| match cursor.on_selector {
                    true => RowFocus::Selector,
                    false => RowFocus::Entry(slot.editor.cursor()),
                });
                FunctionRowView {
                    label: Functions::label(index),
                    entry: slot.editor.tokens(),
                    enabled: slot.enabled,
                    focus,
                }
            });
        FunctionsView {
            rows: rows.collect(),
        }
    }

    /// A settings screen: each field's value from `value` in Normal Float,
    /// whatever the mode, the focused one's entry instead.
    fn fields_view<'a, F: Field>(
        &self,
        cursor: &'a FieldCursor<F>,
        value: impl Fn(F) -> f64,
    ) -> FieldsView<'a> {
        let rows = F::ALL.iter().map(|&field| FieldRowView {
            label: field.label(),
            value: format_real(value(field), crate::settings::Settings::default()),
            editing: (field == cursor.field)
                .then(|| (cursor.editor.tokens(), cursor.editor.cursor())),
        });
        FieldsView {
            title: F::TITLE,
            rows: rows.collect(),
        }
    }

    pub(super) fn graph_view(&self, trace: Option<TraceCursor>) -> GraphView {
        let functions = self.functions.parse_graphed().unwrap_or_default();
        let context = self.context();
        let trace = trace.and_then(|cursor| {
            let (_, expr) = functions.iter().find(|(slot, _)| *slot == cursor.slot)?;
            let y = context.y_at(expr, cursor.x);
            Some(TraceView {
                slot: cursor.slot,
                equation: self.equation(cursor.slot),
                x: cursor.x,
                y,
                x_text: format!("X={}", format_real(cursor.x, self.settings)),
                y_text: format!(
                    "Y={}",
                    y.map(|y| format_real(y, self.settings)).unwrap_or_default()
                ),
            })
        });
        GraphView {
            window: self.window,
            plot: plot(&functions, &context),
            x_ticks: self.window.x_ticks(),
            y_ticks: self.window.y_ticks(),
            format: self.format,
            trace,
            free: None,
            calc: None,
        }
    }

    /// `Y1=X²`: a function's name and entry.
    fn equation(&self, slot: usize) -> String {
        let entry: String = self.functions.slots()[slot % FUNCTION_COUNT]
            .editor
            .tokens()
            .iter()
            .map(|t| t.text())
            .collect();
        format!("{}={entry}", Functions::label(slot))
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

fn error_view(menu: &ErrorMenu) -> ErrorView {
    ErrorView {
        message: menu.error.message(),
        choices: ErrorChoice::ALL,
        highlighted: menu.highlighted,
    }
}

fn zoom_menu(menu: ListMenu) -> MenuView {
    let items = ZoomItem::ALL.iter().map(|item| MenuItemView {
        key: char::from(b'0' + item.digit()),
        label: Cow::Borrowed(item.label()),
    });
    MenuView {
        tabs: vec!["ZOOM"],
        tab: 0,
        items: items.collect(),
        highlighted: menu.highlighted(),
    }
}

fn typing_menu(cursor: MenuCursor) -> MenuView {
    let entries = cursor.current_tab().entries.iter();
    let items = entries.map(|entry| MenuItemView {
        key: entry.key,
        label: Cow::Borrowed(entry.label),
    });
    MenuView {
        tabs: cursor.kind.tabs().iter().map(|tab| tab.title).collect(),
        tab: cursor.tab,
        items: items.collect(),
        highlighted: cursor.highlighted,
    }
}

/// An options screen's lines: each line's options and chosen one, and the
/// highlighted option on the cursor's line.
fn choice_view<R: ChoiceRow>(cursor: ChoiceCursor<R>, settings: &R::Settings) -> ModeView {
    let rows = R::ALL.iter().map(|&row| ModeRowView {
        options: row.options(),
        selected: row.selected(settings),
        highlighted: (row == cursor.row()).then_some(cursor.column()),
    });
    ModeView {
        rows: rows.collect(),
    }
}
