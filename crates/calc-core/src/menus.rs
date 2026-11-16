//! The MATH and TEST menus: tabs of items, each typing one token into the
//! entry the menu was opened from.

use crate::{
    token::Token,
    vocabulary::{DisplayAs, Function, Operator, PostfixOp},
};

/// A menu that types a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    /// MATH: MATH, NUM and PROB.
    Math,
    /// 2nd TEST: TEST and LOGIC.
    Test,
}

/// One item: the key that picks it, its label, and what it types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuEntry {
    pub key: char,
    pub label: &'static str,
    pub token: Token,
}

/// A tab of a menu and its items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuTab {
    pub title: &'static str,
    pub entries: &'static [MenuEntry],
}

const fn entry(key: char, label: &'static str, token: Token) -> MenuEntry {
    MenuEntry { key, label, token }
}

const fn function(key: char, function: Function) -> MenuEntry {
    entry(key, function.text(), Token::Func(function))
}

const fn operator(key: char, label: &'static str, operator: Operator) -> MenuEntry {
    entry(key, label, Token::Op(operator))
}

const MATH: [MenuTab; 3] = [
    MenuTab {
        title: "MATH",
        entries: &[
            entry('1', "►Frac", Token::Display(DisplayAs::Fraction)),
            entry('2', "►Dec", Token::Display(DisplayAs::Decimal)),
            entry('3', "³", Token::Postfix(PostfixOp::Cube)),
            function('4', Function::CubeRoot),
            operator('5', "ˣ√", Operator::XRoot),
            function('A', Function::LogBase),
        ],
    },
    MenuTab {
        title: "NUM",
        entries: &[
            function('1', Function::Abs),
            function('2', Function::Round),
            function('3', Function::IPart),
            function('4', Function::FPart),
            function('5', Function::Int),
            function('6', Function::Min),
            function('7', Function::Max),
            function('8', Function::Lcm),
            function('9', Function::Gcd),
            function('0', Function::Remainder),
        ],
    },
    MenuTab {
        title: "PROB",
        entries: &[
            entry('1', "rand", Token::Rand),
            operator('2', "nPr", Operator::Permutations),
            operator('3', "nCr", Operator::Combinations),
            entry('4', "!", Token::Postfix(PostfixOp::Factorial)),
            function('5', Function::RandInt),
        ],
    },
];

const TEST: [MenuTab; 2] = [
    MenuTab {
        title: "TEST",
        entries: &[
            operator('1', "=", Operator::Equal),
            operator('2', "≠", Operator::NotEqual),
            operator('3', ">", Operator::Greater),
            operator('4', "≥", Operator::GreaterEqual),
            operator('5', "<", Operator::Less),
            operator('6', "≤", Operator::LessEqual),
        ],
    },
    MenuTab {
        title: "LOGIC",
        entries: &[
            operator('1', "and", Operator::And),
            operator('2', "or", Operator::Or),
            operator('3', "xor", Operator::Xor),
            function('4', Function::Not),
        ],
    },
];

impl MenuKind {
    pub fn tabs(self) -> &'static [MenuTab] {
        match self {
            MenuKind::Math => &MATH,
            MenuKind::Test => &TEST,
        }
    }
}

/// An open menu: its tab and highlighted item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuCursor {
    pub kind: MenuKind,
    pub tab: usize,
    pub highlighted: usize,
}

impl MenuCursor {
    pub fn new(kind: MenuKind) -> Self {
        MenuCursor {
            kind,
            tab: 0,
            highlighted: 0,
        }
    }

    pub fn current_tab(self) -> MenuTab {
        self.kind.tabs()[self.tab]
    }

    /// ◀ or ▶: the neighboring tab, wrapping, its first item highlighted.
    pub fn step_tab(self, right: bool) -> Self {
        let count = self.kind.tabs().len();
        let tab = if right {
            (self.tab + 1) % count
        } else {
            (self.tab + count - 1) % count
        };
        MenuCursor {
            tab,
            highlighted: 0,
            ..self
        }
    }

    /// ▲ or ▼: the neighboring item, wrapping.
    pub fn step_item(self, down: bool) -> Self {
        let count = self.current_tab().entries.len();
        let highlighted = if down {
            (self.highlighted + 1) % count
        } else {
            (self.highlighted + count - 1) % count
        };
        MenuCursor {
            highlighted,
            ..self
        }
    }

    pub fn highlighted_token(self) -> Token {
        self.current_tab().entries[self.highlighted].token
    }

    /// The token of the item on this tab that `key` picks, if any.
    pub fn token_for_key(self, key: char) -> Option<Token> {
        let entries = self.current_tab().entries;
        entries
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| entry.token)
    }
}

/// The menu key a typed token stands for: a digit, or a letter typed with ALPHA.
pub fn menu_key(token: Token) -> Option<char> {
    match token {
        Token::Digit(_) | Token::Letter(_) => token.text().chars().next(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Letter;

    #[test]
    fn tabs_and_items_wrap() {
        let cursor = MenuCursor::new(MenuKind::Math).step_tab(false);
        assert_eq!(cursor.current_tab().title, "PROB");
        let cursor = cursor.step_item(false);
        assert_eq!(cursor.highlighted_token(), Token::Func(Function::RandInt));
    }

    #[test]
    fn keys_pick_items_on_the_current_tab() {
        let num = MenuCursor::new(MenuKind::Math).step_tab(true);
        assert_eq!(
            num.token_for_key('0'),
            Some(Token::Func(Function::Remainder))
        );
        let math = MenuCursor::new(MenuKind::Math);
        let key = menu_key(Token::Letter(Letter::A)).unwrap();
        assert_eq!(
            math.token_for_key(key),
            Some(Token::Func(Function::LogBase))
        );
        assert_eq!(math.token_for_key('9'), None);
    }
}
