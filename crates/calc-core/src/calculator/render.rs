use super::{Calculator, View};
use crate::{
    error_screen::{ErrorChoice, ErrorMenu},
    field_editor::{Field, FieldCursor},
    format::format_real,
    functions::{FUNCTION_COUNT, Functions},
    keymap::Layer,
    list_menu::ListMenu,
    mode_screen::{ModeCursor, ModeRow},
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
            view: match &self.view {
                View::Home(_) => ScreenView::Home(HomeView {
                    history: self.history.lines(),
                    entry: self.editor.tokens(),
                    cursor: self.editor.cursor(),
                }),
                View::Mode(cursor) => ScreenView::Mode(self.mode_view(*cursor)),
                View::Error(menu) => ScreenView::Error(error_view(menu)),
                View::Functions(cursor) => ScreenView::Functions(self.functions_view(*cursor)),
                View::Window(cursor) => {
                    ScreenView::Fields(self.fields_view(cursor, |field| field.get(&self.window)))
                }
                View::TableSetup(cursor) => {
                    ScreenView::Fields(self.fields_view(cursor, |field| field.get(&self.table)))
                }
                View::Table(cursor) => ScreenView::Table(self.table_view(*cursor)),
                View::Graph(trace) => ScreenView::Graph(Box::new(self.graph_view(*trace))),
                View::Zoom(menu) => ScreenView::Menu(zoom_menu(*menu)),
            },
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

    /// A settings screen: each field's value from `value`, the focused
    /// one's entry instead.
    fn fields_view<'a, F: Field>(
        &self,
        cursor: &'a FieldCursor<F>,
        value: impl Fn(F) -> f64,
    ) -> FieldsView<'a> {
        let rows = F::ALL.iter().map(|&field| FieldRowView {
            label: field.label(),
            value: format_real(value(field), self.settings),
            editing: (field == cursor.field)
                .then(|| (cursor.editor.tokens(), cursor.editor.cursor())),
        });
        FieldsView {
            title: F::TITLE,
            rows: rows.collect(),
        }
    }

    fn graph_view(&self, trace: Option<TraceCursor>) -> GraphView {
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
            plot: plot(&functions, &self.window, context),
            x_ticks: self.window.x_ticks(),
            y_ticks: self.window.y_ticks(),
            trace,
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
    MenuView {
        title: "ZOOM",
        items: ZoomItem::ALL
            .iter()
            .map(|item| MenuItemView {
                digit: item.digit(),
                label: item.label(),
            })
            .collect(),
        highlighted: menu.highlighted(),
    }
}
