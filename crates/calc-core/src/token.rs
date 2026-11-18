use serde::{Deserialize, Serialize};

pub use crate::vocabulary::{DisplayAs, Function, Keyword, Operator, PostfixOp};
use crate::{field_editor::Field, table::TableField, window_editor::WindowField};

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
    pub const ALL: [Letter; 27] = [
        Letter::A,
        Letter::B,
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::H,
        Letter::I,
        Letter::J,
        Letter::K,
        Letter::L,
        Letter::M,
        Letter::N,
        Letter::O,
        Letter::P,
        Letter::Q,
        Letter::R,
        Letter::S,
        Letter::T,
        Letter::U,
        Letter::V,
        Letter::W,
        Letter::X,
        Letter::Y,
        Letter::Z,
        Letter::Theta,
    ];

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

impl Digit {
    pub const ALL: [Digit; 10] = [
        Digit::Zero,
        Digit::One,
        Digit::Two,
        Digit::Three,
        Digit::Four,
        Digit::Five,
        Digit::Six,
        Digit::Seven,
        Digit::Eight,
        Digit::Nine,
    ];
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

impl ListName {
    pub const ALL: [ListName; 6] = [
        ListName::L1,
        ListName::L2,
        ListName::L3,
        ListName::L4,
        ListName::L5,
        ListName::L6,
    ];
}

/// A built-in matrix, `[A]` to `[J]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MatrixName {
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
}

impl MatrixName {
    pub const ALL: [MatrixName; 10] = [
        MatrixName::A,
        MatrixName::B,
        MatrixName::C,
        MatrixName::D,
        MatrixName::E,
        MatrixName::F,
        MatrixName::G,
        MatrixName::H,
        MatrixName::I,
        MatrixName::J,
    ];

    pub const fn text(self) -> &'static str {
        [
            "[A]", "[B]", "[C]", "[D]", "[E]", "[F]", "[G]", "[H]", "[I]", "[J]",
        ][self as usize]
    }
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
    Matrix(MatrixName),
    SequenceU,
    SequenceV,
    SequenceW,
    Colon,
    Quote,
    Space,
    Question,
    /// A WINDOW setting from VARS (`Xmin`).
    Window(WindowField),
    /// A TBLSET setting from VARS (`TblStart`).
    Table(TableField),
    /// A Y= function from VARS by slot (`Y₁` is slot 0, `Y₀` slot 9).
    YVar(usize),
    /// `prgm`, followed by a program's name to run it.
    Prgm,
    /// A program command from the editor's PRGM menu (`If `, `Disp `).
    Keyword(Keyword),
}

const Y_VAR_NAMES: [&str; 10] = ["Y₁", "Y₂", "Y₃", "Y₄", "Y₅", "Y₆", "Y₇", "Y₈", "Y₉", "Y₀"];

impl Token {
    /// How the token reads on screen.
    pub fn text(self) -> &'static str {
        match self {
            Token::Digit(digit) => {
                ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"][digit as usize]
            }
            Token::Letter(letter) => letter.symbol(),
            Token::List(list) => ["L₁", "L₂", "L₃", "L₄", "L₅", "L₆"][list as usize],
            Token::Matrix(name) => name.text(),
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
            Token::Window(field) => field.label(),
            Token::Table(field) => field.label(),
            Token::YVar(slot) => Y_VAR_NAMES[slot % Y_VAR_NAMES.len()],
            Token::Prgm => "prgm",
            Token::Keyword(keyword) => keyword.text(),
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
