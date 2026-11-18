use super::{Calculator, HOME, View, error_view};
use crate::{
    editor::Editor,
    error::{CalcError, ErrorKind},
    error_screen::ErrorOrigin,
    format::format_real,
    graph_calc::{CalcItem, CalcResult, compute},
    keymap::{Action, Command, Edit},
    list_menu::ListMenu,
    parse::parse,
    screen::{CalcView, GraphView, MenuItemView, MenuView},
    token::Token,
    trace::TraceCursor,
};

/// A CALC item under way on the graph, the trace cursor marking its points.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct CalcSession {
    pub(super) item: CalcItem,
    pub(super) cursor: TraceCursor,
    pub(super) stage: CalcStage,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum CalcStage {
    /// value: the X being typed.
    EnterX(Editor),
    /// The points marked so far, one per prompt answered.
    Marking(Vec<TraceCursor>),
    /// Shown until the next key, which then traces from the result.
    Done(CalcResult),
}

impl Calculator {
    /// The CALC menu: ▲ ▼ and ENTER or an item's digit start an item on the
    /// graph; CLEAR returns home.
    pub(super) fn perform_calc_menu(&mut self, menu: ListMenu, action: Action) -> View {
        let pick = match action {
            Action::Command(Command::Up) => return View::CalcMenu(menu.step(false)),
            Action::Command(Command::Down) => return View::CalcMenu(menu.step(true)),
            Action::Edit(Edit::Clear) => return HOME,
            Action::Command(Command::Enter) => Some(CalcItem::ALL[menu.highlighted()]),
            Action::Insert(token) => digit_item(token),
            _ => None,
        };
        match pick {
            Some(item) => self.start_calc(item),
            None => View::CalcMenu(menu),
        }
    }

    /// The graph with `item` asking for its first point (or X); the graph
    /// alone when nothing is graphed, the error screen when it cannot draw.
    fn start_calc(&mut self, item: CalcItem) -> View {
        let cursor = match self.graph(true) {
            View::Graph(Some(cursor)) => cursor,
            other => return other,
        };
        let stage = match item {
            CalcItem::Value => CalcStage::EnterX(Editor::default()),
            _ => CalcStage::Marking(Vec::new()),
        };
        View::GraphCalc(CalcSession {
            item,
            cursor,
            stage,
        })
    }

    pub(super) fn perform_graph_calc(&mut self, session: CalcSession, action: Action) -> View {
        let CalcSession {
            item,
            cursor,
            stage,
        } = session;
        match stage {
            CalcStage::Done(result) => self.perform_graph(Some(result.cursor), action),
            CalcStage::EnterX(editor) => self.enter_x((cursor, editor), action),
            CalcStage::Marking(marks) => self.mark((item, cursor, marks), action),
        }
    }

    /// value: typing keys edit X; ENTER moves the cursor there; CLEAR on an
    /// empty entry leaves for the graph.
    fn enter_x(&mut self, (cursor, mut editor): (TraceCursor, Editor), action: Action) -> View {
        let session = |editor| CalcSession {
            item: CalcItem::Value,
            cursor,
            stage: CalcStage::EnterX(editor),
        };
        match action {
            Action::Edit(Edit::Clear) if editor.tokens().is_empty() => View::Graph(Some(cursor)),
            Action::Command(Command::Enter) => match self.x_value(editor.tokens()) {
                Ok(x) => done(CalcItem::Value, plain(TraceCursor { x, ..cursor })),
                Err(error) => error_view(error, ErrorOrigin::Graph),
            },
            Action::Insert(token) => {
                editor.type_token(token);
                View::GraphCalc(session(editor))
            }
            Action::Edit(edit) => {
                editor.apply(edit);
                View::GraphCalc(session(editor))
            }
            Action::Command(_) => View::GraphCalc(session(editor)),
        }
    }

    /// The X typed for value, which must lie in the window.
    fn x_value(&self, entry: &[Token]) -> Result<f64, CalcError> {
        let statement = parse(entry)?;
        let x = self.context().evaluate_real(&statement.expr)?;
        if (self.window.x_min..=self.window.x_max).contains(&x) {
            Ok(x)
        } else {
            Err(CalcError::new(ErrorKind::Invalid))
        }
    }

    /// Answering a prompt: ◀ ▶ move the cursor, ▲ ▼ change function, ENTER
    /// marks the point (after the first curve of intersect, the cursor moves
    /// to the next function); the last mark computes the item.
    fn mark(
        &mut self,
        (item, cursor, mut marks): (CalcItem, TraceCursor, Vec<TraceCursor>),
        action: Action,
    ) -> View {
        let cursor = match action {
            Action::Edit(Edit::Clear) => return View::Graph(Some(cursor)),
            Action::Edit(Edit::Left) => self.step_trace(cursor, -1.0),
            Action::Edit(Edit::Right) => self.step_trace(cursor, 1.0),
            Action::Command(Command::Up) => cursor.switched(&self.graphed(), false),
            Action::Command(Command::Down) => cursor.switched(&self.graphed(), true),
            Action::Command(Command::Enter) => {
                marks.push(cursor);
                if marks.len() == item.prompts().len() {
                    return self.finish(item, &marks);
                }
                match item {
                    CalcItem::Intersect => cursor.switched(&self.graphed(), true),
                    _ => cursor,
                }
            }
            _ => cursor,
        };
        View::GraphCalc(CalcSession {
            item,
            cursor,
            stage: CalcStage::Marking(marks),
        })
    }

    fn finish(&mut self, item: CalcItem, marks: &[TraceCursor]) -> View {
        let context = self.context();
        let y = |slot: usize, x: f64| context.function_at(slot, x);
        match compute(item, marks, (&y, &self.window)) {
            Ok(result) => done(item, result),
            Err(error) => error_view(error, ErrorOrigin::Graph),
        }
    }
}

fn done(item: CalcItem, result: CalcResult) -> View {
    View::GraphCalc(CalcSession {
        item,
        cursor: result.cursor,
        stage: CalcStage::Done(result),
    })
}

fn plain(cursor: TraceCursor) -> CalcResult {
    CalcResult {
        cursor,
        title: None,
        value: None,
        shade: None,
        exact_y: None,
    }
}

fn digit_item(token: Token) -> Option<CalcItem> {
    let Token::Digit(digit) = token else {
        return None;
    };
    CalcItem::ALL
        .into_iter()
        .find(|item| item.digit() == digit as u8)
}

impl Calculator {
    /// The graph traced at the session's cursor, with what the item shows;
    /// a result's exact y replaces the traced one.
    pub(super) fn graph_calc_view(&self, session: &CalcSession) -> GraphView {
        let mut view = self.graph_view(Some(session.cursor));
        let exact_y = match &session.stage {
            CalcStage::Done(result) => result.exact_y,
            _ => None,
        };
        if let (Some(y), Some(trace)) = (exact_y, view.trace.as_mut()) {
            trace.y = Some(y);
            trace.y_text = format!("Y={}", format_real(y, self.settings));
        }
        GraphView {
            calc: Some(self.calc_view(session)),
            ..view
        }
    }

    fn calc_view(&self, session: &CalcSession) -> CalcView {
        let empty = CalcView {
            heading: None,
            entry: None,
            value: None,
            shade: None,
        };
        match &session.stage {
            CalcStage::EnterX(editor) => CalcView {
                entry: Some((editor.tokens().to_vec(), editor.cursor())),
                ..empty
            },
            CalcStage::Marking(marks) => CalcView {
                heading: session
                    .item
                    .prompts()
                    .get(marks.len())
                    .copied()
                    .filter(|p| !p.is_empty()),
                ..empty
            },
            CalcStage::Done(result) => CalcView {
                heading: result.title,
                value: result
                    .value
                    .map(|(label, value)| format!("{label}={}", format_real(value, self.settings))),
                shade: result.shade,
                ..empty
            },
        }
    }
}

/// The CALC menu: its items by digit, the highlighted one filled.
pub(super) fn calc_menu(menu: ListMenu) -> MenuView {
    let items = CalcItem::ALL.iter().map(|item| MenuItemView {
        key: char::from(b'0' + item.digit()),
        label: item.label(),
    });
    MenuView {
        tabs: vec!["CALCULATE"],
        tab: 0,
        items: items.collect(),
        highlighted: menu.highlighted(),
    }
}
