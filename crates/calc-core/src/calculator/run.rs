use super::{Calculator, HOME, View, error_view};
use crate::{
    error::{CalcError, ErrorKind},
    error_screen::ErrorOrigin,
    expr::Expr,
    format::format_value,
    home::Output,
    keymap::{Action, Command, Edit},
    parse::parse,
    program::{Counter, Op, Step, compile},
    token::{Letter, Token},
};

/// Steps run between screen updates, so a long program stays interruptible.
const STEPS_PER_SLICE: usize = 2000;

/// A program running: the programs called and where each is, and what it
/// waits for, if anything.
#[derive(Debug, Clone)]
pub(super) struct Run {
    frames: Vec<Frame>,
    pub(super) waiting: Option<Wait>,
}

#[derive(Debug, Clone)]
struct Frame {
    name: String,
    steps: Vec<Step>,
    /// The next step to run.
    next: usize,
}

/// What a running program waits for: a value typed for a variable, or
/// ENTER after Pause.
#[derive(Debug, Clone)]
pub(super) enum Wait {
    Input { prompt: String, letter: Letter },
    Pause,
}

/// Where a step sends the program.
enum Flow {
    Next,
    Jump(usize),
    Wait(Wait),
    Call(String),
    Stop,
}

impl Run {
    /// The program and line of the step that ran last.
    fn place(&self) -> Option<(String, usize)> {
        let frame = self.frames.last()?;
        let step = frame.steps.get(frame.next.saturating_sub(1))?;
        Some((frame.name.clone(), step.line))
    }
}

impl Calculator {
    /// ENTER on `prgmNAME`: the entry goes into the history and the program
    /// starts.
    pub(super) fn start_program(&mut self, entry: &[Token], name: &[Token]) -> View {
        let name: String = name.iter().map(|token| token.text()).collect();
        self.history.push(Output {
            entry: entry.to_vec(),
            latex: format!("\\mathrm{{prgm{name}}}"),
            result: String::new(),
            lines: Vec::new(),
        });
        let mut run = Run {
            frames: Vec::new(),
            waiting: None,
        };
        match self.call(&mut run, name) {
            Ok(()) => self.resume_run(run),
            Err(error) => error,
        }
    }

    /// Pushes the program named `name` onto the run; the error screen when
    /// there is none or it does not compile.
    fn call(&self, run: &mut Run, name: String) -> Result<(), View> {
        let Some(program) = self.program(&name) else {
            return Err(error_view(
                CalcError::new(ErrorKind::Undefined),
                ErrorOrigin::Home(Vec::new()),
            ));
        };
        let steps = compile(&program.lines)
            .map_err(|(error, line)| error_view(error, ErrorOrigin::Program(name.clone(), line)))?;
        run.frames.push(Frame {
            name,
            steps,
            next: 0,
        });
        Ok(())
    }

    /// Runs up to a slice of steps: on to the next wait, the end, an error,
    /// or the end of the slice, after which the program carries on when
    /// resumed.
    pub(super) fn resume_run(&mut self, mut run: Run) -> View {
        for _ in 0..STEPS_PER_SLICE {
            let Some(frame) = run.frames.last_mut() else {
                return self.finish_run();
            };
            let Some(step) = frame.steps.get(frame.next) else {
                run.frames.pop();
                continue;
            };
            frame.next += 1;
            let flow = self.execute(&step.op);
            if let Some(view) = self.follow(&mut run, flow) {
                return view;
            }
        }
        View::Running(run)
    }

    /// Applies a step's flow to the run; the view to show when the run
    /// stops here.
    fn follow(&mut self, run: &mut Run, flow: Result<Flow, CalcError>) -> Option<View> {
        match flow {
            Ok(Flow::Next) => None,
            Ok(Flow::Jump(to)) => {
                run.frames.last_mut()?.next = to;
                None
            }
            Ok(Flow::Wait(wait)) => {
                run.waiting = Some(wait);
                Some(View::Running(std::mem::replace(run, empty_run())))
            }
            Ok(Flow::Call(name)) => self.call(run, name).err(),
            Ok(Flow::Stop) => Some(self.finish_run()),
            Err(error) => Some(program_error(run, error)),
        }
    }

    fn finish_run(&mut self) -> View {
        self.history.push(Output {
            entry: Vec::new(),
            latex: String::new(),
            result: "Done".to_owned(),
            lines: Vec::new(),
        });
        HOME
    }

    fn execute(&mut self, op: &Op) -> Result<Flow, CalcError> {
        match op {
            Op::Eval(statement) => self.evaluate_statement(statement).map(|_| Flow::Next),
            Op::Disp(items) => {
                for item in items {
                    self.display(item)?;
                }
                Ok(Flow::Next)
            }
            Op::Input { prompt, letter } => Ok(Flow::Wait(Wait::Input {
                prompt: prompt.clone(),
                letter: *letter,
            })),
            Op::Unless(condition, to) => Ok(match self.context().evaluate_real(condition)? {
                0.0 => Flow::Jump(*to),
                _ => Flow::Next,
            }),
            Op::Jump(to) => Ok(Flow::Jump(*to)),
            Op::ForStart {
                counter,
                start,
                exit,
            } => self.for_start(counter, (start, *exit)),
            Op::ForNext { counter, body } => self.for_next(counter, *body),
            Op::Pause => Ok(Flow::Wait(Wait::Pause)),
            Op::Stop => Ok(Flow::Stop),
            Op::ClrHome => {
                self.history.clear();
                Ok(Flow::Next)
            }
            Op::Call(name) => Ok(Flow::Call(name.clone())),
        }
    }

    /// Disp: text on the left, a value on the right, each on a line.
    fn display(&mut self, item: &Expr) -> Result<(), CalcError> {
        let (result, lines) = match item {
            Expr::Text(text) => (String::new(), vec![text.clone()]),
            expr => {
                let value = self.context().evaluate(expr)?;
                (format_value(&value, self.settings, None), Vec::new())
            }
        };
        self.history.push(Output {
            entry: Vec::new(),
            latex: String::new(),
            result,
            lines,
        });
        Ok(())
    }

    fn for_start(
        &mut self,
        counter: &Counter,
        (start, exit): (&Expr, usize),
    ) -> Result<Flow, CalcError> {
        let start = self.context().evaluate_real(start)?;
        self.memory.set(counter.letter, start);
        Ok(match self.counting(counter)? {
            true => Flow::Next,
            false => Flow::Jump(exit),
        })
    }

    fn for_next(&mut self, counter: &Counter, body: usize) -> Result<Flow, CalcError> {
        let value = self.memory.get(counter.letter) + self.step(counter)?;
        self.memory.set(counter.letter, value);
        Ok(match self.counting(counter)? {
            true => Flow::Jump(body),
            false => Flow::Next,
        })
    }

    fn step(&self, counter: &Counter) -> Result<f64, CalcError> {
        counter
            .step
            .as_ref()
            .map_or(Ok(1.0), |step| self.context().evaluate_real(step))
    }

    /// Whether the loop variable has not passed the end in its step's direction.
    fn counting(&self, counter: &Counter) -> Result<bool, CalcError> {
        let end = self.context().evaluate_real(&counter.end)?;
        let value = self.memory.get(counter.letter);
        Ok(match self.step(counter)? < 0.0 {
            true => value >= end,
            false => value <= end,
        })
    }

    /// Keys while a program runs: ON breaks it; while it waits for input,
    /// keys type the value and ENTER gives it; after Pause, ENTER carries on.
    pub(super) fn perform_running(&mut self, run: Run, action: Action) -> View {
        if action == Action::Command(Command::On) {
            return program_error(&run, CalcError::new(ErrorKind::Break));
        }
        match (&run.waiting, action) {
            (Some(Wait::Pause), Action::Command(Command::Enter)) => self.resume_run(Run {
                waiting: None,
                ..run
            }),
            (Some(Wait::Input { .. }), Action::Command(Command::Enter)) => self.give_input(run),
            (Some(Wait::Input { .. }), Action::Insert(token)) => {
                self.editor.type_token(token);
                View::Running(run)
            }
            (Some(Wait::Input { .. }), Action::Edit(edit)) => {
                self.editor.apply(edit);
                View::Running(run)
            }
            _ => View::Running(run),
        }
    }

    /// The typed value into the variable waited for, the prompt and value
    /// into the history, and the program carries on.
    fn give_input(&mut self, mut run: Run) -> View {
        let Some(Wait::Input { prompt, letter }) = run.waiting.take() else {
            return View::Running(run);
        };
        let tokens = self.editor.tokens().to_vec();
        let value =
            parse(&tokens).and_then(|statement| self.context().evaluate_real(&statement.expr));
        let value = match value {
            Ok(value) => value,
            Err(error) => return program_error(&run, error),
        };
        self.editor.apply(Edit::Clear);
        self.memory.set(letter, value);
        let typed: String = tokens.iter().map(|token| token.text()).collect();
        self.history.push(Output {
            entry: Vec::new(),
            latex: String::new(),
            result: String::new(),
            lines: vec![format!("{prompt}{typed}")],
        });
        self.resume_run(run)
    }

    /// The prompt a running program shows, while it waits for input.
    pub(super) fn input_prompt(run: &Run) -> Option<&str> {
        match &run.waiting {
            Some(Wait::Input { prompt, .. }) => Some(prompt),
            _ => None,
        }
    }

    /// Carries on a program between screen updates: the next slice of its
    /// steps, unless it waits for a key.
    pub fn resume(&mut self) {
        self.view = match std::mem::take(&mut self.view) {
            View::Running(run) if run.waiting.is_none() => self.resume_run(run),
            view => view,
        };
    }
}

fn empty_run() -> Run {
    Run {
        frames: Vec::new(),
        waiting: None,
    }
}

/// The error screen for `error`, Goto opening the program at the line that ran.
fn program_error(run: &Run, error: CalcError) -> View {
    let origin = match run.place() {
        Some((name, line)) => ErrorOrigin::Program(name, line),
        None => ErrorOrigin::Home(Vec::new()),
    };
    error_view(CalcError::new(error.kind), origin)
}
