use crate::{
    error::{CalcError, ErrorKind},
    expr::{BinaryOp, Constant, Expr, Function, Number, Postfix, Statement},
    token::Token,
};

/// Parses an entry with the calculator's rules: `^` groups left to right,
/// negation binds looser than `^` (`⁻2²` is `⁻4`), juxtaposition multiplies
/// as `×` does, and parentheses left open close at the end of the entry.
pub fn parse(tokens: &[Token]) -> Result<Statement, CalcError> {
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.sum()?;
    let store = parser.store_target()?;
    match parser.peek() {
        None => Ok(Statement { expr, store }),
        Some(_) => Err(parser.error(ErrorKind::Syntax)),
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.peek();
        self.pos += usize::from(token.is_some());
        token
    }

    fn eat(&mut self, token: Token) -> bool {
        let found = self.peek() == Some(token);
        self.pos += usize::from(found);
        found
    }

    fn error(&self, kind: ErrorKind) -> CalcError {
        CalcError::at(kind, self.pos)
    }

    fn store_target(&mut self) -> Result<Option<crate::token::Letter>, CalcError> {
        if !self.eat(Token::Store) {
            return Ok(None);
        }
        let target = self.pos;
        match self.advance() {
            Some(Token::Letter(letter)) => Ok(Some(letter)),
            _ => Err(CalcError::at(ErrorKind::Syntax, target)),
        }
    }

    fn sum(&mut self) -> Result<Expr, CalcError> {
        let mut left = self.product()?;
        loop {
            let op = match self.peek() {
                Some(Token::Add) => BinaryOp::Add,
                Some(Token::Subtract) => BinaryOp::Subtract,
                _ => return Ok(left),
            };
            self.pos += 1;
            left = Expr::Binary(op, Box::new(left), Box::new(self.product()?));
        }
    }

    fn product(&mut self) -> Result<Expr, CalcError> {
        let mut left = self.signed()?;
        loop {
            let op = match self.peek() {
                Some(Token::Multiply) => BinaryOp::Multiply,
                Some(Token::Divide) => BinaryOp::Divide,
                Some(token) if starts_implied_factor(token) => BinaryOp::ImpliedMultiply,
                _ => return Ok(left),
            };
            self.pos += usize::from(op != BinaryOp::ImpliedMultiply);
            left = Expr::Binary(op, Box::new(left), Box::new(self.signed()?));
        }
    }

    fn signed(&mut self) -> Result<Expr, CalcError> {
        if self.eat(Token::Negate) {
            return Ok(Expr::Negate(Box::new(self.signed()?)));
        }
        self.power()
    }

    fn power(&mut self) -> Result<Expr, CalcError> {
        let mut base = self.postfix()?;
        while self.eat(Token::Power) {
            base = Expr::Binary(BinaryOp::Power, Box::new(base), Box::new(self.exponent()?));
        }
        Ok(base)
    }

    fn exponent(&mut self) -> Result<Expr, CalcError> {
        if self.eat(Token::Negate) {
            return Ok(Expr::Negate(Box::new(self.exponent()?)));
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, CalcError> {
        let mut operand = self.primary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Square) => Postfix::Square,
                Some(Token::Inverse) => Postfix::Inverse,
                _ => return Ok(operand),
            };
            self.pos += 1;
            operand = Expr::Postfix(op, Box::new(operand));
        }
    }

    fn primary(&mut self) -> Result<Expr, CalcError> {
        let Some(token) = self.peek() else {
            return Err(self.error(ErrorKind::Syntax));
        };
        if starts_number(token) {
            return self.number();
        }
        if let Some(function) = function(token) {
            self.pos += 1;
            return Ok(Expr::Function(function, Box::new(self.group()?)));
        }
        let expr = atom(token).ok_or_else(|| self.error(unexpected(token)))?;
        self.pos += 1;
        match expr {
            Some(expr) => Ok(expr),
            None => self.group(),
        }
    }

    /// The inside of a parenthesis whose `(` was consumed; its `)` may be
    /// left out at the end of the entry or before `→`.
    fn group(&mut self) -> Result<Expr, CalcError> {
        let inner = self.sum()?;
        match self.peek() {
            Some(Token::RightParen) => self.pos += 1,
            None | Some(Token::Store) => {}
            Some(_) => return Err(self.error(ErrorKind::Syntax)),
        }
        Ok(inner)
    }

    fn number(&mut self) -> Result<Expr, CalcError> {
        let start = self.pos;
        let mantissa = self.collect(|t| matches!(t, Token::Digit(_) | Token::DecimalPoint));
        let exponent = self.scientific_exponent()?;
        let invalid = mantissa.matches('.').count() > 1 || mantissa == ".";
        let literal = format!(
            "{}e{}",
            if mantissa.is_empty() { "1" } else { &mantissa },
            exponent.as_deref().unwrap_or("0")
        );
        match literal.parse::<f64>() {
            Ok(value) if !invalid => Ok(Expr::Number(Number {
                value,
                mantissa,
                exponent,
            })),
            _ => Err(CalcError::at(ErrorKind::Syntax, start)),
        }
    }

    fn scientific_exponent(&mut self) -> Result<Option<String>, CalcError> {
        if !self.eat(Token::Exponent) {
            return Ok(None);
        }
        let sign = if self.eat(Token::Negate) { "-" } else { "" };
        let digits = self.collect(|t| matches!(t, Token::Digit(_)));
        if digits.is_empty() {
            return Err(self.error(ErrorKind::Syntax));
        }
        Ok(Some(format!("{sign}{digits}")))
    }

    fn collect(&mut self, accept: impl Fn(Token) -> bool) -> String {
        let mut text = String::new();
        while let Some(token) = self.peek().filter(|&t| accept(t)) {
            text.push(if token == Token::DecimalPoint {
                '.'
            } else {
                digit_char(token)
            });
            self.pos += 1;
        }
        text
    }
}

fn digit_char(token: Token) -> char {
    match token {
        Token::Digit(digit) => char::from(b'0' + digit as u8),
        _ => '?',
    }
}

fn starts_number(token: Token) -> bool {
    matches!(
        token,
        Token::Digit(_) | Token::DecimalPoint | Token::Exponent
    )
}

/// Whether `token` begins a factor that multiplies what precedes it.
fn starts_implied_factor(token: Token) -> bool {
    starts_number(token)
        || token == Token::Negate
        || function(token).is_some()
        || atom(token).is_some()
}

fn function(token: Token) -> Option<Function> {
    Some(match token {
        Token::Sqrt => Function::Sqrt,
        Token::Sin => Function::Sin,
        Token::Cos => Function::Cos,
        Token::Tan => Function::Tan,
        Token::ArcSin => Function::ArcSin,
        Token::ArcCos => Function::ArcCos,
        Token::ArcTan => Function::ArcTan,
        Token::Log => Function::Log,
        Token::Ln => Function::Ln,
        Token::TenPower => Function::TenPower,
        Token::ExpPower => Function::ExpPower,
        _ => return None,
    })
}

/// A one-token operand, or `Some(None)` for `(`, which opens a group.
fn atom(token: Token) -> Option<Option<Expr>> {
    Some(match token {
        Token::Pi => Some(Expr::Constant(Constant::Pi)),
        Token::EulerE => Some(Expr::Constant(Constant::E)),
        Token::Letter(letter) => Some(Expr::Variable(letter)),
        Token::Ans => Some(Expr::Ans),
        Token::LeftParen => None,
        _ => return None,
    })
}

/// What a token that cannot start an operand means here: calculator syntax
/// not evaluated yet, or a syntax error.
fn unexpected(token: Token) -> ErrorKind {
    match token {
        Token::ImaginaryI
        | Token::List(_)
        | Token::LeftBrace
        | Token::LeftBracket
        | Token::SequenceU
        | Token::SequenceV
        | Token::SequenceW
        | Token::Quote
        | Token::Colon
        | Token::Space
        | Token::Question => ErrorKind::Unsupported,
        _ => ErrorKind::Syntax,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::token::{Digit, Letter};

    /// Tokens for a compact spelling: digits, `.`, `+-*/^`, `_` negate,
    /// `()`, `E` exponent, `p` π, `²`, `!` inverse, `>` store, `A`-`D` and
    /// `X` letters, `a` Ans, `s` sin(, `r` √(.
    pub(crate) fn tokens(spelling: &str) -> Vec<Token> {
        spelling.chars().map(token_for).collect()
    }

    fn token_for(c: char) -> Token {
        const DIGITS: [Digit; 10] = [
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
        if let Some(d) = c.to_digit(10) {
            return Token::Digit(DIGITS[d as usize]);
        }
        symbol_token(c)
    }

    fn symbol_token(c: char) -> Token {
        match c {
            '.' => Token::DecimalPoint,
            '+' => Token::Add,
            '-' => Token::Subtract,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            '^' => Token::Power,
            '_' => Token::Negate,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            'E' => Token::Exponent,
            'p' => Token::Pi,
            '²' => Token::Square,
            '!' => Token::Inverse,
            '>' => Token::Store,
            'A' => Token::Letter(Letter::A),
            'B' => Token::Letter(Letter::B),
            'C' => Token::Letter(Letter::C),
            'D' => Token::Letter(Letter::D),
            'X' => Token::Letter(Letter::X),
            'a' => Token::Ans,
            's' => Token::Sin,
            'r' => Token::Sqrt,
            'L' => Token::List(crate::token::ListName::L1),
            other => panic!("no token spelled {other:?}"),
        }
    }

    fn error_of(spelling: &str) -> CalcError {
        parse(&tokens(spelling)).unwrap_err()
    }

    #[test]
    fn syntax_errors_point_at_the_offending_token() {
        let cases = [
            ("2+", 2),
            ("*3", 0),
            ("1.2.3", 0),
            ("2E", 2),
            ("3)", 1),
            ("4>", 2),
            ("4>5", 2),
        ];
        for (spelling, position) in cases {
            assert_eq!(
                error_of(spelling),
                CalcError::at(ErrorKind::Syntax, position),
                "{spelling}"
            );
        }
    }

    #[test]
    fn syntax_not_evaluated_yet_is_reported_as_unsupported() {
        assert_eq!(error_of("2+L").kind, ErrorKind::Unsupported);
    }

    #[test]
    fn a_trailing_store_names_the_variable() {
        let statement = parse(&tokens("5>B")).unwrap();
        assert_eq!(statement.store, Some(Letter::B));
    }
}
