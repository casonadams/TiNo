/// A named variable: `A` to `Z` and `θ`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListName {
    L1,
    L2,
    L3,
    L4,
    L5,
    L6,
}

/// One unit of an entry: what one key press inserts and one DEL removes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    Digit(Digit),
    DecimalPoint,
    Exponent,
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Negate,
    Square,
    Inverse,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Sqrt,
    Sin,
    Cos,
    Tan,
    ArcSin,
    ArcCos,
    ArcTan,
    Log,
    Ln,
    TenPower,
    ExpPower,
    Pi,
    EulerE,
    ImaginaryI,
    Ans,
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
            Token::DecimalPoint => ".",
            Token::Exponent => "ᴇ",
            Token::Add => "+",
            Token::Subtract => "−",
            Token::Multiply => "×",
            Token::Divide => "÷",
            Token::Power => "^",
            Token::Negate => "⁻",
            Token::Square => "²",
            Token::Inverse => "⁻¹",
            Token::Sqrt => "√(",
            Token::Sin => "sin(",
            Token::Cos => "cos(",
            Token::Tan => "tan(",
            Token::ArcSin => "sin⁻¹(",
            Token::ArcCos => "cos⁻¹(",
            Token::ArcTan => "tan⁻¹(",
            Token::Log => "log(",
            Token::Ln => "ln(",
            Token::TenPower => "10^(",
            Token::ExpPower => "e^(",
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
            (Token::Subtract, "−"),
            (Token::Negate, "⁻"),
            (Token::ArcSin, "sin⁻¹("),
            (Token::Store, "→"),
        ];
        for (token, text) in cases {
            assert_eq!(token.text(), text, "{token:?}");
        }
    }
}
