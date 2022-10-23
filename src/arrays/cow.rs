use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use crate::{JArray, Word};

pub type CowArrayD<'t, T> = CowArray<'t, T, IxDyn>;

#[derive(Clone, Debug, PartialEq)]
pub enum JArrayCow<'a> {
    BoolArray(CowArrayD<'a, u8>),
    CharArray(CowArrayD<'a, char>),
    IntArray(CowArrayD<'a, i64>),
    ExtIntArray(CowArrayD<'a, BigInt>),
    RationalArray(CowArrayD<'a, BigRational>),
    FloatArray(CowArrayD<'a, f64>),
    ComplexArray(CowArrayD<'a, Complex64>),
    BoxArray(CowArrayD<'a, Word>),
}

macro_rules! impl_array {
    ($arr:ident, $func:expr) => {
        match $arr {
            JArrayCow::BoolArray(a) => $func(a),
            JArrayCow::CharArray(a) => $func(a),
            JArrayCow::IntArray(a) => $func(a),
            JArrayCow::ExtIntArray(a) => $func(a),
            JArrayCow::RationalArray(a) => $func(a),
            JArrayCow::FloatArray(a) => $func(a),
            JArrayCow::ComplexArray(a) => $func(a),
            JArrayCow::BoxArray(a) => $func(a),
        }
    };
}

impl<'v> JArrayCow<'v> {
    pub fn len(&self) -> usize {
        impl_array!(self, |x: &ArrayBase<_, _>| x.len())
    }

    pub fn shape(&'v self) -> &[usize] {
        impl_array!(self, |x: &'v ArrayBase<_, _>| x.shape())
    }

    // TODO: Iterator
    pub fn outer_iter(&'v self) -> Vec<Self> {
        impl_array!(self, |x: &'v ArrayBase<_, _>| x
            .outer_iter()
            .map(|x| Self::from(x))
            .collect())
    }
}

impl<'v> From<JArrayCow<'v>> for JArray {
    fn from(value: JArrayCow<'v>) -> Self {
        match value {
            JArrayCow::BoolArray(v) => JArray::BoolArray(v.into_owned()),
            JArrayCow::CharArray(v) => JArray::CharArray(v.into_owned()),
            JArrayCow::IntArray(v) => JArray::IntArray(v.into_owned()),
            JArrayCow::ExtIntArray(v) => JArray::ExtIntArray(v.into_owned()),
            JArrayCow::RationalArray(v) => JArray::RationalArray(v.into_owned()),
            JArrayCow::FloatArray(v) => JArray::FloatArray(v.into_owned()),
            JArrayCow::ComplexArray(v) => JArray::ComplexArray(v.into_owned()),
            JArrayCow::BoxArray(v) => JArray::BoxArray(v.into_owned()),
        }
    }
}

macro_rules! impl_from_nd {
    ($t:ty, $j:path) => {
        impl<'v> From<ArrayD<$t>> for JArrayCow<'v> {
            fn from(value: ArrayD<$t>) -> JArrayCow<'v> {
                $j(value.into())
            }
        }
    };
}

impl_from_nd!(u8, JArrayCow::BoolArray);
impl_from_nd!(char, JArrayCow::CharArray);
impl_from_nd!(i64, JArrayCow::IntArray);
impl_from_nd!(BigInt, JArrayCow::ExtIntArray);
impl_from_nd!(BigRational, JArrayCow::RationalArray);
impl_from_nd!(f64, JArrayCow::FloatArray);
impl_from_nd!(Complex64, JArrayCow::ComplexArray);
impl_from_nd!(Word, JArrayCow::BoxArray);

macro_rules! impl_from_nd_view {
    ($t:ty, $j:path) => {
        impl<'v> From<ArrayViewD<'v, $t>> for JArrayCow<'v> {
            fn from(value: ArrayViewD<'v, $t>) -> JArrayCow<'v> {
                $j(value.into())
            }
        }
    };
}

impl_from_nd_view!(u8, JArrayCow::BoolArray);
impl_from_nd_view!(char, JArrayCow::CharArray);
impl_from_nd_view!(i64, JArrayCow::IntArray);
impl_from_nd_view!(BigInt, JArrayCow::ExtIntArray);
impl_from_nd_view!(BigRational, JArrayCow::RationalArray);
impl_from_nd_view!(f64, JArrayCow::FloatArray);
impl_from_nd_view!(Complex64, JArrayCow::ComplexArray);
impl_from_nd_view!(Word, JArrayCow::BoxArray);

macro_rules! impl_from_nd {
    ($t:ty, $j:path) => {
        impl<'v> From<ArrayD<$t>> for JArrayCow<'v> {
            fn from(value: ArrayD<$t>) -> JArrayCow<'v> {
                $j(value.into())
            }
        }
    };
}

impl_from_nd!(u8, JArrayCow::BoolArray);
impl_from_nd!(char, JArrayCow::CharArray);
impl_from_nd!(i64, JArrayCow::IntArray);
impl_from_nd!(BigInt, JArrayCow::ExtIntArray);
impl_from_nd!(BigRational, JArrayCow::RationalArray);
impl_from_nd!(f64, JArrayCow::FloatArray);
impl_from_nd!(Complex64, JArrayCow::ComplexArray);
impl_from_nd!(Word, JArrayCow::BoxArray);
