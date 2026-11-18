//! Programs: named lists of lines, and their compilation into steps with
//! the jumps of If, While, Repeat and For resolved.

use serde::{Deserialize, Serialize};

use crate::{
    error::{CalcError, ErrorKind},
    expr::{Expr, Statement},
    parse::{expressions, parse},
    token::{Keyword, Letter, Token},
};

type Result<T, E = CalcError> = std::result::Result<T, E>;

/// Characters a program's name has at most.
pub const LONGEST_NAME: usize = 8;

/// A program: its name and its lines of tokens, each line one or more
/// instructions separated by `:`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub name: String,
    pub lines: Vec<Vec<Token>>,
}

/// The name `tokens` spell, if they make one: a letter, then letters or
/// digits, at most eight.
pub fn name(tokens: &[Token]) -> Option<String> {
    let first_is_letter = matches!(tokens.first(), Some(Token::Letter(_)));
    let valid = tokens
        .iter()
        .all(|token| matches!(token, Token::Letter(_) | Token::Digit(_)));
    (first_is_letter && valid && tokens.len() <= LONGEST_NAME)
        .then(|| tokens.iter().map(|token| token.text()).collect())
}

/// What a step does when run.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Evaluates, stores and sets `Ans`, as on the home screen.
    Eval(Statement),
    /// Shows each item on a line of its own: text left, values right.
    Disp(Vec<Expr>),
    /// Asks for a value for the variable, showing the prompt.
    Input {
        prompt: String,
        letter: Letter,
    },
    /// Goes to the step when the condition is 0.
    Unless(Expr, usize),
    Jump(usize),
    /// Sets the variable to its start and enters the loop unless it is
    /// already past the end; else goes to the exit step.
    ForStart {
        counter: Counter,
        start: Expr,
        exit: usize,
    },
    /// Steps the variable and goes back to the body unless it passed the end.
    ForNext {
        counter: Counter,
        body: usize,
    },
    Pause,
    Stop,
    ClrHome,
    /// Runs the program of this name, then carries on.
    Call(String),
}

/// A For( loop's variable, end and step (1 when left out).
#[derive(Debug, Clone, PartialEq)]
pub struct Counter {
    pub letter: Letter,
    pub end: Expr,
    pub step: Option<Expr>,
}

/// A step of a compiled program: what it does and the line it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub op: Op,
    pub line: usize,
}

/// A program's steps, or the first error and the line it is on.
pub fn compile(lines: &[Vec<Token>]) -> Result<Vec<Step>, (CalcError, usize)> {
    let mut compiler = Compiler::default();
    for (line, tokens) in lines.iter().enumerate() {
        for tokens in instructions(tokens) {
            compiler.line = line;
            let instruction = instruction(tokens).map_err(|error| (error, line))?;
            compiler.add(instruction).map_err(|error| (error, line))?;
        }
    }
    match compiler.blocks.is_empty() {
        true => Ok(compiler.steps),
        false => Err((
            CalcError::new(ErrorKind::Syntax),
            lines.len().saturating_sub(1),
        )),
    }
}

/// A line's instructions: its tokens split at each `:` outside quotes.
fn instructions(tokens: &[Token]) -> impl Iterator<Item = &[Token]> {
    let mut quoted = false;
    tokens.split(move |&token| {
        quoted ^= token == Token::Quote;
        token == Token::Colon && !quoted
    })
}

/// One instruction, parsed.
enum Instruction {
    Nothing,
    If(Expr),
    Then,
    Else,
    End,
    While(Expr),
    Repeat(Expr),
    For(Expr, Counter),
    Op(Op),
    /// Several steps from one instruction (`Prompt A,B`).
    Ops(Vec<Op>),
}

fn instruction(tokens: &[Token]) -> Result<Instruction> {
    match tokens.split_first() {
        None => Ok(Instruction::Nothing),
        Some((Token::Keyword(keyword), rest)) => keyword_instruction(*keyword, rest),
        Some((Token::Prgm, rest)) => match name(rest) {
            Some(name) => Ok(Instruction::Op(Op::Call(name))),
            None => syntax(),
        },
        Some(_) => parse(tokens).map(|statement| Instruction::Op(Op::Eval(statement))),
    }
}

fn syntax<T>() -> Result<T> {
    Err(CalcError::new(ErrorKind::Syntax))
}

fn keyword_instruction(keyword: Keyword, rest: &[Token]) -> Result<Instruction> {
    let bare = |instruction: Instruction| match rest {
        [] => Ok(instruction),
        _ => syntax(),
    };
    match keyword {
        Keyword::If => condition(rest).map(Instruction::If),
        Keyword::While => condition(rest).map(Instruction::While),
        Keyword::Repeat => condition(rest).map(Instruction::Repeat),
        Keyword::For => for_loop(rest),
        Keyword::Then => bare(Instruction::Then),
        Keyword::Else => bare(Instruction::Else),
        Keyword::End => bare(Instruction::End),
        Keyword::Pause => bare(Instruction::Op(Op::Pause)),
        Keyword::Stop => bare(Instruction::Op(Op::Stop)),
        Keyword::ClrHome => bare(Instruction::Op(Op::ClrHome)),
        Keyword::Disp => expressions(rest).map(|items| Instruction::Op(Op::Disp(items))),
        Keyword::Input => input(rest),
        Keyword::Prompt => prompt(rest),
    }
}

fn condition(tokens: &[Token]) -> Result<Expr> {
    let statement = parse(tokens)?;
    match (statement.display, statement.store) {
        (None, None) => Ok(statement.expr),
        _ => syntax(),
    }
}

/// `For(variable, start, end[, step])`.
fn for_loop(tokens: &[Token]) -> Result<Instruction> {
    let mut arguments = expressions(tokens)?.into_iter();
    let (Some(Expr::Variable(letter)), Some(start), Some(end)) =
        (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err(CalcError::new(ErrorKind::Argument));
    };
    let step = arguments.next();
    if arguments.next().is_some() {
        return Err(CalcError::new(ErrorKind::Argument));
    }
    Ok(Instruction::For(start, Counter { letter, end, step }))
}

/// `Input variable` (prompting `?`) or `Input "text",variable`.
fn input(tokens: &[Token]) -> Result<Instruction> {
    let input = |prompt: String, letter: Letter| Instruction::Op(Op::Input { prompt, letter });
    match expressions(tokens)?.as_slice() {
        [Expr::Variable(letter)] => Ok(input("?".to_owned(), *letter)),
        [Expr::Text(prompt), Expr::Variable(letter)] => Ok(input(prompt.clone(), *letter)),
        _ => syntax(),
    }
}

/// `Prompt A,B`: one input per variable, prompting `A=?`.
fn prompt(tokens: &[Token]) -> Result<Instruction> {
    let input = |expr: Expr| match expr {
        Expr::Variable(letter) => Ok(Op::Input {
            prompt: format!("{}=?", letter.symbol()),
            letter,
        }),
        _ => syntax(),
    };
    let inputs = expressions(tokens)?.into_iter().map(input);
    inputs.collect::<Result<_>>().map(Instruction::Ops)
}

/// A block open while its instructions compile, with the step whose jump
/// its end fills in.
enum Block {
    If { jump: usize },
    Else { jump: usize },
    While { start: usize },
    Repeat { start: usize, until: Expr },
    For { start: usize, counter: Counter },
}

#[derive(Default)]
struct Compiler {
    steps: Vec<Step>,
    blocks: Vec<Block>,
    line: usize,
    /// One-line Ifs' jumps, filled in after the instruction they guard.
    guards: Vec<usize>,
    /// Whether the last instruction was an If whose Then may follow.
    after_if: bool,
}

impl Compiler {
    fn add(&mut self, instruction: Instruction) -> Result<()> {
        let after_if = std::mem::take(&mut self.after_if);
        match instruction {
            Instruction::Nothing => self.after_if = after_if,
            Instruction::If(condition) => {
                let jump = self.emit(Op::Unless(condition, 0));
                self.guards.push(jump);
                self.after_if = true;
            }
            Instruction::Then if after_if => self.then(),
            other => {
                self.block_or_op(other)?;
                for jump in std::mem::take(&mut self.guards) {
                    self.patch(jump, self.steps.len());
                }
            }
        }
        Ok(())
    }

    /// Then after an If: the If's jump goes to its Else or End instead of
    /// past the next instruction.
    fn then(&mut self) {
        if let Some(jump) = self.guards.pop() {
            self.blocks.push(Block::If { jump });
        }
    }

    fn block_or_op(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::While(condition) => {
                let start = self.emit(Op::Unless(condition, 0));
                self.blocks.push(Block::While { start });
            }
            Instruction::Repeat(until) => {
                let start = self.steps.len();
                self.blocks.push(Block::Repeat { start, until });
            }
            Instruction::For(start, counter) => {
                let exit = 0;
                let begin = self.emit(Op::ForStart {
                    counter: counter.clone(),
                    start,
                    exit,
                });
                self.blocks.push(Block::For {
                    start: begin,
                    counter,
                });
            }
            Instruction::Else => self.otherwise()?,
            Instruction::End => self.end()?,
            Instruction::Op(op) => {
                self.emit(op);
            }
            Instruction::Ops(ops) => {
                for op in ops {
                    self.emit(op);
                }
            }
            _ => return syntax(),
        }
        Ok(())
    }

    /// Else: the If's jump comes here, and the Then part jumps to the End.
    fn otherwise(&mut self) -> Result<()> {
        let Some(Block::If { jump }) = self.blocks.pop() else {
            return syntax();
        };
        let skip = self.emit(Op::Jump(0));
        self.patch(jump, self.steps.len());
        self.blocks.push(Block::Else { jump: skip });
        Ok(())
    }

    fn end(&mut self) -> Result<()> {
        match self.blocks.pop() {
            Some(Block::If { jump } | Block::Else { jump }) => self.patch(jump, self.steps.len()),
            Some(Block::While { start }) => {
                self.emit(Op::Jump(start));
                self.patch(start, self.steps.len());
            }
            Some(Block::Repeat { start, until }) => {
                self.emit(Op::Unless(until, start));
            }
            Some(Block::For { start, counter }) => {
                self.emit(Op::ForNext {
                    counter,
                    body: start + 1,
                });
                self.patch(start, self.steps.len());
            }
            None => return syntax(),
        }
        Ok(())
    }

    fn emit(&mut self, op: Op) -> usize {
        self.steps.push(Step {
            op,
            line: self.line,
        });
        self.steps.len() - 1
    }

    /// Points the jump of step `at` to step `target`.
    fn patch(&mut self, at: usize, target: usize) {
        match &mut self.steps[at].op {
            Op::Unless(_, to) | Op::Jump(to) | Op::ForStart { exit: to, .. } => *to = target,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests;
