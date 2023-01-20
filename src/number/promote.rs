use std::fmt;

use num::complex::Complex64;
use num::rational::BigRational;
use num::BigInt;

use super::Num;
use crate::arrays::{Elem, JArray, JArrayKind};

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
