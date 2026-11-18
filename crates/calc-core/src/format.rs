use crate::{
    eval::{Complex, Value},
    settings::{Decimals, Notation, NumberMode, Settings},
    token::{Digit, Token},
    vocabulary::{DisplayAs, Operator},
};

/// Significant digits a result shows at most.
const DIGITS: i32 = 10;

/// The tokens that type a formatted result (`⁻1.5ᴇ3`, `⁻3/4`, `{1 2}`,
/// a matrix's rows); the padding that aligns a matrix's columns types nothing.
pub fn result_tokens(text: &str) -> Vec<Token> {
    text.chars()
        .filter_map(|c| match c {
            '.' => Some(Token::DecimalPoint),
            '⁻' => Some(Token::Negate),
            'ᴇ' => Some(Token::Exponent),
            '/' => Some(Token::Op(Operator::Divide)),
            '+' => Some(Token::Op(Operator::Add)),
            '−' => Some(Token::Op(Operator::Subtract)),
            '^' => Some(Token::Op(Operator::Power)),
            '(' => Some(Token::LeftParen),
            ')' => Some(Token::RightParen),
            'e' => Some(Token::EulerE),
            'i' => Some(Token::ImaginaryI),
            '{' => Some(Token::LeftBrace),
            '}' => Some(Token::RightBrace),
            '[' => Some(Token::LeftBracket),
            ']' => Some(Token::RightBracket),
            ' ' => Some(Token::Comma),
            c => c.to_digit(10).map(|d| Token::Digit(Digit::ALL[d as usize])),
        })
        .collect()
}

/// Pads a matrix's elements to their column's width: as wide as a digit,
/// and no comma when the result is typed back.
const FIGURE_SPACE: char = '\u{2007}';

/// A result as the home screen shows it: a number as `format_real` (or
/// `format_fraction` after `►Frac`) writes it, a complex number as `a+bi`
/// or `re^(θi)` by mode or `►Rect`/`►Polar`, a list as its elements between
/// braces (`{1 2.5 ⁻3}`), a matrix one line per row with its columns
/// aligned (`[[1 2]` over ` [3 4]]`).
pub fn format_value(value: &Value, settings: Settings, display: Option<DisplayAs>) -> String {
    let number = |x: f64| match display {
        Some(DisplayAs::Fraction) => format_fraction(x, settings),
        _ => format_real(x, settings),
    };
    match value {
        Value::Real(x) => number(*x),
        Value::Complex(z) => {
            let polar = match display {
                Some(DisplayAs::Polar) => true,
                Some(DisplayAs::Rectangular) => false,
                _ => settings.number == NumberMode::Polar,
            };
            format_complex(*z, polar, &number)
        }
        Value::List(list) => {
            let elements: Vec<String> = list.iter().map(|&x| number(x)).collect();
            format!("{{{}}}", elements.join(" "))
        }
        Value::Matrix(rows) => {
            let cells: Vec<Vec<String>> = rows
                .iter()
                .map(|row| row.iter().map(|&x| number(x)).collect())
                .collect();
            format_matrix(&cells)
        }
    }
}

/// `a+bi` (`3−4i`, `⁻2i`, `i`), or `re^(θi)` with θ in radians.
fn format_complex(z: Complex, polar: bool, number: &impl Fn(f64) -> String) -> String {
    if polar {
        return format!("{}e^({}i)", number(z.abs()), number(z.arg()));
    }
    let coefficient = |im: f64| match im.abs() {
        1.0 => String::new(),
        magnitude => number(magnitude),
    };
    let sign = if z.im < 0.0 { "−" } else { "+" };
    match z.re {
        0.0 if z.im < 0.0 => format!("⁻{}i", coefficient(z.im)),
        0.0 => format!("{}i", coefficient(z.im)),
        re => format!("{}{sign}{}i", number(re), coefficient(z.im)),
    }
}

fn format_matrix(cells: &[Vec<String>]) -> String {
    let width = |column: usize| {
        let widths = cells.iter().map(|row| row[column].chars().count());
        widths.max().unwrap_or(0)
    };
    let widths: Vec<usize> = (0..cells[0].len()).map(width).collect();
    let row_text = |row: &Vec<String>| {
        let padded = row.iter().zip(&widths).map(|(cell, &width)| {
            let padding = width - cell.chars().count();
            format!("{}{cell}", FIGURE_SPACE.to_string().repeat(padding))
        });
        format!("[{}]", padded.collect::<Vec<_>>().join(" "))
    };
    let rows: Vec<String> = cells.iter().map(row_text).collect();
    format!("[{}]", rows.join(&format!("\n{FIGURE_SPACE}")))
}

/// A result as the calculator shows it under `settings`: at most 10
/// significant digits, no leading zero (`.5`), `⁻` for negatives. Normal
/// notation turns scientific below .001 and from 10 digits before the point.
pub fn format_real(value: f64, settings: Settings) -> String {
    let body = magnitude(value.abs(), settings);
    if value < 0.0 {
        format!("⁻{body}")
    } else {
        body
    }
}

/// Denominators `►Frac` tries, up to this one.
const LARGEST_DENOMINATOR: f64 = 9999.0;

/// `►Frac`: the value as a fraction `n/d` (`⁻1/3`) when one with a
/// denominator up to 9999 matches it to the digits shown; otherwise as
/// `format_real` writes it.
pub fn format_fraction(value: f64, settings: Settings) -> String {
    match fraction(value.abs()) {
        Some((numerator, denominator)) if denominator > 1.0 => {
            let sign = if value < 0.0 { "⁻" } else { "" };
            format!("{sign}{numerator}/{denominator}")
        }
        _ => format_real(value, settings),
    }
}

/// The simplest fraction matching `x` to 10 significant digits, from its
/// continued fraction.
fn fraction(x: f64) -> Option<(f64, f64)> {
    let tolerance = 5e-11 * x.max(1.0);
    let (mut previous, mut current) = ((0.0, 1.0), (1.0, 0.0));
    let mut rest = x;
    for _ in 0..32 {
        let whole = rest.floor();
        let next = (
            whole * current.0 + previous.0,
            whole * current.1 + previous.1,
        );
        if next.1 > LARGEST_DENOMINATOR {
            return None;
        }
        if (next.0 / next.1 - x).abs() <= tolerance {
            return Some(next);
        }
        (previous, current) = (current, next);
        rest = 1.0 / (rest - whole);
    }
    None
}

fn magnitude(abs: f64, settings: Settings) -> String {
    if abs == 0.0 {
        return zero(settings.decimals);
    }
    let exponent = decimal_exponent(abs);
    match settings.notation {
        Notation::Normal if (-3..DIGITS).contains(&exponent) => {
            positional(abs, exponent, settings.decimals)
        }
        Notation::Normal | Notation::Scientific => {
            scaled(abs, Scale { exponent, step: 1 }, settings.decimals)
        }
        Notation::Engineering => {
            let exponent = exponent.div_euclid(3) * 3;
            scaled(abs, Scale { exponent, step: 3 }, settings.decimals)
        }
    }
}

fn zero(decimals: Decimals) -> String {
    match decimals {
        Decimals::Float => "0".to_owned(),
        Decimals::Fix(places) => without_leading_zero(format!("{:.*}", places.get(), 0.0)),
    }
}

/// The power of ten of `abs`'s leading digit once rounded to 10 digits.
fn decimal_exponent(abs: f64) -> i32 {
    let scientific = format!("{:.*e}", DIGITS as usize - 1, abs);
    scientific
        .split_once('e')
        .and_then(|(_, exponent)| exponent.parse().ok())
        .unwrap_or(0)
}

fn positional(abs: f64, exponent: i32, decimals: Decimals) -> String {
    let fitting = usize::try_from(DIGITS - 1 - exponent).unwrap_or(0);
    let text = match decimals {
        Decimals::Float => trim_fraction(format!("{abs:.fitting$}")),
        Decimals::Fix(places) => fixed(abs, places.get().min(fitting)),
    };
    without_leading_zero(text)
}

/// A power of ten for a mantissa, kept to multiples of `step` (3 in ENG).
#[derive(Clone, Copy)]
struct Scale {
    exponent: i32,
    step: i32,
}

/// `abs` as a mantissa times `10^exponent`, rounding up into the next
/// allowed exponent when the mantissa rounds to `10^step`.
fn scaled(abs: f64, scale: Scale, decimals: Decimals) -> String {
    let Scale { exponent, step } = scale;
    let text = mantissa(abs / 10f64.powi(exponent), decimals);
    if text.parse::<f64>().is_ok_and(|m| m >= 10f64.powi(step)) {
        let next = Scale {
            exponent: exponent + step,
            step,
        };
        return scaled(abs, next, decimals);
    }
    let sign = if exponent < 0 { "⁻" } else { "" };
    format!("{text}ᴇ{sign}{}", exponent.unsigned_abs())
}

fn mantissa(value: f64, decimals: Decimals) -> String {
    match decimals {
        Decimals::Float => {
            let whole_digits = value.log10().floor() as i32 + 1;
            let places = usize::try_from(DIGITS - whole_digits).unwrap_or(0);
            trim_fraction(format!("{value:.places$}"))
        }
        Decimals::Fix(places) => fixed(value, places.get()),
    }
}

/// `value` with exactly `places` decimals, ties rounded away from zero
/// (`2.5` → `3`), as the calculator rounds.
fn fixed(value: f64, places: usize) -> String {
    let scale = 10f64.powi(places as i32);
    format!("{:.places$}", (value * scale).round() / scale)
}

fn trim_fraction(text: String) -> String {
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    } else {
        text
    }
}

fn without_leading_zero(text: String) -> String {
    match text.strip_prefix("0.") {
        Some(fraction) => format!(".{fraction}"),
        None => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::FixedPlaces;
    use std::f64::consts::PI;

    fn settings(notation: Notation, decimals: Decimals) -> Settings {
        Settings {
            notation,
            decimals,
            ..Settings::default()
        }
    }

    fn fix(places: u8) -> Decimals {
        Decimals::Fix(FixedPlaces::new(places).unwrap())
    }

    #[test]
    fn normal_float_shows_up_to_ten_digits() {
        let cases = [
            (0.0, "0"),
            (14.0, "14"),
            (-4.0, "⁻4"),
            (0.5, ".5"),
            (-0.25, "⁻.25"),
            (0.1 + 0.2, ".3"),
            (1.0 / 3.0, ".3333333333"),
            (2.0 / 3.0, ".6666666667"),
            (PI, "3.141592654"),
            (0.001, ".001"),
            (0.0001, "1ᴇ⁻4"),
            (1234567890.0, "1234567890"),
            (12345678901.0, "1.23456789ᴇ10"),
            (9999999999.6, "1ᴇ10"),
            (-2.5e-12, "⁻2.5ᴇ⁻12"),
            (6.02e23, "6.02ᴇ23"),
        ];
        for (value, text) in cases {
            assert_eq!(format_real(value, Settings::default()), text, "{value}");
        }
    }

    #[test]
    fn each_mode_formats_as_the_calculator_does() {
        let cases = [
            (settings(Notation::Normal, fix(2)), PI, "3.14"),
            (settings(Notation::Normal, fix(2)), 5.0, "5.00"),
            (settings(Notation::Normal, fix(2)), 0.5, ".50"),
            (settings(Notation::Normal, fix(0)), 2.5, "3"),
            (settings(Notation::Normal, fix(2)), 0.125, ".13"),
            (settings(Notation::Normal, fix(2)), 0.0, ".00"),
            (
                settings(Notation::Normal, fix(3)),
                123456789.123,
                "123456789.1",
            ),
            (settings(Notation::Normal, fix(2)), 0.0001, "1.00ᴇ⁻4"),
            (
                settings(Notation::Scientific, Decimals::Float),
                123.0,
                "1.23ᴇ2",
            ),
            (settings(Notation::Scientific, Decimals::Float), 0.5, "5ᴇ⁻1"),
            (settings(Notation::Scientific, fix(2)), 9.999, "1.00ᴇ1"),
            (
                settings(Notation::Engineering, Decimals::Float),
                12345.0,
                "12.345ᴇ3",
            ),
            (
                settings(Notation::Engineering, Decimals::Float),
                0.00012,
                "120ᴇ⁻6",
            ),
            (settings(Notation::Engineering, fix(1)), 999.96, "1.0ᴇ3"),
        ];
        for (settings, value, text) in cases {
            assert_eq!(format_real(value, settings), text, "{settings:?} {value}");
        }
    }

    #[test]
    fn frac_writes_simple_fractions_and_leaves_others_decimal() {
        let cases = [
            (1.0 / 3.0, "1/3"),
            (-0.75, "⁻3/4"),
            (2.5, "5/2"),
            (4.0, "4"),
            (PI, "3.141592654"),
            (1.0 / 9999.0, "1/9999"),
            (1.0 / 10007.0, "9.993004897ᴇ⁻5"),
        ];
        for (value, text) in cases {
            assert_eq!(format_fraction(value, Settings::default()), text, "{value}");
        }
    }
}
