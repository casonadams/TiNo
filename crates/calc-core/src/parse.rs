use crate::{
    error::{CalcError, ErrorKind},
    expr::{Constant, Expr, Number, Precedence, Statement, StoreTarget},
    token::Token,
    vocabulary::{DisplayAs, Operator},
};

/// Parses an entry with the calculator's rules: `^` groups left to right,
/// negation binds looser than `^` (`⁻2²` is `⁻4`), juxtaposition multiplies
/// as `×` does, functions take comma-separated arguments, and parentheses
/// left open close at the end of the entry.
pub fn parse(tokens: &[Token]) -> Result<Statement, CalcError> {
    let mut parser = Parser { tokens, pos: 0 };
    let expr = parser.expression()?;
    let display = parser.display();
    let store = parser.store_target()?;
    match parser.peek() {
        None => Ok(Statement {
            expr,
            display,
            store,
        }),
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

    fn display(&mut self) -> Option<DisplayAs> {
        let Some(Token::Display(display)) = self.peek() else {
            return None;
        };
        self.pos += 1;
        Some(display)
    }

    fn store_target(&mut self) -> Result<Option<StoreTarget>, CalcError> {
        if !self.eat(Token::Store) {
            return Ok(None);
        }
        let target = self.pos;
        match self.advance() {
            Some(Token::Letter(letter)) => Ok(Some(StoreTarget::Variable(letter))),
            Some(Token::List(name)) => Ok(Some(StoreTarget::List(name))),
            _ => Err(CalcError::at(ErrorKind::Syntax, target)),
        }
    }

    fn expression(&mut self) -> Result<Expr, CalcError> {
        self.infix(Precedence::Either)
    }

    /// The operators of `level`, grouping left to right, between operands
    /// of the next tighter level.
    fn infix(&mut self, level: Precedence) -> Result<Expr, CalcError> {
        match level {
            Precedence::Product => return self.product(),
            Precedence::Negation => return self.signed(),
            _ => {}
        }
        let tighter = tighter(level);
        let mut left = self.infix(tighter)?;
        while let Some(op) = self.operator_of(level) {
            let at = self.pos;
            self.pos += 1;
            left = binary(op, (left, self.infix(tighter)?), at);
        }
        Ok(left)
    }

    fn operator_of(&self, level: Precedence) -> Option<Operator> {
        match self.peek() {
            Some(Token::Op(op)) if op.precedence() == level => Some(op),
            _ => None,
        }
    }

    fn product(&mut self) -> Result<Expr, CalcError> {
        let mut left = self.infix(Precedence::Combination)?;
        loop {
            left = match (self.operator_of(Precedence::Product), self.peek()) {
                (Some(op), _) => {
                    let at = self.pos;
                    self.pos += 1;
                    binary(op, (left, self.infix(Precedence::Combination)?), at)
                }
                (None, Some(token)) if starts_implied_factor(token) => {
                    let right = self.infix(Precedence::Combination)?;
                    Expr::ImpliedMultiply(Box::new(left), Box::new(right))
                }
                _ => return Ok(left),
            };
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
        while let Some(op) = self.operator_of(Precedence::Power) {
            let at = self.pos;
            self.pos += 1;
            base = binary(op, (base, self.exponent()?), at);
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
        while let Some(Token::Postfix(op)) = self.peek() {
            operand = Expr::Postfix(op, Box::new(operand), self.pos);
            self.pos += 1;
        }
        Ok(operand)
    }

    fn primary(&mut self) -> Result<Expr, CalcError> {
        let Some(token) = self.peek() else {
            return Err(self.error(ErrorKind::Syntax));
        };
        if starts_number(token) {
            return self.number();
        }
        let at = self.pos;
        self.pos += 1;
        match token {
            Token::Func(function) => Ok(Expr::Call(function, self.arguments()?, at)),
            Token::YVar(slot) => self.y_var(slot),
            Token::List(name) => Ok(Expr::ListName(name, self.index()?)),
            Token::LeftBrace => self.list(),
            Token::LeftParen => self.group(),
            _ => atom(token).ok_or_else(|| CalcError::at(unexpected(token), at)),
        }
    }

    /// A Y= function, its token consumed: `Y₁(3)` at 3 when a `(` follows.
    fn y_var(&mut self, slot: usize) -> Result<Expr, CalcError> {
        Ok(Expr::YVar(slot, self.index()?))
    }

    /// A parenthesized argument right after a name (`Y₁(3)`, `L₁(2)`), if any.
    fn index(&mut self) -> Result<Option<Box<Expr>>, CalcError> {
        if self.eat(Token::LeftParen) {
            Ok(Some(Box::new(self.group()?)))
        } else {
            Ok(None)
        }
    }

    /// A list's comma-separated elements, its `{` consumed; the `}` may be
    /// left out where a `)` may.
    fn list(&mut self) -> Result<Expr, CalcError> {
        let mut elements = vec![self.expression()?];
        while self.eat(Token::Comma) {
            elements.push(self.expression()?);
        }
        match self.peek() {
            Some(Token::RightBrace) => self.pos += 1,
            None | Some(Token::Store | Token::Display(_)) => {}
            Some(_) => return Err(self.error(ErrorKind::Syntax)),
        }
        Ok(Expr::List(elements))
    }

    /// The inside of a parenthesis whose `(` was consumed.
    fn group(&mut self) -> Result<Expr, CalcError> {
        let inner = self.expression()?;
        self.close()?;
        Ok(inner)
    }

    /// A function's comma-separated arguments, its `(` consumed.
    fn arguments(&mut self) -> Result<Vec<Expr>, CalcError> {
        let mut arguments = vec![self.expression()?];
        while self.eat(Token::Comma) {
            arguments.push(self.expression()?);
        }
        self.close()?;
        Ok(arguments)
    }

    /// A `)`, which may be left out at the end of the entry or before `►` or `→`.
    fn close(&mut self) -> Result<(), CalcError> {
        match self.peek() {
            Some(Token::RightParen) => self.pos += 1,
            None | Some(Token::Store | Token::Display(_)) => {}
            Some(_) => return Err(self.error(ErrorKind::Syntax)),
        }
        Ok(())
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

    /// The text of the tokens `accept` takes, from here on.
    fn collect(&mut self, accept: impl Fn(Token) -> bool) -> String {
        let mut text = String::new();
        while let Some(token) = self.peek().filter(|&t| accept(t)) {
            text.push_str(token.text());
            self.pos += 1;
        }
        text
    }
}

/// The next tighter level of infix operators below `level`.
fn tighter(level: Precedence) -> Precedence {
    match level {
        Precedence::Either => Precedence::Both,
        Precedence::Both => Precedence::Relation,
        Precedence::Relation => Precedence::Sum,
        Precedence::Sum => Precedence::Product,
        _ => Precedence::Negation,
    }
}

fn binary(op: Operator, (left, right): (Expr, Expr), at: usize) -> Expr {
    Expr::Binary(op, Box::new(left), Box::new(right), at)
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
        || matches!(
            token,
            Token::Negate
                | Token::Func(_)
                | Token::YVar(_)
                | Token::List(_)
                | Token::LeftBrace
                | Token::LeftParen
        )
        || atom(token).is_some()
}

/// A one-token operand.
fn atom(token: Token) -> Option<Expr> {
    Some(match token {
        Token::Pi => Expr::Constant(Constant::Pi),
        Token::EulerE => Expr::Constant(Constant::E),
        Token::Letter(letter) => Expr::Variable(letter),
        Token::Ans => Expr::Ans,
        Token::Rand => Expr::Rand,
        Token::Window(field) => Expr::Window(field),
        Token::Table(field) => Expr::Table(field),
        _ => return None,
    })
}

/// What a token that cannot start an operand means here: calculator syntax
/// not evaluated yet, or a syntax error.
fn unexpected(token: Token) -> ErrorKind {
    match token {
        Token::ImaginaryI
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
    use crate::{
        token::{Digit, Letter, ListName},
        vocabulary::{Function, PostfixOp},
    };

    /// Tokens for a compact spelling: digits, `.`, `+-*/^`, `_` negate,
    /// `()`, `,`, `E` exponent, `p` π, `²`, `³`, `!` inverse, `f` factorial,
    /// `→` store, `=<>` relations, `&` and, `|` or, `A`-`D`, `K` and `X`
    /// letters, `a` Ans, `s` sin(, `r` √(, `m` max(, `R` round(, `F` ►Frac,
    /// `L` L₁, `N` L₂, `{}` braces, `u` sum(, `q` seq(, `d` nDeriv(, `i` fnInt(,
    /// `n` fMin(, `x` fMax(, `S` Σ(, `y` Y₁, `z` Y₂, `w` Xmin, `t` ΔTbl.
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
        operator_token(c)
            .or_else(|| symbol_token(c))
            .unwrap_or_else(|| panic!("no token spelled {c:?}"))
    }

    fn operator_token(c: char) -> Option<Token> {
        let op = match c {
            '+' => Operator::Add,
            '-' => Operator::Subtract,
            '*' => Operator::Multiply,
            '/' => Operator::Divide,
            '^' => Operator::Power,
            '=' => Operator::Equal,
            '<' => Operator::Less,
            '>' => Operator::Greater,
            '&' => Operator::And,
            '|' => Operator::Or,
            _ => return None,
        };
        Some(Token::Op(op))
    }

    fn symbol_token(c: char) -> Option<Token> {
        Some(match c {
            '.' => Token::DecimalPoint,
            '_' => Token::Negate,
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            ',' => Token::Comma,
            'E' => Token::Exponent,
            'p' => Token::Pi,
            '²' => Token::Postfix(PostfixOp::Square),
            '³' => Token::Postfix(PostfixOp::Cube),
            '!' => Token::Postfix(PostfixOp::Inverse),
            'f' => Token::Postfix(PostfixOp::Factorial),
            '→' => Token::Store,
            'A' => Token::Letter(Letter::A),
            'B' => Token::Letter(Letter::B),
            'C' => Token::Letter(Letter::C),
            'D' => Token::Letter(Letter::D),
            'K' => Token::Letter(Letter::K),
            'X' => Token::Letter(Letter::X),
            'a' => Token::Ans,
            's' => Token::Func(Function::Sin),
            'r' => Token::Func(Function::Sqrt),
            'm' => Token::Func(Function::Max),
            'R' => Token::Func(Function::Round),
            'd' => Token::Func(Function::NDeriv),
            'i' => Token::Func(Function::FnInt),
            'n' => Token::Func(Function::FMin),
            'x' => Token::Func(Function::FMax),
            'S' => Token::Func(Function::Summation),
            'F' => Token::Display(DisplayAs::Fraction),
            'L' => Token::List(ListName::L1),
            'N' => Token::List(ListName::L2),
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            'u' => Token::Func(Function::Sum),
            'q' => Token::Func(Function::Seq),
            'y' => Token::YVar(0),
            'z' => Token::YVar(1),
            'w' => Token::Window(crate::window_editor::WindowField::XMin),
            't' => Token::Table(crate::table::TableField::Step),
            '[' => Token::LeftBracket,
            _ => return None,
        })
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
            ("4→", 2),
            ("4→5", 2),
            ("m1,", 3),
            ("2F3", 2),
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
        assert_eq!(error_of("2+[").kind, ErrorKind::Unsupported);
    }

    #[test]
    fn a_trailing_store_names_the_variable_after_any_display_conversion() {
        let statement = parse(&tokens("5F→B")).unwrap();
        assert_eq!(statement.store, Some(StoreTarget::Variable(Letter::B)));
        assert_eq!(statement.display, Some(DisplayAs::Fraction));
    }

    #[test]
    fn functions_take_comma_separated_arguments() {
        let statement = parse(&tokens("m1,2+3)")).unwrap();
        let Expr::Call(Function::Max, arguments, _) = statement.expr else {
            panic!("expected a call");
        };
        assert_eq!(arguments.len(), 2);
    }
}
