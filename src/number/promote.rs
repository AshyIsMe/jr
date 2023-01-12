use anyhow::{anyhow, Context, Result};
use std::fmt;

use ndarray::prelude::*;
use num::complex::Complex64;
use num::rational::BigRational;
use num::BigInt;
use num_traits::Zero;

use super::Num;
use crate::arrays::{Elem, JArray, JArrayKind};
use crate::error::JError;

#[deprecated = "use infer_kind_from_boxes via. fill_promote_list"]
pub fn infer_kind_from_elems(parts: &[Elem]) -> JArrayKind {
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

pub fn infer_kind_from_boxes(parts: &[JArray]) -> JArrayKind {
    // priority table: https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Numeric_Precisions_in_J
    if parts.iter().any(|n| matches!(n, JArray::BoxArray(_))) {
        JArrayKind::Box
    } else if parts.iter().any(|n| matches!(n, JArray::CharArray(_))) {
        JArrayKind::Char
    } else if parts.iter().any(|n| matches!(n, JArray::ComplexArray(_))) {
        JArrayKind::Complex
    } else if parts.iter().any(|n| matches!(n, JArray::FloatArray(_))) {
        JArrayKind::Float
    } else if parts.iter().any(|n| matches!(n, JArray::RationalArray(_))) {
        JArrayKind::Rational
    } else if parts.iter().any(|n| matches!(n, JArray::ExtIntArray(_))) {
        JArrayKind::ExtInt
    } else if parts.iter().any(|n| matches!(n, JArray::IntArray(_))) {
        JArrayKind::Int
    } else {
        JArrayKind::Bool
    }
}

#[deprecated = "believed to be less efficient than fill_promote_list"]
pub fn promote_to_array(parts: Vec<Elem>) -> Result<JArray> {
    let kind = infer_kind_from_elems(&parts);
    elems_to_jarray(kind, parts)
}

/// panics if the kind isn't directly from [`infer_kind_from_elems`].
#[deprecated = "believed to be less efficient than fill_promote_list"]
pub fn elems_to_jarray(kind: JArrayKind, parts: Vec<Elem>) -> Result<JArray> {
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
            let Elem::Num(v) = v else { unreachable!("checked by infer") };
            Ok(match v {
                Num::Complex(i) => i,
                other => Complex64::new(other.approx_f64().expect("covered above"), 0.),
            })
        })),
        JArrayKind::Float => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("checked by infer") };
            Ok(match v {
                Num::Complex(_) => unreachable!("covered by above cases"),
                Num::Float(i) => i,
                other => other.approx_f64().expect("covered above"),
            })
        })),
        JArrayKind::Rational => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("checked by infer") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) => unreachable!("checked by infer"),
                Num::Rational(i) => i,
                Num::ExtInt(i) => BigRational::new(i, 1.into()),
                Num::Int(i) => BigRational::new(i.into(), 1.into()),
                Num::Bool(i) => BigRational::new(i.into(), 1.into()),
            })
        })),
        JArrayKind::ExtInt => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("checked by infer") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) => {
                    unreachable!("checked by infer")
                }
                Num::ExtInt(i) => i,
                Num::Int(i) => i.into(),
                Num::Bool(i) => i.into(),
            })
        })),
        JArrayKind::Int => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("checked by infer") };
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) | Num::ExtInt(_) => {
                    unreachable!("checked by infer")
                }
                Num::Int(i) => i,
                Num::Bool(i) => i.into(),
            })
        })),
        JArrayKind::Bool => arrayise(parts.into_iter().map(|v| {
            let Elem::Num(v) = v else { unreachable!("checked by infer") };
            Ok(match v {
                Num::Complex(_)
                | Num::Float(_)
                | Num::Rational(_)
                | Num::ExtInt(_)
                | Num::Int(_) => unreachable!("checked by infer"),
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

pub trait Promote: Clone + fmt::Debug {
    fn promote(value: Elem) -> Self;
}

impl Promote for JArray {
    fn promote(value: Elem) -> Self {
        match value {
            Elem::Boxed(b) => b,
            Elem::Num(n) => n.into(),
            _ => unreachable!("promotion inference error"),
        }
    }
}

impl Promote for char {
    fn promote(value: Elem) -> char {
        use Num::*;
        match value {
            Elem::Char(c) => c,
            Elem::Num(Bool(0)) => ' ',
            _ => unreachable!("promotion inference error"),
        }
    }
}

impl Promote for Complex64 {
    fn promote(value: Elem) -> Complex64 {
        use Num::*;
        match value {
            Elem::Num(Complex(v)) => v,
            Elem::Num(n) => Complex64::new(n.approx_f64().expect("covered above"), 0.),
            _ => unreachable!("promotion inference error"),
        }
    }
}

impl Promote for f64 {
    fn promote(value: Elem) -> f64 {
        match value {
            Elem::Num(n) => n.approx_f64().expect("covered above"),
            _ => unreachable!("promotion inference error"),
        }
    }
}
impl Promote for BigRational {
    fn promote(value: Elem) -> BigRational {
        use Num::*;
        match value {
            Elem::Num(Rational(i)) => i,
            Elem::Num(ExtInt(i)) => BigRational::new(i, 1.into()),
            Elem::Num(Int(i)) => BigRational::new(i.into(), 1.into()),
            Elem::Num(Bool(i)) => BigRational::new(i.into(), 1.into()),
            _ => unreachable!("promotion inference error"),
        }
    }
}

impl Promote for BigInt {
    fn promote(value: Elem) -> Self {
        use Num::*;
        match value {
            Elem::Num(ExtInt(i)) => i,
            Elem::Num(Int(i)) => i.into(),
            Elem::Num(Bool(i)) => i.into(),
            _ => unreachable!("promotion inference error"),
        }
    }
}

impl Promote for i64 {
    fn promote(value: Elem) -> Self {
        use Num::*;
        match value {
            Elem::Num(Int(i)) => i,
            Elem::Num(Bool(i)) => i.into(),
            _ => unreachable!("promotion inference error"),
        }
    }
}

impl Promote for u8 {
    fn promote(value: Elem) -> u8 {
        use Num::*;
        match value {
            Elem::Num(Bool(v)) => v,
            _ => unreachable!("promotion inference error"),
        }
    }
}
