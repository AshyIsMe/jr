use anyhow::{bail, Result};
use ndarray::prelude::*;
use ndarray::IntoDimension;
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
        impl_array!(self, ArrayBase::len)
    }

    pub fn shape(&self) -> &[usize] {
        impl_array!(self, ArrayBase::shape)
    }

    pub fn to_shape(&self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArrayCow> {
        use JArray::*;
        Ok(match self {
            JArrayCow::BoolArray(a) => JArrayCow::BoolArray(a.to_shape(shape)?),
            JArrayCow::CharArray(a) => JArrayCow::CharArray(a.to_shape(shape)?),
            JArrayCow::IntArray(a) => JArrayCow::IntArray(a.to_shape(shape)?),
            JArrayCow::ExtIntArray(a) => JArrayCow::ExtIntArray(a.to_shape(shape)?),
            JArrayCow::RationalArray(a) => JArrayCow::RationalArray(a.to_shape(shape)?),
            JArrayCow::FloatArray(a) => JArrayCow::FloatArray(a.to_shape(shape)?),
            JArrayCow::ComplexArray(a) => JArrayCow::ComplexArray(a.to_shape(shape)?),
            JArrayCow::BoxArray(a) => JArrayCow::BoxArray(a.to_shape(shape)?),
        })
    }

    // TODO: Iterator
    pub fn outer_iter(&'v self) -> Vec<Self> {
        impl_array!(self, |x: &'v ArrayBase<_, _>| x
            .outer_iter()
            .map(|x| Self::from(x))
            .collect())
    }

    // TODO: Iterator
    pub fn iter(&'v self) -> Vec<Self> {
        // AA TODO into_raw_vec() map, from
        // impl_array!(self, |x: &'v ArrayBase<_, _>| x
        //     .iter()
        //     .map(|x| Self::from(x))
        //     .collect())
        match self {
            JArrayCow::BoolArray(a) => a.iter().map(|x| JArrayCow::from(*x)).collect(),
            JArrayCow::CharArray(a) => a.iter().map(|x| JArrayCow::from(*x)).collect(),
            JArrayCow::IntArray(a) => a.iter().map(|x| JArrayCow::from(*x)).collect(),
            JArrayCow::ExtIntArray(a) => a.iter().map(|x| JArrayCow::from(x.clone())).collect(),
            JArrayCow::RationalArray(a) => a.iter().map(|x| JArrayCow::from(x.clone())).collect(),
            JArrayCow::FloatArray(a) => a.iter().map(|x| JArrayCow::from(x.clone())).collect(),
            JArrayCow::ComplexArray(a) => a.iter().map(|x| JArrayCow::from(x.clone())).collect(),
            JArrayCow::BoxArray(a) => a.iter().map(|x| JArrayCow::from(x.clone())).collect(),
        }
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

impl<'v> From<&'v JArray> for JArrayCow<'v> {
    fn from(value: &'v JArray) -> Self {
        match value {
            JArray::BoolArray(v) => JArrayCow::BoolArray(v.into()),
            JArray::CharArray(v) => JArrayCow::CharArray(v.into()),
            JArray::IntArray(v) => JArrayCow::IntArray(v.into()),
            JArray::ExtIntArray(v) => JArrayCow::ExtIntArray(v.into()),
            JArray::RationalArray(v) => JArrayCow::RationalArray(v.into()),
            JArray::FloatArray(v) => JArrayCow::FloatArray(v.into()),
            JArray::ComplexArray(v) => JArrayCow::ComplexArray(v.into()),
            JArray::BoxArray(v) => JArrayCow::BoxArray(v.into()),
        }
    }
}

macro_rules! impl_from_atom {
    ($t:ty, $j:path) => {
        impl<'v> From<$t> for JArrayCow<'v> {
            fn from(value: $t) -> JArrayCow<'v> {
                $j(CowArrayD::from(ArrayD::from_elem(IxDyn(&[]), value)))
            }
        }
    };
}
impl_from_atom!(u8, JArrayCow::BoolArray);
impl_from_atom!(char, JArrayCow::CharArray);
impl_from_atom!(i64, JArrayCow::IntArray);
impl_from_atom!(BigInt, JArrayCow::ExtIntArray);
impl_from_atom!(BigRational, JArrayCow::RationalArray);
impl_from_atom!(f64, JArrayCow::FloatArray);
impl_from_atom!(Complex64, JArrayCow::ComplexArray);
impl_from_atom!(Word, JArrayCow::BoxArray);

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
