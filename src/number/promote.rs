use anyhow::{Context, Result};

use ndarray::prelude::*;
use num::complex::Complex64;
use num::rational::BigRational;

use super::Num;
use crate::arrays::{Elem, IntoJArray, JArray};
use crate::error::JError;

pub fn promote_to_array(parts: Vec<Elem>) -> Result<JArray> {
    // priority table: https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Numeric_Precisions_in_J
    if parts.iter().any(|n| matches!(n, Elem::Boxed(_))) {
        arrayise(parts.into_iter().map(|v| match v {
            Elem::Boxed(b) => Ok(b),
            _ => {
                Err(JError::NonceError).context("TODO: unable to arrayise partially boxed content")
            }
        }))
    } else if parts.iter().any(|n| matches!(n, Elem::Char(_))) {
        arrayise(parts.into_iter().map(|v| match v {
            Elem::Char(c) => Ok(c),
            _ => Err(JError::NonceError).context("TODO: unable to arrayise partially char content"),
        }))
    } else if parts.iter().any(|n| matches!(n, Elem::Literal(_))) {
        arrayise(parts.into_iter().map(|v| {
            match v {
                Elem::Literal(c) => Ok(c),
                _ => Err(JError::NonceError)
                    .context("TODO: unable to arrayise partially literal content"),
            }
        }))
    } else if parts
        .iter()
        .any(|n| matches!(n, Elem::Num(Num::Complex(_))))
    {
        arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(i) => i,
                other => Complex64::new(other.approx_f64().expect("covered above"), 0.),
            })
        }))
    } else if parts.iter().any(|n| matches!(n, Elem::Num(Num::Float(_)))) {
        arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) => unreachable!("covered by above cases"),
                Num::Float(i) => i,
                other => other.approx_f64().expect("covered above"),
            })
        }))
    } else if parts
        .iter()
        .any(|n| matches!(n, Elem::Num(Num::Rational(_))))
    {
        arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) => unreachable!("covered by above cases"),
                Num::Rational(i) => i,
                Num::ExtInt(i) => BigRational::new(i, 1.into()),
                Num::Int(i) => BigRational::new(i.into(), 1.into()),
                Num::Bool(i) => BigRational::new(i.into(), 1.into()),
            })
        }))
    } else if parts.iter().any(|n| matches!(n, Elem::Num(Num::ExtInt(_)))) {
        arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) => {
                    unreachable!("covered by above cases")
                }
                Num::ExtInt(i) => i,
                Num::Int(i) => i.into(),
                Num::Bool(i) => i.into(),
            })
        }))
    } else if parts.iter().any(|n| matches!(n, Elem::Num(Num::Int(_)))) {
        arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) | Num::ExtInt(_) => {
                    unreachable!("covered by above cases")
                }
                Num::Int(i) => i,
                Num::Bool(i) => i.into(),
            })
        }))
    } else {
        arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_)
                | Num::Float(_)
                | Num::Rational(_)
                | Num::ExtInt(_)
                | Num::Int(_) => unreachable!("covered by above cases"),
                Num::Bool(i) => i,
            })
        }))
    }
}

#[inline]
fn arrayise<T>(it: impl IntoIterator<Item = Result<T>>) -> Result<JArray>
where
    T: Clone,
    ArrayD<T>: IntoJArray,
{
    let vec = it.into_iter().collect::<Result<Vec<T>>>()?;
    Ok(if vec.len() == 1 {
        ArrayD::from_elem(IxDyn(&[]), vec.into_iter().next().expect("checked length"))
    } else {
        ArrayD::from_shape_vec(IxDyn(&[vec.len()]), vec).expect("simple shape")
    }
    .into_jarray())
}
