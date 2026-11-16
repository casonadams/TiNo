use std::{error::Error, fmt, str::FromStr};

/// A physical key of the keypad, independent of 2nd and ALPHA.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    YEquals,
    Window,
    Zoom,
    Trace,
    Graph,
    Second,
    Mode,
    Del,
    Up,
    Down,
    Left,
    Right,
    Alpha,
    XtThetaN,
    Stat,
    Math,
    Apps,
    Prgm,
    Vars,
    Clear,
    Inverse,
    Sin,
    Cos,
    Tan,
    Power,
    Square,
    Comma,
    LeftParen,
    RightParen,
    Divide,
    Log,
    Digit7,
    Digit8,
    Digit9,
    Multiply,
    Ln,
    Digit4,
    Digit5,
    Digit6,
    Subtract,
    Store,
    Digit1,
    Digit2,
    Digit3,
    Add,
    On,
    Digit0,
    Decimal,
    Negate,
    Enter,
}

impl Key {
    /// Every key, in keypad order (top row first, left to right).
    pub const ALL: [Key; 50] = [
        Key::YEquals,
        Key::Window,
        Key::Zoom,
        Key::Trace,
        Key::Graph,
        Key::Second,
        Key::Mode,
        Key::Del,
        Key::Up,
        Key::Down,
        Key::Left,
        Key::Right,
        Key::Alpha,
        Key::XtThetaN,
        Key::Stat,
        Key::Math,
        Key::Apps,
        Key::Prgm,
        Key::Vars,
        Key::Clear,
        Key::Inverse,
        Key::Sin,
        Key::Cos,
        Key::Tan,
        Key::Power,
        Key::Square,
        Key::Comma,
        Key::LeftParen,
        Key::RightParen,
        Key::Divide,
        Key::Log,
        Key::Digit7,
        Key::Digit8,
        Key::Digit9,
        Key::Multiply,
        Key::Ln,
        Key::Digit4,
        Key::Digit5,
        Key::Digit6,
        Key::Subtract,
        Key::Store,
        Key::Digit1,
        Key::Digit2,
        Key::Digit3,
        Key::Add,
        Key::On,
        Key::Digit0,
        Key::Decimal,
        Key::Negate,
        Key::Enter,
    ];

    /// The stable id skins and keyboard maps name this key by.
    pub fn id(self) -> &'static str {
        match self {
            Key::YEquals => "y-equals",
            Key::Window => "window",
            Key::Zoom => "zoom",
            Key::Trace => "trace",
            Key::Graph => "graph",
            Key::Second => "second",
            Key::Mode => "mode",
            Key::Del => "del",
            Key::Up => "up",
            Key::Down => "down",
            Key::Left => "left",
            Key::Right => "right",
            Key::Alpha => "alpha",
            Key::XtThetaN => "xt-theta-n",
            Key::Stat => "stat",
            Key::Math => "math",
            Key::Apps => "apps",
            Key::Prgm => "prgm",
            Key::Vars => "vars",
            Key::Clear => "clear",
            Key::Inverse => "inverse",
            Key::Sin => "sin",
            Key::Cos => "cos",
            Key::Tan => "tan",
            Key::Power => "power",
            Key::Square => "square",
            Key::Comma => "comma",
            Key::LeftParen => "left-paren",
            Key::RightParen => "right-paren",
            Key::Divide => "divide",
            Key::Log => "log",
            Key::Digit7 => "7",
            Key::Digit8 => "8",
            Key::Digit9 => "9",
            Key::Multiply => "multiply",
            Key::Ln => "ln",
            Key::Digit4 => "4",
            Key::Digit5 => "5",
            Key::Digit6 => "6",
            Key::Subtract => "subtract",
            Key::Store => "store",
            Key::Digit1 => "1",
            Key::Digit2 => "2",
            Key::Digit3 => "3",
            Key::Add => "add",
            Key::On => "on",
            Key::Digit0 => "0",
            Key::Decimal => "decimal",
            Key::Negate => "negate",
            Key::Enter => "enter",
        }
    }
}

impl FromStr for Key {
    type Err = UnknownKey;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        Key::ALL
            .into_iter()
            .find(|key| key.id() == id)
            .ok_or_else(|| UnknownKey(id.to_owned()))
    }
}

/// A key id no key has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownKey(pub String);

impl fmt::Display for UnknownKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown key `{}`", self.0)
    }
}

impl Error for UnknownKey {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_key_round_trips_through_a_unique_id() {
        let ids: HashSet<_> = Key::ALL.map(Key::id).into_iter().collect();
        assert_eq!(ids.len(), Key::ALL.len());
        for key in Key::ALL {
            assert_eq!(key.id().parse::<Key>(), Ok(key));
        }
    }

    #[test]
    fn an_unknown_id_is_an_error() {
        assert_eq!("f6".parse::<Key>(), Err(UnknownKey("f6".to_owned())));
    }
}
