use std::collections::VecDeque;

use crate::token::Token;

/// Entries the home screen keeps, oldest dropped first.
const CAPACITY: usize = 50;

/// How an evaluated entry shows: typeset as TeX, and its formatted result.
#[derive(Debug, Clone, PartialEq)]
pub struct Output {
    pub latex: String,
    pub result: String,
}

/// One evaluated entry: what was typed and how it shows.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryLine {
    /// Unique for the calculator's lifetime, so views can key lines by it.
    pub id: u32,
    pub entry: Vec<Token>,
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

    pub fn push(&mut self, entry: Vec<Token>, output: Output) {
        if self.lines.len() == CAPACITY {
            self.lines.pop_front();
        }
        self.lines.push_back(HistoryLine {
            id: self.next_id,
            entry,
            output,
        });
        self.next_id = self.next_id.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// The entry `back` steps before the newest (0 is the newest), wrapping
    /// around to the newest past the oldest.
    pub fn recall(&self, back: usize) -> Option<&[Token]> {
        let len = self.lines.len();
        let index = len.checked_sub(1 + back % len.max(1))?;
        Some(&self.lines[index].entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Digit;

    fn digit(digit: Digit) -> Vec<Token> {
        vec![Token::Digit(digit)]
    }

    fn output(text: &str) -> Output {
        Output {
            latex: text.to_owned(),
            result: text.to_owned(),
        }
    }

    #[test]
    fn recall_walks_back_and_wraps() {
        let mut history = History::default();
        assert_eq!(history.recall(0), None);
        history.push(digit(Digit::One), output("1"));
        history.push(digit(Digit::Two), output("2"));
        assert_eq!(history.recall(0), Some(&digit(Digit::Two)[..]));
        assert_eq!(history.recall(1), Some(&digit(Digit::One)[..]));
        assert_eq!(history.recall(2), Some(&digit(Digit::Two)[..]));
    }

    #[test]
    fn keeps_the_newest_lines_with_unique_ids() {
        let mut history = History::default();
        for _ in 0..CAPACITY + 2 {
            history.push(digit(Digit::Five), output("5"));
        }
        assert_eq!(history.lines().len(), CAPACITY);
        assert_eq!(history.lines()[0].id, 2);
    }
}
