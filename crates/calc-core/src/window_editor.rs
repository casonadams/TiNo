use crate::{
    editor::Editor,
    error::{CalcError, ErrorKind},
    eval::{Memory, evaluate},
    format::format_real,
    parse::parse,
    settings::{Angle, Settings},
    token::{Digit, Token},
    window::Window,
};

/// A line of the WINDOW screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowField {
    XMin,
    XMax,
    XScale,
    YMin,
    YMax,
    YScale,
    XRes,
}

impl WindowField {
    pub const ALL: [WindowField; 7] = [
        WindowField::XMin,
        WindowField::XMax,
        WindowField::XScale,
        WindowField::YMin,
        WindowField::YMax,
        WindowField::YScale,
        WindowField::XRes,
    ];

    pub fn label(self) -> &'static str {
        match self {
            WindowField::XMin => "Xmin",
            WindowField::XMax => "Xmax",
            WindowField::XScale => "Xscl",
            WindowField::YMin => "Ymin",
            WindowField::YMax => "Ymax",
            WindowField::YScale => "Yscl",
            WindowField::XRes => "Xres",
        }
    }

    pub fn get(self, window: &Window) -> f64 {
        match self {
            WindowField::XMin => window.x_min,
            WindowField::XMax => window.x_max,
            WindowField::XScale => window.x_scale,
            WindowField::YMin => window.y_min,
            WindowField::YMax => window.y_max,
            WindowField::YScale => window.y_scale,
            WindowField::XRes => window.x_res,
        }
    }

    pub fn set(self, window: &mut Window, value: f64) {
        let field = match self {
            WindowField::XMin => &mut window.x_min,
            WindowField::XMax => &mut window.x_max,
            WindowField::XScale => &mut window.x_scale,
            WindowField::YMin => &mut window.y_min,
            WindowField::YMax => &mut window.y_max,
            WindowField::YScale => &mut window.y_scale,
            WindowField::XRes => &mut window.x_res,
        };
        *field = value;
    }

    fn index(self) -> usize {
        WindowField::ALL
            .iter()
            .position(|&f| f == self)
            .unwrap_or(0)
    }
}

/// The WINDOW screen's cursor: the focused field and what is typed into it.
#[derive(Debug, Clone, PartialEq)]
pub struct WindowCursor {
    pub field: WindowField,
    pub editor: Editor,
}

impl WindowCursor {
    /// On `field`, its entry showing the field's value as the calculator writes it.
    pub fn new(field: WindowField, window: &Window, settings: Settings) -> Self {
        let mut editor = Editor::default();
        editor.replace(number_tokens(&format_real(field.get(window), settings)));
        editor.place_cursor(0);
        WindowCursor { field, editor }
    }

    /// The field above or below, stopping at the first and last.
    pub fn neighbor(&self, down: bool) -> WindowField {
        let index = self.field.index();
        let next = if down {
            (index + 1).min(WindowField::ALL.len() - 1)
        } else {
            index.saturating_sub(1)
        };
        WindowField::ALL[next]
    }

    /// The entry's value, angles in `angle` units; `None` for an empty
    /// entry, which keeps the field's value.
    pub fn value(&self, memory: &Memory, angle: Angle) -> Result<Option<f64>, CalcError> {
        let tokens = self.editor.tokens();
        if tokens.is_empty() {
            return Ok(None);
        }
        let statement = parse(tokens)?;
        if statement.store.is_some() {
            return Err(CalcError::new(ErrorKind::Syntax));
        }
        evaluate(&statement.expr, memory, angle).map(Some)
    }
}

/// The tokens that type a formatted number (`⁻1.5ᴇ3`).
fn number_tokens(text: &str) -> Vec<Token> {
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
    text.chars()
        .filter_map(|c| match c {
            '.' => Some(Token::DecimalPoint),
            '⁻' => Some(Token::Negate),
            'ᴇ' => Some(Token::Exponent),
            c => c.to_digit(10).map(|d| Token::Digit(DIGITS[d as usize])),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_field_opens_with_its_value_typed_out() {
        let cursor = WindowCursor::new(WindowField::XMin, &Window::standard(), Settings::default());
        let text: String = cursor.editor.tokens().iter().map(|t| t.text()).collect();
        assert_eq!(text, "⁻10");
    }

    #[test]
    fn the_entry_evaluates_as_an_expression() {
        let mut cursor =
            WindowCursor::new(WindowField::XMax, &Window::standard(), Settings::default());
        cursor
            .editor
            .replace(vec![Token::Pi, Token::Divide, Token::Digit(Digit::Two)]);
        let value = cursor
            .value(&Memory::default(), Angle::Radian)
            .unwrap()
            .unwrap();
        assert!((value - std::f64::consts::FRAC_PI_2).abs() < 1e-15);
        cursor.editor.replace(Vec::new());
        assert_eq!(cursor.value(&Memory::default(), Angle::Radian), Ok(None));
    }

    #[test]
    fn number_text_round_trips_through_tokens() {
        let text: String = number_tokens("⁻1.5ᴇ⁻3").iter().map(|t| t.text()).collect();
        assert_eq!(text, "⁻1.5ᴇ⁻3");
    }

    #[test]
    fn fields_stop_at_the_ends() {
        let cursor = WindowCursor::new(WindowField::XRes, &Window::standard(), Settings::default());
        assert_eq!(cursor.neighbor(true), WindowField::XRes);
        assert_eq!(cursor.neighbor(false), WindowField::YScale);
    }
}
