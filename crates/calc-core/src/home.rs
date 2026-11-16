use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::{format::result_tokens, token::Token};

/// Lines the home screen keeps, and entries 2nd ENTRY can recall; oldest
/// dropped first.
const CAPACITY: usize = 50;

/// How an evaluated entry shows: typeset as TeX, and its formatted result;
/// with the entry's tokens, for ▲ ▼ to paste back.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Output {
    #[serde(default)]
    pub entry: Vec<Token>,
    pub latex: String,
    pub result: String,
}

/// The half of a history line ▲ ▼ highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryPart {
    Entry,
    Result,
}

/// A highlighted entry or result in the history; `line` counts from the
/// oldest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistorySelection {
    pub line: usize,
    pub part: HistoryPart,
}

impl HistorySelection {
    /// ▲ from the entry line: the newest result, if any.
    pub fn newest(lines: usize) -> Option<Self> {
        let line = lines.checked_sub(1)?;
        Some(HistorySelection {
            line,
            part: HistoryPart::Result,
        })
    }

    /// ▲: the entry above a result, or the result above an entry; the
    /// oldest entry stays.
    pub fn up(self) -> Self {
        match (self.part, self.line) {
            (HistoryPart::Result, line) => HistorySelection {
                line,
                part: HistoryPart::Entry,
            },
            (HistoryPart::Entry, 0) => self,
            (HistoryPart::Entry, line) => HistorySelection {
                line: line - 1,
                part: HistoryPart::Result,
            },
        }
    }

    /// ▼: the part below in a history of `lines` lines; `None` past the
    /// newest result, back on the entry line.
    pub fn down(self, lines: usize) -> Option<Self> {
        match self.part {
            HistoryPart::Entry => Some(HistorySelection {
                part: HistoryPart::Result,
                ..self
            }),
            HistoryPart::Result if self.line + 1 < lines => Some(HistorySelection {
                line: self.line + 1,
                part: HistoryPart::Entry,
            }),
            HistoryPart::Result => None,
        }
    }
}

/// One evaluated entry on the home screen.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryLine {
    /// Unique for the calculator's lifetime, so views can key lines by it.
    pub id: u32,
    pub output: Output,
}

/// The home screen's evaluated entries, oldest first.
#[derive(Debug, Clone, Default)]
pub struct History {
    lines: VecDeque<HistoryLine>,
    next_id: u32,
}

impl History {
    pub fn lines(&self) -> &VecDeque<HistoryLine> {
        &self.lines
    }

    pub fn push(&mut self, output: Output) {
        if self.lines.len() == CAPACITY {
            self.lines.pop_front();
        }
        self.lines.push_back(HistoryLine {
            id: self.next_id,
            output,
        });
        self.next_id = self.next_id.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// The tokens that type the selected entry or result.
    pub fn pasted(&self, selection: HistorySelection) -> Vec<Token> {
        let Some(line) = self.lines.get(selection.line) else {
            return Vec::new();
        };
        match selection.part {
            HistoryPart::Entry => line.output.entry.clone(),
            HistoryPart::Result => result_tokens(&line.output.result),
        }
    }
}

/// Every entry ENTER ran, failed ones included, for 2nd ENTRY to recall.
#[derive(Debug, Clone, Default)]
pub struct EntryStack {
    entries: VecDeque<Vec<Token>>,
}

impl EntryStack {
    /// The entries, oldest first.
    pub fn entries(&self) -> &VecDeque<Vec<Token>> {
        &self.entries
    }

    pub fn push(&mut self, entry: Vec<Token>) {
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// The entry `back` steps before the newest (0 is the newest), wrapping
    /// around to the newest past the oldest.
    pub fn recall(&self, back: usize) -> Option<&[Token]> {
        let len = self.entries.len();
        let index = len.checked_sub(1 + back % len.max(1))?;
        Some(&self.entries[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Digit;

    fn digit(digit: Digit) -> Vec<Token> {
        vec![Token::Digit(digit)]
    }

    #[test]
    fn recall_walks_back_and_wraps() {
        let mut entries = EntryStack::default();
        assert_eq!(entries.recall(0), None);
        entries.push(digit(Digit::One));
        entries.push(digit(Digit::Two));
        assert_eq!(entries.recall(0), Some(&digit(Digit::Two)[..]));
        assert_eq!(entries.recall(1), Some(&digit(Digit::One)[..]));
        assert_eq!(entries.recall(2), Some(&digit(Digit::Two)[..]));
    }

    #[test]
    fn history_keeps_the_newest_lines_with_unique_ids() {
        let mut history = History::default();
        for _ in 0..CAPACITY + 2 {
            history.push(Output {
                entry: digit(Digit::Five),
                latex: "5".into(),
                result: "5".into(),
            });
        }
        assert_eq!(history.lines().len(), CAPACITY);
        assert_eq!(history.lines()[0].id, 2);
    }
}
