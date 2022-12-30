use anyhow::{anyhow, Context, Result};

use ndarray::prelude::*;
use num::complex::Complex64;
use num::rational::BigRational;
use num_traits::Zero;

use super::Num;
use crate::arrays::{Elem, JArray, JArrayKind};
use crate::error::JError;

pub fn infer(parts: &[Elem]) -> JArrayKind {
    // priority table: https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Numeric_Precisions_in_J
    if parts.iter().any(|n| matches!(n, Elem::Boxed(_))) {
        JArrayKind::Box
    } else if parts.iter().any(|n| matches!(n, Elem::Char(_))) {
        JArrayKind::Char
    } else if parts
        .iter()
        .any(|n| matches!(n, Elem::Num(Num::Complex(_))))
    {
        JArrayKind::Complex
    } else if parts.iter().any(|n| matches!(n, Elem::Num(Num::Float(_)))) {
        JArrayKind::Float
    } else if parts
        .iter()
        .any(|n| matches!(n, Elem::Num(Num::Rational(_))))
    {
        JArrayKind::Rational
    } else if parts.iter().any(|n| matches!(n, Elem::Num(Num::ExtInt(_)))) {
        JArrayKind::ExtInt
    } else if parts.iter().any(|n| matches!(n, Elem::Num(Num::Int(_)))) {
        JArrayKind::Int
    } else {
        JArrayKind::Bool
    }
}

pub fn promote_to_array(parts: Vec<Elem>) -> Result<JArray> {
    let kind = infer(&parts);
    elem_to_j(kind, parts)
}

pub fn elem_to_j(kind: JArrayKind, parts: Vec<Elem>) -> Result<JArray> {
    // priority table: https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Numeric_Precisions_in_J
    match kind {
        JArrayKind::Box => arrayise(parts.into_iter().map(|v| match v {
            Elem::Boxed(b) => Ok(b),
            Elem::Num(n) => Ok(n.into()),
            other => Err(JError::NonceError).with_context(|| {
                anyhow!("TODO: unable to arrayise partially boxed content: {other:?}")
            }),
        })),
        JArrayKind::Char => arrayise(parts.into_iter().map(|v| match v {
            Elem::Char(c) => Ok(c),
            // we get here 'cos fill fills us with zeros, instead of with spaces, as it doesn't know?
            Elem::Num(n) if n.is_zero() => Ok(' '),
            other => Err(JError::NonceError).with_context(|| {
                anyhow!("TODO: unable to arrayise partially char content: {other:?}")
            }),
        })),
        JArrayKind::Complex => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(i) => i,
                other => Complex64::new(other.approx_f64().expect("covered above"), 0.),
            })
        })),
        JArrayKind::Float => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) => unreachable!("covered by above cases"),
                Num::Float(i) => i,
                other => other.approx_f64().expect("covered above"),
            })
        })),
        JArrayKind::Rational => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) => unreachable!("covered by above cases"),
                Num::Rational(i) => i,
                Num::ExtInt(i) => BigRational::new(i, 1.into()),
                Num::Int(i) => BigRational::new(i.into(), 1.into()),
                Num::Bool(i) => BigRational::new(i.into(), 1.into()),
            })
        })),
        JArrayKind::ExtInt => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) => {
                    unreachable!("covered by above cases")
                }
                Num::ExtInt(i) => i,
                Num::Int(i) => i.into(),
                Num::Bool(i) => i.into(),
            })
        })),
        JArrayKind::Int => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) | Num::ExtInt(_) => {
                    unreachable!("covered by above cases")
                }
                Num::Int(i) => i,
                Num::Bool(i) => i.into(),
            })
        })),
        JArrayKind::Bool => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("covered by above cases") };
            Ok(match v {
                Num::Complex(_)
                | Num::Float(_)
                | Num::Rational(_)
                | Num::ExtInt(_)
                | Num::Int(_) => unreachable!("covered by above cases"),
                Num::Bool(i) => i,
            })
        })),
    }
}

#[inline]
fn arrayise<T>(it: impl IntoIterator<Item = Result<T>>) -> Result<JArray>
where
    T: Clone,
    JArray: From<ArrayD<T>>,
{
    let vec = it.into_iter().collect::<Result<Vec<T>>>()?;
    Ok(if vec.len() == 1 {
        ArrayD::from_elem(IxDyn(&[]), vec.into_iter().next().expect("checked length"))
    } else {
        ArrayD::from_shape_vec(IxDyn(&[vec.len()]), vec).expect("simple shape")
    }
    .into())
}
