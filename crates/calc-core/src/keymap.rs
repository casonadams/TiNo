use crate::{
    key::Key,
    token::{Digit, Letter, ListName, Token},
    vocabulary::{Function as F, Operator as O, PostfixOp as P},
};

/// Which legend of a key a press means: its own, its 2nd or its ALPHA one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Normal,
    Second,
    Alpha,
}

/// What a key press does once 2nd and ALPHA are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Insert(Token),
    Edit(Edit),
    Command(Command),
}

/// A change to the entry line that inserts nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edit {
    Delete,
    ToggleInsert,
    Clear,
    Left,
    Right,
}

/// A press that opens a screen or menu, or acts on the whole calculator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    YEquals,
    StatPlot,
    Window,
    TableSetup,
    Zoom,
    Format,
    Trace,
    Calc,
    Graph,
    Table,
    Shortcut(FunctionKey),
    Mode,
    Quit,
    Link,
    Stat,
    List,
    Math,
    Test,
    Apps,
    Angle,
    Prgm,
    Draw,
    Vars,
    Distr,
    Matrix,
    Recall,
    Mem,
    Catalog,
    Up,
    Down,
    Enter,
    Entry,
    Solve,
    On,
    Off,
}

/// The shortcut menus ALPHA opens from the top row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionKey {
    F1,
    F2,
    F3,
    F4,
    F5,
}

/// What `key` does on `layer`; `None` when that legend is empty. 2nd and
/// ALPHA themselves change the layer and have no action.
pub fn action(key: Key, layer: Layer) -> Option<Action> {
    let [normal, second, alpha] = legends(key);
    match layer {
        Layer::Normal => normal,
        Layer::Second => second,
        Layer::Alpha => alpha,
    }
}

type Legends = [Option<Action>; 3];

fn legends(key: Key) -> Legends {
    use Command as C;
    use Token as T;
    match key {
        Key::Second | Key::Alpha => [None; 3],
        Key::YEquals => [cmd(C::YEquals), cmd(C::StatPlot), shortcut(FunctionKey::F1)],
        Key::Window => [
            cmd(C::Window),
            cmd(C::TableSetup),
            shortcut(FunctionKey::F2),
        ],
        Key::Zoom => [cmd(C::Zoom), cmd(C::Format), shortcut(FunctionKey::F3)],
        Key::Trace => [cmd(C::Trace), cmd(C::Calc), shortcut(FunctionKey::F4)],
        Key::Graph => [cmd(C::Graph), cmd(C::Table), shortcut(FunctionKey::F5)],
        Key::Mode => [cmd(C::Mode), cmd(C::Quit), None],
        Key::Del => [edit(Edit::Delete), edit(Edit::ToggleInsert), None],
        Key::Clear => [edit(Edit::Clear); 3],
        Key::Left => [edit(Edit::Left); 3],
        Key::Right => [edit(Edit::Right); 3],
        Key::Up => [cmd(C::Up); 3],
        Key::Down => [cmd(C::Down); 3],
        Key::XtThetaN => [ins(T::Letter(Letter::X)), cmd(C::Link), None],
        Key::Stat => [cmd(C::Stat), cmd(C::List), None],
        Key::Math => [cmd(C::Math), cmd(C::Test), letter(Letter::A)],
        Key::Apps => [cmd(C::Apps), cmd(C::Angle), letter(Letter::B)],
        Key::Prgm => [cmd(C::Prgm), cmd(C::Draw), letter(Letter::C)],
        Key::Vars => [cmd(C::Vars), cmd(C::Distr), None],
        Key::Inverse => [post(P::Inverse), cmd(C::Matrix), None],
        Key::Sin => [func(F::Sin), func(F::ArcSin), letter(Letter::E)],
        Key::Cos => [func(F::Cos), func(F::ArcCos), letter(Letter::F)],
        Key::Tan => [func(F::Tan), func(F::ArcTan), letter(Letter::G)],
        Key::Power => [op(O::Power), ins(T::Pi), letter(Letter::H)],
        Key::Square => [post(P::Square), func(F::Sqrt), letter(Letter::I)],
        Key::Comma => [ins(T::Comma), ins(T::Exponent), letter(Letter::J)],
        Key::LeftParen => [ins(T::LeftParen), ins(T::LeftBrace), letter(Letter::K)],
        Key::RightParen => [ins(T::RightParen), ins(T::RightBrace), letter(Letter::L)],
        Key::Divide => [op(O::Divide), ins(T::EulerE), letter(Letter::M)],
        Key::Log => [func(F::Log), func(F::TenPower), letter(Letter::N)],
        Key::Multiply => [op(O::Multiply), ins(T::LeftBracket), letter(Letter::R)],
        Key::Ln => [func(F::Ln), func(F::ExpPower), letter(Letter::S)],
        Key::Subtract => [op(O::Subtract), ins(T::RightBracket), letter(Letter::W)],
        Key::Store => [ins(T::Store), cmd(C::Recall), letter(Letter::X)],
        Key::Add => [op(O::Add), cmd(C::Mem), ins(T::Quote)],
        Key::On => [cmd(C::On), cmd(C::Off), None],
        Key::Decimal => [ins(T::DecimalPoint), ins(T::ImaginaryI), ins(T::Colon)],
        Key::Negate => [ins(T::Negate), ins(T::Ans), ins(T::Question)],
        Key::Enter => [cmd(C::Enter), cmd(C::Entry), cmd(C::Solve)],
        digit_key => digit_legends(digit_key),
    }
}

fn digit_legends(key: Key) -> Legends {
    let (digit, second, alpha) = match key {
        Key::Digit0 => (Digit::Zero, cmd(Command::Catalog), ins(Token::Space)),
        Key::Digit1 => (Digit::One, list(ListName::L1), letter(Letter::Y)),
        Key::Digit2 => (Digit::Two, list(ListName::L2), letter(Letter::Z)),
        Key::Digit3 => (Digit::Three, list(ListName::L3), letter(Letter::Theta)),
        Key::Digit4 => (Digit::Four, list(ListName::L4), letter(Letter::T)),
        Key::Digit5 => (Digit::Five, list(ListName::L5), letter(Letter::U)),
        Key::Digit6 => (Digit::Six, list(ListName::L6), letter(Letter::V)),
        Key::Digit7 => (Digit::Seven, ins(Token::SequenceU), letter(Letter::O)),
        Key::Digit8 => (Digit::Eight, ins(Token::SequenceV), letter(Letter::P)),
        Key::Digit9 => (Digit::Nine, ins(Token::SequenceW), letter(Letter::Q)),
        _ => return [None; 3],
    };
    [ins(Token::Digit(digit)), second, alpha]
}

fn ins(token: Token) -> Option<Action> {
    Some(Action::Insert(token))
}

fn op(operator: O) -> Option<Action> {
    ins(Token::Op(operator))
}

fn post(operator: P) -> Option<Action> {
    ins(Token::Postfix(operator))
}

fn func(function: F) -> Option<Action> {
    ins(Token::Func(function))
}

fn letter(letter: Letter) -> Option<Action> {
    ins(Token::Letter(letter))
}

fn list(list: ListName) -> Option<Action> {
    ins(Token::List(list))
}

fn edit(edit: Edit) -> Option<Action> {
    Some(Action::Edit(edit))
}

fn cmd(command: Command) -> Option<Action> {
    Some(Action::Command(command))
}

fn shortcut(key: FunctionKey) -> Option<Action> {
    cmd(Command::Shortcut(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_layer_reads_its_own_legend() {
        let cases = [
            (Key::Sin, Layer::Normal, func(F::Sin)),
            (Key::Sin, Layer::Second, func(F::ArcSin)),
            (Key::Sin, Layer::Alpha, letter(Letter::E)),
            (Key::Digit3, Layer::Second, list(ListName::L3)),
            (Key::Digit3, Layer::Alpha, letter(Letter::Theta)),
            (Key::Negate, Layer::Second, ins(Token::Ans)),
            (Key::Del, Layer::Second, edit(Edit::ToggleInsert)),
            (Key::Graph, Layer::Second, cmd(Command::Table)),
            (Key::Mode, Layer::Alpha, None),
            (Key::Clear, Layer::Alpha, edit(Edit::Clear)),
        ];
        for (key, layer, expected) in cases {
            assert_eq!(action(key, layer), expected, "{key:?} on {layer:?}");
        }
    }

    #[test]
    fn modifier_keys_have_no_action() {
        for layer in [Layer::Normal, Layer::Second, Layer::Alpha] {
            assert_eq!(action(Key::Second, layer), None);
            assert_eq!(action(Key::Alpha, layer), None);
        }
    }

    #[test]
    fn every_other_key_does_something_unshifted() {
        let modifiers = [Key::Second, Key::Alpha];
        for key in Key::ALL.into_iter().filter(|k| !modifiers.contains(k)) {
            assert!(action(key, Layer::Normal).is_some(), "{key:?}");
        }
    }
}
