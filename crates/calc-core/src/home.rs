use std::collections::VecDeque;

use crate::token::Token;

/// Lines the home screen keeps, and entries 2nd ENTRY can recall; oldest
/// dropped first.
const CAPACITY: usize = 50;

/// How an evaluated entry shows: typeset as TeX, and its formatted result.
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub latex: String,
    pub result: String,
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
}

/// Every entry ENTER ran, failed ones included, for 2nd ENTRY to recall.
#[derive(Debug, Clone, Default)]
pub struct EntryStack {
    entries: VecDeque<Vec<Token>>,
}

impl EntryStack {
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
                latex: "5".into(),
                result: "5".into(),
            });
        }
        assert_eq!(history.lines().len(), CAPACITY);
        assert_eq!(history.lines()[0].id, 2);
    }
}
