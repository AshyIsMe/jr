use anyhow::Result;
use ndarray::prelude::*;
use ndarray::IntoDimension;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use crate::JArray;

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
    BoxArray(CowArrayD<'a, JArray>),
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

#[macro_export]
macro_rules! map_to_cow {
    ($arr:ident, $func:expr) => {
        Ok(match $arr {
            JArray::BoolArray(a) => JArrayCow::BoolArray($func(a)?),
            JArray::CharArray(a) => JArrayCow::CharArray($func(a)?),
            JArray::IntArray(a) => JArrayCow::IntArray($func(a)?),
            JArray::ExtIntArray(a) => JArrayCow::ExtIntArray($func(a)?),
            JArray::RationalArray(a) => JArrayCow::RationalArray($func(a)?),
            JArray::FloatArray(a) => JArrayCow::FloatArray($func(a)?),
            JArray::ComplexArray(a) => JArrayCow::ComplexArray($func(a)?),
            JArray::BoxArray(a) => JArrayCow::BoxArray($func(a)?),
        })
    };
}

impl<'v> JArrayCow<'v> {
    pub fn len(&self) -> usize {
        //impl_array!(self, ArrayBase::len)
        impl_array!(self, |a: &ArrayBase<_, _>| {
            match a.shape() {
                [] => 1,
                a => a[0],
            }
        })
    }

    pub fn shape(&self) -> &[usize] {
        impl_array!(self, ArrayBase::shape)
    }

    pub fn to_shape(&self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArrayCow> {
        use JArrayCow::*;
        Ok(match self {
            BoolArray(a) => BoolArray(a.to_shape(shape)?),
            CharArray(a) => CharArray(a.to_shape(shape)?),
            IntArray(a) => IntArray(a.to_shape(shape)?),
            ExtIntArray(a) => ExtIntArray(a.to_shape(shape)?),
            RationalArray(a) => RationalArray(a.to_shape(shape)?),
            FloatArray(a) => FloatArray(a.to_shape(shape)?),
            ComplexArray(a) => ComplexArray(a.to_shape(shape)?),
            BoxArray(a) => BoxArray(a.to_shape(shape)?),
        })
    }

    // TODO: Iterator
    pub fn outer_iter(&'v self) -> Vec<Self> {
        impl_array!(self, |x: &'v ArrayBase<_, _>| x
            .outer_iter()
            .map(Self::from)
            .collect())
    }

    pub fn to_owned(&self) -> JArray {
        match self {
            JArrayCow::BoolArray(v) => JArray::BoolArray(v.to_owned()),
            JArrayCow::CharArray(v) => JArray::CharArray(v.to_owned()),
            JArrayCow::IntArray(v) => JArray::IntArray(v.to_owned()),
            JArrayCow::ExtIntArray(v) => JArray::ExtIntArray(v.to_owned()),
            JArrayCow::RationalArray(v) => JArray::RationalArray(v.to_owned()),
            JArrayCow::FloatArray(v) => JArray::FloatArray(v.to_owned()),
            JArrayCow::ComplexArray(v) => JArray::ComplexArray(v.to_owned()),
            JArrayCow::BoxArray(v) => JArray::BoxArray(v.to_owned()),
        }
    }

    pub fn into_owned(self) -> JArray {
        match self {
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
impl_from_nd!(JArray, JArrayCow::BoxArray);

macro_rules! impl_from_nd_cow {
    ($t:ty, $j:path) => {
        impl<'v> From<CowArrayD<'v, $t>> for JArrayCow<'v> {
            fn from(value: CowArrayD<'v, $t>) -> JArrayCow<'v> {
                $j(value.into())
            }
        }
    };
}

impl_from_nd_cow!(u8, JArrayCow::BoolArray);
impl_from_nd_cow!(char, JArrayCow::CharArray);
impl_from_nd_cow!(i64, JArrayCow::IntArray);
impl_from_nd_cow!(BigInt, JArrayCow::ExtIntArray);
impl_from_nd_cow!(BigRational, JArrayCow::RationalArray);
impl_from_nd_cow!(f64, JArrayCow::FloatArray);
impl_from_nd_cow!(Complex64, JArrayCow::ComplexArray);
impl_from_nd_cow!(JArray, JArrayCow::BoxArray);

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
impl_from_nd_view!(JArray, JArrayCow::BoxArray);
