//! Matrices: sums, products and powers, transposes, inverses, determinants
//! and row reduction, as MATRIX MATH and the operators apply them.

use super::{arithmetic::truth, fail, value::Value, whole};
use crate::{
    error::{CalcError, ErrorKind},
    vocabulary::{Function, Operator, PostfixOp},
};

type Result<T, E = CalcError> = std::result::Result<T, E>;

/// A matrix by rows, every row the same length.
pub type Matrix = Vec<Vec<f64>>;

/// Rows and columns a matrix has at most.
pub const LARGEST_DIMENSION: usize = 99;

/// Pivots smaller than this in magnitude count as zero.
const PIVOT_TOLERANCE: f64 = 1e-12;

/// An operator with a matrix on at least one side: matrices add, subtract
/// and compare element by element, multiply as matrices, scale by a real
/// number on either side, and raise to whole powers.
pub(super) fn binary(op: Operator, left: Value, right: Value) -> Result<Value> {
    match (op, left, right) {
        (Operator::Add | Operator::Subtract, Value::Matrix(a), Value::Matrix(b)) => {
            elementwise(op, &a, &b).map(Value::Matrix)
        }
        (Operator::Multiply, Value::Matrix(a), Value::Matrix(b)) => {
            product(&a, &b).map(Value::Matrix)
        }
        (Operator::Multiply, Value::Real(k), Value::Matrix(m))
        | (Operator::Multiply, Value::Matrix(m), Value::Real(k)) => {
            Ok(Value::Matrix(scaled(&m, k)))
        }
        (Operator::Power, Value::Matrix(m), Value::Real(n)) => power(&m, n).map(Value::Matrix),
        (Operator::Equal | Operator::NotEqual, Value::Matrix(a), Value::Matrix(b)) => {
            Ok(Value::Real(truth((a == b) == (op == Operator::Equal))))
        }
        _ => fail(ErrorKind::DataType),
    }
}

/// `²`, `³`, `⁻¹` and `ᵀ` of a matrix.
pub(super) fn postfix(op: PostfixOp, m: &Matrix) -> Result<Matrix> {
    match op {
        PostfixOp::Square => power(m, 2.0),
        PostfixOp::Cube => power(m, 3.0),
        PostfixOp::Inverse => inverse(m),
        PostfixOp::Transpose => Ok(transpose(m)),
        PostfixOp::Factorial => fail(ErrorKind::DataType),
    }
}

/// MATRIX MATH's functions and `dim(` of a matrix; `None` for the rest.
pub(super) fn function(function: Function, values: &[Value]) -> Option<Result<Value>> {
    let result = match (function, values) {
        (Function::Det, [Value::Matrix(m)]) => determinant(m).map(Value::Real),
        (Function::Rref, [Value::Matrix(m)]) => Ok(Value::Matrix(reduced(m))),
        (Function::Dim, [Value::Matrix(m)]) => {
            Ok(Value::List(vec![m.len() as f64, m[0].len() as f64]))
        }
        (Function::Identity, [Value::Real(n)]) => dimension(*n).map(|n| Value::Matrix(identity(n))),
        (Function::Det | Function::Rref | Function::Identity, _) => fail(ErrorKind::DataType),
        _ => return None,
    };
    Some(result)
}

/// A size from 1 to 99, else `ERR:INVALID DIM`.
pub fn dimension(n: f64) -> Result<usize> {
    whole(n, 1.0..=LARGEST_DIMENSION as f64)
        .map(|n| n as usize)
        .map_err(|_| CalcError::new(ErrorKind::InvalidDim))
}

fn elementwise(op: Operator, a: &Matrix, b: &Matrix) -> Result<Matrix> {
    if size(a) != size(b) {
        return fail(ErrorKind::DimMismatch);
    }
    let sign = if op == Operator::Add { 1.0 } else { -1.0 };
    let rows = a.iter().zip(b);
    Ok(rows
        .map(|(x, y)| x.iter().zip(y).map(|(x, y)| x + sign * y).collect())
        .collect())
}

fn size(m: &Matrix) -> (usize, usize) {
    (m.len(), m[0].len())
}

fn scaled(m: &Matrix, k: f64) -> Matrix {
    m.iter()
        .map(|row| row.iter().map(|x| k * x).collect())
        .collect()
}

fn product(a: &Matrix, b: &Matrix) -> Result<Matrix> {
    if a[0].len() != b.len() {
        return fail(ErrorKind::DimMismatch);
    }
    let columns = transpose(b);
    let dot = |row: &Vec<f64>, column: &Vec<f64>| row.iter().zip(column).map(|(x, y)| x * y).sum();
    Ok(a.iter()
        .map(|row| columns.iter().map(|column| dot(row, column)).collect())
        .collect())
}

fn transpose(m: &Matrix) -> Matrix {
    (0..m[0].len())
        .map(|column| m.iter().map(|row| row[column]).collect())
        .collect()
}

fn identity(n: usize) -> Matrix {
    (0..n)
        .map(|row| (0..n).map(|column| truth(row == column)).collect())
        .collect()
}

fn square(m: &Matrix) -> Result<usize> {
    match size(m) {
        (rows, columns) if rows == columns => Ok(rows),
        _ => fail(ErrorKind::InvalidDim),
    }
}

/// A square matrix to a whole power from 0 to 255.
fn power(m: &Matrix, n: f64) -> Result<Matrix> {
    let order = square(m)?;
    let n = whole(n, 0.0..=255.0)? as usize;
    (0..n).try_fold(identity(order), |result, _| product(&result, m))
}

/// The determinant, by elimination with partial pivoting.
fn determinant(m: &Matrix) -> Result<f64> {
    square(m)?;
    let mut rows = m.clone();
    let mut sign = 1.0;
    for column in 0..rows.len() {
        let Some(pivot) = pivot_row(&rows, (column, column)) else {
            return Ok(0.0);
        };
        if pivot != column {
            rows.swap(pivot, column);
            sign = -sign;
        }
        eliminate_below(&mut rows, column);
    }
    Ok(sign * (0..rows.len()).map(|i| rows[i][i]).product::<f64>())
}

fn eliminate_below(rows: &mut Matrix, column: usize) {
    let pivot = rows[column].clone();
    for row in rows.iter_mut().skip(column + 1) {
        let factor = row[column] / pivot[column];
        row.iter_mut()
            .zip(&pivot)
            .for_each(|(x, p)| *x -= factor * p);
    }
}

/// The row from `start` down with the largest entry in `column`, unless all
/// are zero.
fn pivot_row(rows: &Matrix, (start, column): (usize, usize)) -> Option<usize> {
    (start..rows.len())
        .max_by(|&i, &j| rows[i][column].abs().total_cmp(&rows[j][column].abs()))
        .filter(|&row| rows[row][column].abs() > PIVOT_TOLERANCE)
}

/// The inverse, from the reduced form of the matrix beside the identity.
fn inverse(m: &Matrix) -> Result<Matrix> {
    let order = square(m)?;
    let augmented: Matrix = m
        .iter()
        .zip(identity(order))
        .map(|(row, unit)| [row.clone(), unit].concat())
        .collect();
    let reduced = reduced(&augmented);
    if (0..order).any(|i| (reduced[i][i] - 1.0).abs() > PIVOT_TOLERANCE) {
        return fail(ErrorKind::SingularMatrix);
    }
    Ok(reduced
        .into_iter()
        .map(|row| row[order..].to_vec())
        .collect())
}

/// The reduced row-echelon form, by Gauss-Jordan elimination.
fn reduced(m: &Matrix) -> Matrix {
    let mut rows = m.clone();
    let mut lead = 0;
    for column in 0..rows[0].len() {
        let Some(pivot) = pivot_row(&rows, (lead, column)) else {
            continue;
        };
        rows.swap(lead, pivot);
        normalize(&mut rows, (lead, column));
        lead += 1;
    }
    rows
}

/// Row `lead` scaled to a 1 in `column`, then subtracted from every other
/// row to clear the column.
fn normalize(rows: &mut Matrix, (lead, column): (usize, usize)) {
    let scale = rows[lead][column];
    rows[lead].iter_mut().for_each(|x| *x /= scale);
    let pivot = rows[lead].clone();
    for (_, row) in rows
        .iter_mut()
        .enumerate()
        .filter(|(index, _)| *index != lead)
    {
        let factor = row[column];
        row.iter_mut()
            .zip(&pivot)
            .for_each(|(x, p)| *x -= factor * p);
        row[column] = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matrix(rows: &[&[f64]]) -> Matrix {
        rows.iter().map(|row| row.to_vec()).collect()
    }

    #[test]
    fn products_and_powers_multiply_rows_by_columns() {
        let a = matrix(&[&[1.0, 2.0], &[3.0, 4.0]]);
        let squared = matrix(&[&[7.0, 10.0], &[15.0, 22.0]]);
        assert_eq!(postfix(PostfixOp::Square, &a), Ok(squared));
        let column = matrix(&[&[1.0], &[1.0]]);
        assert_eq!(product(&a, &column), Ok(matrix(&[&[3.0], &[7.0]])));
        assert_eq!(
            product(&column, &column).unwrap_err().kind,
            ErrorKind::DimMismatch
        );
        assert_eq!(power(&a, 0.0), Ok(identity(2)));
    }

    #[test]
    fn an_inverse_times_its_matrix_is_the_identity() {
        let a = matrix(&[&[4.0, 7.0], &[2.0, 6.0]]);
        assert_eq!(determinant(&a), Ok(10.0));
        let inverted = postfix(PostfixOp::Inverse, &a).unwrap();
        let rounded = scaled(&product(&inverted, &a).unwrap(), 1e12)
            .into_iter()
            .map(|row| row.into_iter().map(|x| x.round() / 1e12).collect())
            .collect::<Matrix>();
        assert_eq!(rounded, identity(2));
    }

    #[test]
    fn singular_and_non_square_matrices_have_no_inverse() {
        let singular = matrix(&[&[1.0, 2.0], &[2.0, 4.0]]);
        assert_eq!(determinant(&singular), Ok(0.0));
        assert_eq!(
            inverse(&singular).unwrap_err().kind,
            ErrorKind::SingularMatrix
        );
        let wide = matrix(&[&[1.0, 2.0, 3.0]]);
        assert_eq!(determinant(&wide).unwrap_err().kind, ErrorKind::InvalidDim);
    }

    #[test]
    fn rref_solves_a_system() {
        let system = matrix(&[&[1.0, 1.0, 3.0], &[1.0, -1.0, 1.0]]);
        assert_eq!(
            reduced(&system),
            matrix(&[&[1.0, 0.0, 2.0], &[0.0, 1.0, 1.0]])
        );
        let dependent = matrix(&[&[1.0, 2.0], &[2.0, 4.0]]);
        assert_eq!(reduced(&dependent), matrix(&[&[1.0, 2.0], &[0.0, 0.0]]));
    }
}
