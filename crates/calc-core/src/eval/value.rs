use serde::{Deserialize, Serialize};

use super::fail;
use crate::error::{CalcError, ErrorKind};

/// What an expression evaluates to: a real number or a list of them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Real(f64),
    List(Vec<f64>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Real(0.0)
    }
}

impl Value {
    /// The real number, or `ERR:DATA TYPE` for a list.
    pub fn real(self) -> Result<f64, CalcError> {
        match self {
            Value::Real(x) => Ok(x),
            Value::List(_) => fail(ErrorKind::DataType),
        }
    }

    /// The list, or `ERR:DATA TYPE` for a real number.
    pub fn list(self) -> Result<Vec<f64>, CalcError> {
        match self {
            Value::List(list) => Ok(list),
            Value::Real(_) => fail(ErrorKind::DataType),
        }
    }

    /// `f` of a real number, or of each element of a list.
    pub fn map(self, f: impl Fn(f64) -> Result<f64, CalcError>) -> Result<Value, CalcError> {
        match self {
            Value::Real(x) => f(x).map(Value::Real),
            Value::List(list) => list
                .into_iter()
                .map(f)
                .collect::<Result<_, _>>()
                .map(Value::List),
        }
    }

    /// `f` element by element: a real number pairs with each element of a
    /// list; lists must have the same length.
    pub fn zip(
        self,
        other: Value,
        f: impl Fn(f64, f64) -> Result<f64, CalcError>,
    ) -> Result<Value, CalcError> {
        broadcast(&[self, other], |values| f(values[0], values[1]))
    }
}

/// `f` of the values' reals, or element by element when any is a list (a
/// real number pairs with each element); `ERR:DIM MISMATCH` for lists of
/// different lengths.
pub fn broadcast(
    values: &[Value],
    f: impl Fn(&[f64]) -> Result<f64, CalcError>,
) -> Result<Value, CalcError> {
    let mut lengths = values.iter().filter_map(|value| match value {
        Value::List(list) => Some(list.len()),
        Value::Real(_) => None,
    });
    let Some(length) = lengths.next() else {
        let reals: Vec<f64> = values
            .iter()
            .map(|value| value.clone().real())
            .collect::<Result<_, _>>()?;
        return f(&reals).map(Value::Real);
    };
    if lengths.any(|other| other != length) {
        return fail(ErrorKind::DimMismatch);
    }
    let element = |index: usize| -> Vec<f64> {
        values
            .iter()
            .map(|value| match value {
                Value::Real(x) => *x,
                Value::List(list) => list[index],
            })
            .collect()
    };
    (0..length)
        .map(|index| f(&element(index)))
        .collect::<Result<_, _>>()
        .map(Value::List)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reals_pair_with_each_element_and_lists_pair_up() {
        let add = |a: f64, b: f64| Ok(a + b);
        let list = Value::List(vec![1.0, 2.0]);
        assert_eq!(
            list.clone().zip(Value::Real(10.0), add),
            Ok(Value::List(vec![11.0, 12.0]))
        );
        assert_eq!(
            list.clone().zip(list.clone(), add),
            Ok(Value::List(vec![2.0, 4.0]))
        );
        let longer = Value::List(vec![1.0, 2.0, 3.0]);
        assert_eq!(
            list.zip(longer, add).unwrap_err().kind,
            ErrorKind::DimMismatch
        );
        assert_eq!(
            Value::Real(1.0).zip(Value::Real(2.0), add),
            Ok(Value::Real(3.0))
        );
    }

    #[test]
    fn kept_values_read_back_from_plain_json() {
        let values: Vec<Value> = serde_json::from_str("[2.5,[1,2]]").unwrap();
        assert_eq!(values, [Value::Real(2.5), Value::List(vec![1.0, 2.0])]);
    }
}
