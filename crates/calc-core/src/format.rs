/// Significant digits a result shows.
const DIGITS: usize = 10;

/// A result as the calculator shows it in Float mode: up to 10 significant
/// digits, no leading zero (`.5`), `⁻` for negatives, and scientific
/// notation (`1.5ᴇ12`, `2ᴇ⁻4`) below .001 or from 10 digits before the point.
pub fn format_real(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    let scientific = format!("{:.*e}", DIGITS - 1, value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let body = if (-3..DIGITS as i32).contains(&exponent) {
        positional(&digits, exponent)
    } else {
        exponential(&digits, exponent)
    };
    if value < 0.0 {
        format!("⁻{body}")
    } else {
        body
    }
}

fn positional(digits: &str, exponent: i32) -> String {
    let (whole, fraction) = match usize::try_from(exponent) {
        Ok(point) => (digits[..=point].to_owned(), digits[point + 1..].to_owned()),
        Err(_) => (
            String::new(),
            "0".repeat(exponent.unsigned_abs() as usize - 1) + digits,
        ),
    };
    join(&whole, fraction.trim_end_matches('0'))
}

fn exponential(digits: &str, exponent: i32) -> String {
    let mantissa = join(&digits[..1], digits[1..].trim_end_matches('0'));
    let sign = if exponent < 0 { "⁻" } else { "" };
    format!("{mantissa}ᴇ{sign}{}", exponent.unsigned_abs())
}

fn join(whole: &str, fraction: &str) -> String {
    if fraction.is_empty() {
        whole.to_owned()
    } else {
        format!("{whole}.{fraction}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_float_mode() {
        let cases = [
            (0.0, "0"),
            (14.0, "14"),
            (-4.0, "⁻4"),
            (0.5, ".5"),
            (-0.25, "⁻.25"),
            (0.1 + 0.2, ".3"),
            (1.0 / 3.0, ".3333333333"),
            (2.0 / 3.0, ".6666666667"),
            (std::f64::consts::PI, "3.141592654"),
            (0.001, ".001"),
            (0.0001, "1ᴇ⁻4"),
            (1234567890.0, "1234567890"),
            (12345678901.0, "1.23456789ᴇ10"),
            (9999999999.6, "1ᴇ10"),
            (-2.5e-12, "⁻2.5ᴇ⁻12"),
            (6.02e23, "6.02ᴇ23"),
        ];
        for (value, text) in cases {
            assert_eq!(format_real(value), text, "{value}");
        }
    }
}
