use crate::{
    error::{CalcError, ErrorKind},
    eval::fail,
    settings::Angle,
    vocabulary::Function,
};

/// `sin`, `cos` or `tan` of `x` in `angle` units. Multiples of 90° (π/2 in
/// radians, to within rounding of π) are exact (`sin(π)` is 0) and `tan` of
/// an odd multiple is a domain error, as on the calculator.
pub fn forward(function: Function, x: f64, angle: Angle) -> Result<f64, CalcError> {
    let (quadrant, radians) = match angle {
        Angle::Radian => (radian_quadrant(x), x),
        Angle::Degree => (right_angle_quadrant(x), x.to_radians()),
    };
    if let Some(quadrant) = quadrant {
        return exact(function, quadrant);
    }
    Ok(match function {
        Function::Sin => radians.sin(),
        Function::Cos => radians.cos(),
        _ => radians.tan(),
    })
}

/// `sin⁻¹`, `cos⁻¹` or `tan⁻¹` of `x`, the angle in `angle` units.
pub fn inverse(function: Function, x: f64, angle: Angle) -> Result<f64, CalcError> {
    let radians = match function {
        Function::ArcTan => x.atan(),
        _ if x.abs() > 1.0 => return fail(ErrorKind::Domain),
        Function::ArcSin => x.asin(),
        _ => x.acos(),
    };
    Ok(match angle {
        Angle::Radian => radians,
        Angle::Degree => radians.to_degrees(),
    })
}

/// Which multiple of 90° `degrees` is, modulo 4; `None` when it is none.
fn right_angle_quadrant(degrees: f64) -> Option<u8> {
    let turn = degrees.rem_euclid(360.0);
    (turn % 90.0 == 0.0).then(|| (turn / 90.0) as u8)
}

/// Which multiple of π/2 `radians` is, modulo 4, allowing the rounding a
/// nonzero multiple of π picks up (0 itself is exact, so `sin(1ᴇ⁻²⁰)` keeps
/// its value); `None` when it is none.
fn radian_quadrant(radians: f64) -> Option<u8> {
    let quarters = radians / std::f64::consts::FRAC_PI_2;
    let nearest = quarters.round();
    let close = (quarters - nearest).abs() <= 1e-13 * nearest.abs();
    close.then(|| nearest.rem_euclid(4.0) as u8)
}

fn exact(function: Function, quadrant: u8) -> Result<f64, CalcError> {
    const SIN: [f64; 4] = [0.0, 1.0, 0.0, -1.0];
    let quadrant = usize::from(quadrant % 4);
    match function {
        Function::Sin => Ok(SIN[quadrant]),
        Function::Cos => Ok(SIN[(quadrant + 1) % 4]),
        _ if quadrant % 2 == 1 => fail(ErrorKind::Domain),
        _ => Ok(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degrees_are_exact_at_right_angles() {
        let cases = [
            (Function::Sin, 180.0, Ok(0.0)),
            (Function::Cos, 90.0, Ok(0.0)),
            (Function::Cos, -180.0, Ok(-1.0)),
            (Function::Sin, 450.0, Ok(1.0)),
            (Function::Tan, 90.0, Err(ErrorKind::Domain)),
            (Function::Tan, 180.0, Ok(0.0)),
        ];
        for (function, degrees, expected) in cases {
            let value = forward(function, degrees, Angle::Degree).map_err(|e| e.kind);
            assert_eq!(value, expected, "{function:?}({degrees})");
        }
    }

    #[test]
    fn radians_are_exact_at_multiples_of_a_right_angle() {
        use std::f64::consts::{FRAC_PI_2, PI};
        let cases = [
            (Function::Sin, PI, Ok(0.0)),
            (Function::Sin, 3.0 * PI, Ok(0.0)),
            (Function::Cos, FRAC_PI_2, Ok(0.0)),
            (Function::Cos, -PI, Ok(-1.0)),
            (Function::Sin, -FRAC_PI_2, Ok(-1.0)),
            (Function::Tan, FRAC_PI_2, Err(ErrorKind::Domain)),
        ];
        for (function, radians, expected) in cases {
            let value = forward(function, radians, Angle::Radian).map_err(|e| e.kind);
            assert_eq!(value, expected, "{function:?}({radians})");
        }
        let near = forward(Function::Sin, PI + 1e-9, Angle::Radian).unwrap();
        assert!((near + 1e-9).abs() < 1e-15);
        assert_eq!(forward(Function::Sin, 1e-20, Angle::Radian), Ok(1e-20));
    }

    #[test]
    fn degrees_convert_elsewhere() {
        let half = forward(Function::Sin, 30.0, Angle::Degree).unwrap();
        assert!((half - 0.5).abs() < 1e-15);
        assert_eq!(inverse(Function::ArcCos, 0.0, Angle::Degree), Ok(90.0));
        assert_eq!(
            inverse(Function::ArcSin, 2.0, Angle::Degree).map_err(|e| e.kind),
            Err(ErrorKind::Domain)
        );
    }
}
