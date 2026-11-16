use serde::{Deserialize, Serialize};

pub use crate::vocabulary::{DisplayAs, Function, Operator, PostfixOp};

/// A named variable: `A` to `Z` and `θ`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Letter {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Theta,
}

const LETTER_SYMBOLS: [&str; 27] = [
    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S",
    "T", "U", "V", "W", "X", "Y", "Z", "θ",
];

impl Letter {
    pub fn symbol(self) -> &'static str {
        LETTER_SYMBOLS[self as usize]
    }
}

/// A decimal digit, 0 to 9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Digit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

/// A built-in list, `L₁` to `L₆`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ListName {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

/// One unit of an entry: what one key press or menu pick inserts and one
/// DEL removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Token {
    Digit(Digit),
    DecimalPoint,
    Exponent,
    Negate,
    Op(Operator),
    Postfix(PostfixOp),
    Func(Function),
    Display(DisplayAs),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Pi,
    EulerE,
    ImaginaryI,
    Ans,
    Rand,
    Store,
    Letter(Letter),
    List(ListName),
    SequenceU,
    SequenceV,
    SequenceW,
    Colon,
    Quote,
    Space,
    Question,
}

impl Token {
    /// How the token reads on screen.
    pub fn text(self) -> &'static str {
        match self {
            Token::Digit(digit) => {
                ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"][digit as usize]
            }
            Token::Letter(letter) => letter.symbol(),
            Token::List(list) => ["L₁", "L₂", "L₃", "L₄", "L₅", "L₆"][list as usize],
            Token::Op(op) => op.text(),
            Token::Postfix(op) => op.text(),
            Token::Func(function) => function.text(),
            Token::Display(display) => display.text(),
            Token::DecimalPoint => ".",
            Token::Exponent => "ᴇ",
            Token::Negate => "⁻",
            Token::LeftParen => "(",
            Token::RightParen => ")",
            Token::LeftBrace => "{",
            Token::RightBrace => "}",
            Token::LeftBracket => "[",
            Token::RightBracket => "]",
            Token::Comma => ",",
            Token::Pi => "π",
            Token::EulerE => "e",
            Token::ImaginaryI => "i",
            Token::Ans => "Ans",
            Token::Rand => "rand",
            Token::Store => "→",
            Token::SequenceU => "u",
            Token::SequenceV => "v",
            Token::SequenceW => "w",
            Token::Colon => ":",
            Token::Quote => "\"",
            Token::Space => " ",
            Token::Question => "?",
        }
    }

    /// Whether the token operates on a value before it (`+`, `²`, `→`), so
    /// typing it first on the home screen applies it to `Ans`.
    pub fn takes_left_operand(self) -> bool {
        matches!(
            self,
            Token::Op(_) | Token::Postfix(_) | Token::Display(_) | Token::Store
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_read_as_the_calculator_shows_them() {
        let cases = [
            (Token::Digit(Digit::Seven), "7"),
            (Token::Letter(Letter::Theta), "θ"),
            (Token::List(ListName::L6), "L₆"),
            (Token::Op(Operator::Subtract), "−"),
            (Token::Negate, "⁻"),
            (Token::Func(Function::ArcSin), "sin⁻¹("),
            (Token::Store, "→"),
        ];
        for (token, text) in cases {
            assert_eq!(token.text(), text, "{token:?}");
        }
    }
}
