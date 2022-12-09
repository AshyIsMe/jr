//! This file contains v_ methods for simple-ish math operators and booleans
//! If it's not rank 0 or 0,0; and doesn't use Num, it probably doesn't belong here
//! If it's aware of cells or boxes, it probably doesn't belong here.

use std::cmp::Ordering;

use crate::arrays::Arrayable;
use crate::number::Num;
use crate::{IntoJArray, JArray, JError};

use anyhow::{Context, Result};
use ndarray::prelude::*;
use num::complex::Complex64;
use num_traits::{FloatConst, Zero};
use rand::prelude::*;

use super::maff::*;

/// = (dyad)
pub fn v_equal(x: &JArray, y: &JArray) -> Result<JArray> {
    d00eb(x, y, |x, y| x == y)
}

/// < (dyad)
pub fn v_less_than(x: &JArray, y: &JArray) -> Result<JArray> {
    d00erb(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Less) => Ok(true),
        None => Err(JError::DomainError).context("non-comparable number"),
        _ => Ok(false),
    })
}

/// <. (monad)
pub fn v_floor(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nrn(y, |y| {
        Ok(match y {
            Bool(x) => Bool(x),
            Int(x) => Int(x),
            ExtInt(x) => ExtInt(x),
            Rational(x) => Rational(x.floor()),
            Float(x) => Num::float_or_int(x.floor()),
            Complex(_) => {
                return Err(JError::NonceError)
                    .context("floor of a complex number is a complex subject")
            }
        })
    })
}
/// <. (dyad)
pub fn v_lesser_of_min(x: &JArray, y: &JArray) -> Result<JArray> {
    d00ere(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Less) | Some(Ordering::Equal) => Ok(x),
        Some(Ordering::Greater) => Ok(y),
        None => Err(JError::DomainError).context("non-comparable number"),
    })
}

/// <: (monad)
pub fn v_decrement(y: &JArray) -> Result<JArray> {
    m0nn(y, |y| y - Num::one())
}

/// <: (dyad)
pub fn v_less_or_equal(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// > (dyad)
pub fn v_larger_than(x: &JArray, y: &JArray) -> Result<JArray> {
    d00erb(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Greater) => Ok(true),
        None => Err(JError::DomainError).context("non-comparable number"),
        _ => Ok(false),
    })
}

/// >. (monad)
pub fn v_ceiling(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nrn(y, |y| {
        Ok(match y {
            Bool(x) => Bool(x),
            Int(x) => Int(x),
            ExtInt(x) => ExtInt(x),
            Rational(x) => Rational(x.ceil()),
            Float(x) => Num::float_or_int(x.ceil()),
            Complex(_) => {
                return Err(JError::NonceError)
                    .context("ceil of a complex number is a complex subject")
            }
        })
    })
}

/// >. (dyad)
pub fn v_larger_of_max(x: &JArray, y: &JArray) -> Result<JArray> {
    d00ere(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Greater) | Some(Ordering::Equal) => Ok(x),
        Some(Ordering::Less) => Ok(y),
        None => Err(JError::DomainError).context("non-comparable number"),
    })
}

/// >: (monad)
pub fn v_increment(y: &JArray) -> Result<JArray> {
    m0nn(y, |y| y + Num::one())
}

/// >: (dyad)
pub fn v_larger_or_equal(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// + (monad)
pub fn v_conjugate(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nn(y, |y| match y {
        Complex(c) => Complex(c.conj()),
        other => other,
    })
}
/// + (dyad)
pub fn v_plus(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| Ok(x + y))
}

/// +. (monad)
pub fn v_real_imaginary(y: &JArray) -> Result<JArray> {
    match y.to_c64() {
        Some(y) => {
            // y.insert_axis() ...
            let mut shape = y.shape().to_vec();
            shape.push(2);
            let values = y.iter().flat_map(|x| [x.re, x.im]).collect();
            Ok(ArrayD::from_shape_vec(shape, values)?.into_jarray())
        }
        None => Err(JError::DomainError.into()),
    }
}
/// +. (dyad)
pub fn v_gcd_or(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// +: (monad)
pub fn v_double(y: &JArray) -> Result<JArray> {
    m0nn(y, |y| Num::Int(2) * y)
}
/// +: (dyad)
pub fn v_not_or(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| match (x.value_bool(), y.value_bool()) {
        (Some(x), Some(y)) => Ok(Num::bool(!(x || y))),
        _ => Err(JError::DomainError).context("boolean operators only accept zeros and ones"),
    })
}

/// * (monad)
pub fn v_signum(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nrn(y, |y| {
        Ok(match y {
            Complex(_) => {
                return Err(JError::NonceError)
                    .context("sign of a complex number is a complex subject")
            }
            // dumb, so dumb
            n @ Bool(_) => n,
            n if n < Num::zero() => Int(-1),
            n if n.is_zero() => Int(0),
            n if n > Num::zero() => Int(1),
            _ => return Err(JError::NaNError).context("should be able to compare with zero"),
        })
    })
}
/// * (dyad)
pub fn v_times(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| Ok(x * y))
}

/// *. (monad)
pub fn v_length_angle(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nj(y, |y| {
        let pair = match y {
            Complex(c) => {
                let len = ((c.im * c.im) + (c.re * c.re)).sqrt();
                let ang = (c.im / c.re).atan();
                [len, ang]
            }
            other => [other.approx_f64().expect("complex covered above"), 0.],
        };

        pair.into_array()
            .expect("infalliable for fixed arrays")
            .into_jarray()
    })
}
/// *. (dyad)
pub fn v_lcm_and(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// *: (monad)
pub fn v_square(y: &JArray) -> Result<JArray> {
    // TODO: not clone?
    m0nn(y, |y| y.clone() * y)
}
/// *: (dyad)
pub fn v_not_and(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| match (x.value_bool(), y.value_bool()) {
        (Some(x), Some(y)) => Ok(Num::bool(!(x && y))),
        _ => Err(JError::DomainError).context("boolean operators only accept zeros and ones"),
    })
}

/// - (monad)
pub fn v_negate(y: &JArray) -> Result<JArray> {
    m0nn(y, |y| Num::zero() - y)
}
/// - (dyad)
pub fn v_minus(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| Ok(x - y))
}

/// -. (monad)
pub fn v_not(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nn(y, |y| match y {
        Bool(x) if x == 0 => Bool(1),
        Bool(x) if x == 1 => Bool(0),
        other => Num::one() - other,
    })
}

/// -: (monad)
pub fn v_halve(y: &JArray) -> Result<JArray> {
    m0nn(y, |y| y / Num::Int(2))
}

/// % (monad)
pub fn v_reciprocal(y: &JArray) -> Result<JArray> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("reciprocal expects a number")?;
    Ok((Num::one() / y).into())
}

/// % (dyad)
pub fn v_divide(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| Ok(x / y))
}

/// %. (monad)
pub fn v_matrix_inverse(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// %. (dyad)
pub fn v_matrix_divide(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// %: (monad)
pub fn v_square_root(y: &JArray) -> Result<JArray> {
    use Num::*;
    m0nrn(y, |y| {
        if let Some(f) = y.approx_f64() {
            if f >= 0. {
                Ok(Float(f.sqrt()))
            } else {
                Ok(Complex(Complex64::new(0., (-f).sqrt())))
            }
        } else {
            Err(JError::NonceError).context("square roots of complex numbers is a complex subject")
        }
    })
}

/// %: (dyad)
pub fn v_root(_x: &JArray, _y: &JArray) -> Result<JArray> {
    // weird promotion rules here; 2 %: 16 is 4. (float), 2x %: 16 is 4x (extended)
    Err(JError::NonceError.into())
}

/// ^ (monad)
pub fn v_exponential(y: &JArray) -> Result<JArray> {
    m0nrn(y, |y| {
        let y = y
            .approx_f64()
            .ok_or(JError::NonceError)
            .context("unable to exponential complexes")?;
        Ok(f64::E().powf(y).into())
    })
}

/// ^ (dyad)
pub fn v_power(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| {
        // TODO: incomplete around complex and return types
        match (x.approx_f64(), y.approx_f64()) {
            (Some(x), Some(y)) => Ok(x.powf(y).into()),
            _ => Err(JError::NonceError).context("unable to power complexes"),
        }
    })
}

/// ^. (monad)
pub fn v_natural_log(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ^. (dyad)
pub fn v_logarithm(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ~: (dyad)
pub fn v_not_equal(x: &JArray, y: &JArray) -> Result<JArray> {
    d00eb(x, y, |x, y| x != y)
}

/// | (monad)
pub fn v_magnitude(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// | (dyad)
pub fn v_residue(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ! (monad) (0)
pub fn v_factorial(y: &JArray) -> Result<JArray> {
    m0nrn(y, |y| {
        let y = i64::try_from(
            y.value_len()
                .ok_or(JError::NonceError)
                .context("integers only")?,
        )
        .context("oh you poor soul")?;
        Ok((1..=y).map(|x| x as f64).product::<f64>().into())
    })
}

/// ! (dyad)
pub fn v_out_of(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ? (monad)
pub fn v_roll(y: &JArray) -> Result<JArray> {
    let y = y
        .single_math_num()
        .and_then(|v| v.value_len())
        .ok_or(JError::DomainError)
        .context("expecting zero or a positive integer")?;
    let y = i64::try_from(y)
        .map_err(|_| JError::DomainError)
        .context("must fit in an int")?;
    let mut rng = thread_rng();
    Ok(match y {
        0 => JArray::from(Num::from(rng.gen::<f64>())),
        limit => JArray::from(Num::from(rng.gen_range(0..limit))),
    })
}

/// ? (dyad)
pub fn v_deal(x: &JArray, y: &JArray) -> Result<JArray> {
    let x = x
        .single_math_num()
        .and_then(|n| n.value_len())
        .ok_or(JError::DomainError)
        .context("expecting an usize-like x")?;
    // going via. value_len to elide floats and ban negatives
    let y = y
        .single_math_num()
        .and_then(|n| n.value_len())
        .ok_or(JError::DomainError)
        .context("expecting an usize-like y")?;
    if x > y {
        return Err(JError::DomainError).context("can't pick more items than we have");
    }
    let y = i64::try_from(y)
        .map_err(|_| JError::DomainError)
        .context("must fit in an int")?;
    let mut rng = rand::thread_rng();
    let mut chosen = (0..y).choose_multiple(&mut rng, x);
    chosen.shuffle(&mut rng);
    Ok(chosen.into_array()?.into_jarray())
}

/// ?. (dyad)
pub fn v_deal_fixed_seed(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// q: (monad)
pub fn v_prime_factors(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// q: (dyad)
pub fn v_prime_exponents(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// r. (monad)
pub fn v_angle(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// r. (dyad)
pub fn v_polar(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
