use super::{fail, value::Value};
use crate::{
    error::{CalcError, ErrorKind},
    vocabulary::Function,
};

type Result<T, E = CalcError> = std::result::Result<T, E>;

/// The LIST functions, and `min(` or `max(` of one list; `None` for every
/// other function, which applies element by element.
pub(super) fn list_function(function: Function, values: &[Value]) -> Option<Result<Value>> {
    let result = match (function, values) {
        (Function::Min | Function::Max, [one]) => one
            .clone()
            .list()
            .map(|list| Value::Real(extreme(function, &list))),
        (Function::Augment, [first, second]) => augment(first, second),
        (Function::Dim, [one]) => one
            .clone()
            .list()
            .map(|list| Value::Real(list.len() as f64)),
        (Function::CumSum | Function::DeltaList, [one]) => {
            one.clone().list().and_then(|list| running(function, &list))
        }
        (_, [one]) if is_statistic(function) => one
            .clone()
            .list()
            .and_then(|list| statistic(function, &list)),
        _ => return None,
    };
    Some(result)
}

fn is_statistic(function: Function) -> bool {
    matches!(
        function,
        Function::Sum
            | Function::Prod
            | Function::Mean
            | Function::Median
            | Function::StdDev
            | Function::Variance
    )
}

fn extreme(function: Function, list: &[f64]) -> f64 {
    let fold = if function == Function::Min {
        f64::min
    } else {
        f64::max
    };
    list.iter().copied().reduce(fold).unwrap_or(f64::NAN)
}

fn augment(first: &Value, second: &Value) -> Result<Value> {
    let mut joined = first.clone().list()?;
    joined.extend(second.clone().list()?);
    Ok(Value::List(joined))
}

/// `cumSum(`: each element's running total; `ΔList(`: the differences of
/// neighbors, one fewer, needing two elements.
fn running(function: Function, list: &[f64]) -> Result<Value> {
    if function == Function::CumSum {
        let totals = list.iter().scan(0.0, |total, &x| {
            *total += x;
            Some(*total)
        });
        return Ok(Value::List(totals.collect()));
    }
    if list.len() < 2 {
        return fail(ErrorKind::InvalidDim);
    }
    Ok(Value::List(
        list.windows(2).map(|pair| pair[1] - pair[0]).collect(),
    ))
}

/// `sum(`, `prod(`, `mean(`, `median(`, and the sample `stdDev(` and
/// `variance(`, which need two elements.
fn statistic(function: Function, list: &[f64]) -> Result<Value> {
    let n = list.len() as f64;
    let mean = list.iter().sum::<f64>() / n;
    let variance = || {
        if list.len() < 2 {
            return fail(ErrorKind::Domain);
        }
        Ok(list.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0))
    };
    let value = match function {
        Function::Sum => list.iter().sum(),
        Function::Prod => list.iter().product(),
        Function::Mean => mean,
        Function::Median => median(list),
        Function::StdDev => variance()?.sqrt(),
        _ => variance()?,
    };
    Ok(Value::Real(value))
}

fn median(list: &[f64]) -> f64 {
    let mut sorted = list.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len().is_multiple_of(2) {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn applied(function: Function, values: &[Value]) -> Result<Value> {
        list_function(function, values).expect("a list function")
    }

    #[test]
    fn statistics_of_a_list() {
        let list = [Value::List(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0])];
        let cases = [
            (Function::Sum, 40.0),
            (Function::Mean, 5.0),
            (Function::Median, 4.5),
            (Function::Min, 2.0),
            (Function::Max, 9.0),
            (Function::Dim, 8.0),
            (Function::Variance, 32.0 / 7.0),
        ];
        for (function, expected) in cases {
            assert_eq!(
                applied(function, &list),
                Ok(Value::Real(expected)),
                "{function:?}"
            );
        }
    }

    #[test]
    fn running_sums_differences_and_joins() {
        let list = Value::List(vec![1.0, 3.0, 6.0]);
        assert_eq!(
            applied(Function::CumSum, std::slice::from_ref(&list)),
            Ok(Value::List(vec![1.0, 4.0, 10.0]))
        );
        assert_eq!(
            applied(Function::DeltaList, std::slice::from_ref(&list)),
            Ok(Value::List(vec![2.0, 3.0]))
        );
        let joined = applied(Function::Augment, &[list, Value::List(vec![7.0])]);
        assert_eq!(joined, Ok(Value::List(vec![1.0, 3.0, 6.0, 7.0])));
    }

    #[test]
    fn numbers_where_lists_belong_and_short_lists_fail() {
        let kind = |function, values: &[Value]| applied(function, values).unwrap_err().kind;
        assert_eq!(
            kind(Function::Sum, &[Value::Real(3.0)]),
            ErrorKind::DataType
        );
        assert_eq!(
            kind(Function::StdDev, &[Value::List(vec![1.0])]),
            ErrorKind::Domain
        );
        assert_eq!(
            kind(Function::DeltaList, &[Value::List(vec![1.0])]),
            ErrorKind::InvalidDim
        );
        assert_eq!(list_function(Function::Sqrt, &[Value::Real(4.0)]), None);
    }
}
