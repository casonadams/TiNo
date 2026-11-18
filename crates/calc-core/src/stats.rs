//! STAT CALC: one-variable statistics and the linear regression y=ax+b.

use crate::{
    error::{CalcError, ErrorKind},
    eval::fail,
};

/// A statistic as the calculator lists it: its name and value (`x̄`, 2.5).
pub type Statistic = (&'static str, f64);

/// 1-Var Stats: mean, sums, sample and population deviations, size, and the
/// five-number summary, quartiles as the medians of the halves below and
/// above the median.
pub fn one_variable(list: &[f64]) -> Result<Vec<Statistic>, CalcError> {
    if list.is_empty() {
        return fail(ErrorKind::InvalidDim);
    }
    let n = list.len() as f64;
    let sum: f64 = list.iter().sum();
    let sum_squares: f64 = list.iter().map(|x| x * x).sum();
    let mean = sum / n;
    let deviations: f64 = list.iter().map(|x| (x - mean).powi(2)).sum();
    let mut sorted = list.to_vec();
    sorted.sort_by(f64::total_cmp);
    let half = sorted.len() / 2;
    let upper = &sorted[sorted.len() - half..];
    let mut statistics = vec![("x̄", mean), ("Σx", sum), ("Σx²", sum_squares)];
    if list.len() > 1 {
        statistics.push(("Sx", (deviations / (n - 1.0)).sqrt()));
    }
    statistics.extend([
        ("σx", (deviations / n).sqrt()),
        ("n", n),
        ("minX", sorted[0]),
        ("Q₁", median(&sorted[..half.max(1)])),
        ("Med", median(&sorted)),
        ("Q₃", median(if half == 0 { &sorted } else { upper })),
        ("maxX", sorted[sorted.len() - 1]),
    ]);
    Ok(statistics)
}

fn median(sorted: &[f64]) -> f64 {
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

/// LinReg(ax+b): the least-squares line through the points, with the
/// coefficient of determination and correlation.
pub fn linear_regression(xs: &[f64], ys: &[f64]) -> Result<Vec<Statistic>, CalcError> {
    if xs.len() != ys.len() {
        return fail(ErrorKind::DimMismatch);
    }
    if xs.len() < 2 {
        return fail(ErrorKind::InvalidDim);
    }
    let n = xs.len() as f64;
    let (mean_x, mean_y) = (xs.iter().sum::<f64>() / n, ys.iter().sum::<f64>() / n);
    let pairs = xs.iter().zip(ys);
    let sxy: f64 = pairs.map(|(x, y)| (x - mean_x) * (y - mean_y)).sum();
    let sxx: f64 = xs.iter().map(|x| (x - mean_x).powi(2)).sum();
    let syy: f64 = ys.iter().map(|y| (y - mean_y).powi(2)).sum();
    if sxx == 0.0 {
        return fail(ErrorKind::DivideByZero);
    }
    let a = sxy / sxx;
    let r = sxy / (sxx * syy).sqrt();
    Ok(vec![
        ("a", a),
        ("b", mean_y - a * mean_x),
        ("r²", r * r),
        ("r", r),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(statistics: &[Statistic], name: &str) -> f64 {
        statistics
            .iter()
            .find(|(label, _)| *label == name)
            .map(|(_, value)| *value)
            .unwrap()
    }

    #[test]
    fn one_variable_statistics_match_the_calculator() {
        let stats = one_variable(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]).unwrap();
        let rounded: Vec<(&str, f64)> = stats
            .iter()
            .map(|&(name, value)| (name, (value * 1e9).round() / 1e9))
            .collect();
        let expected = [
            ("x̄", 4.0),
            ("Σx", 28.0),
            ("Σx²", 140.0),
            ("Sx", 2.160246899),
            ("σx", 2.0),
            ("n", 7.0),
            ("minX", 1.0),
            ("Q₁", 2.0),
            ("Med", 4.0),
            ("Q₃", 6.0),
            ("maxX", 7.0),
        ];
        assert_eq!(rounded, expected);
    }

    #[test]
    fn quartiles_of_an_even_count_split_it_in_halves() {
        let even = one_variable(&[1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!((value(&even, "Q₁"), value(&even, "Q₃")), (1.5, 3.5));
    }

    #[test]
    fn a_line_through_points_fits_exactly() {
        let fit = linear_regression(&[1.0, 2.0, 3.0], &[3.0, 5.0, 7.0]).unwrap();
        assert_eq!(
            (value(&fit, "a"), value(&fit, "b"), value(&fit, "r")),
            (2.0, 1.0, 1.0)
        );
        let kind = linear_regression(&[1.0, 2.0], &[1.0]).unwrap_err().kind;
        assert_eq!(kind, ErrorKind::DimMismatch);
    }
}
