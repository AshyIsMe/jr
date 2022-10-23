use ndarray::prelude::*;
use num::{BigInt, BigRational};
use num::complex::Complex64;

use crate::{Word, JArray};

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

impl<'v> JArrayCow<'v> {
    pub fn len(&self) -> usize {
        match self {
            JArrayCow::IntArray(x) => x.len(),
            _ => todo!(),
        }
    }

    pub fn shape(&self) -> &[usize] {
        match self {
            JArrayCow::IntArray(x) => x.shape(),
            _ => todo!(),
        }
    }

    pub fn outer_iter(&self) -> impl Iterator<Item = JArrayCow> + Clone {
        match self {
            JArrayCow::IntArray(x) => x.outer_iter().map(|x| x.into()),
            _ => todo!(),
        }
    }
}

impl<'v> From<JArrayCow<'v>> for JArray {
    fn from(value: JArrayCow<'v>) -> Self {
        match value {
            JArrayCow::IntArray(v) => JArray::IntArray(v.into_owned()),
            _ => todo!(),
        }
    }
}

impl<'v> From<ArrayViewD<'v, i64>> for JArrayCow<'v> {
    fn from(value: ArrayViewD<'v, i64>) -> Self {
        JArrayCow::IntArray(value.into())
    }
}
