use anyhow::Result;
use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use super::{CowArrayD, IntoJArray, Word};
use crate::{HasEmpty, JArray, JError};

#[derive(Clone, Debug, PartialEq)]
pub enum ArrayPair<'l, 'r> {
    BoolPair(CowArrayD<'l, u8>, CowArrayD<'r, u8>),
    IntPair(CowArrayD<'l, i64>, CowArrayD<'r, i64>),
    ExtIntPair(CowArrayD<'l, BigInt>, CowArrayD<'r, BigInt>),
    FloatPair(CowArrayD<'l, f64>, CowArrayD<'r, f64>),
    // CharArray(..) // char, again, lacks maths operators, making this annoying
}

#[derive(Debug)]
pub enum JArrays<'v> {
    BoolArrays(Vec<ArrayViewD<'v, u8>>),
    CharArrays(Vec<ArrayViewD<'v, char>>),
    IntArrays(Vec<ArrayViewD<'v, i64>>),
    ExtIntArrays(Vec<ArrayViewD<'v, BigInt>>),
    RationalArrays(Vec<ArrayViewD<'v, BigRational>>),
    FloatArrays(Vec<ArrayViewD<'v, f64>>),
    ComplexArrays(Vec<ArrayViewD<'v, Complex64>>),
    BoxArrays(Vec<ArrayViewD<'v, Word>>),
}

macro_rules! homo_array {
    ($wot:path, $iter:expr) => {
        $iter
            .map(|x| match x {
                $wot(a) => Ok(a.into()),
                _ => Err(::anyhow::Error::from(JError::DomainError)),
            })
            .collect::<Result<Vec<_>>>()?
    };
}

impl<'a> JArrays<'a> {
    pub fn from_homo<'s>(arrs: &'s [&'a JArray]) -> Result<Self> {
        use homo_array as homo;
        use JArray::*;
        use JArrays::*;
        Ok(match arrs.iter().next().ok_or(JError::DomainError)? {
            BoolArray(_) => BoolArrays(homo!(BoolArray, arrs.iter())),
            CharArray(_) => CharArrays(homo!(CharArray, arrs.iter())),
            IntArray(_) => IntArrays(homo!(IntArray, arrs.iter())),
            ExtIntArray(_) => ExtIntArrays(homo!(ExtIntArray, arrs.iter())),
            RationalArray(_) => RationalArrays(homo!(RationalArray, arrs.iter())),
            FloatArray(_) => FloatArrays(homo!(FloatArray, arrs.iter())),
            ComplexArray(_) => ComplexArrays(homo!(ComplexArray, arrs.iter())),
            BoxArray(_) => BoxArrays(homo!(BoxArray, arrs.iter())),
        })
    }
}

macro_rules! impl_pair {
    ($arr:ident, $func:expr) => {
        match $arr {
            ArrayPair::BoolPair(x, y) => $func(x, y),
            ArrayPair::IntPair(x, y) => $func(x, y),
            ArrayPair::ExtIntPair(x, y) => $func(x, y),
            ArrayPair::FloatPair(x, y) => $func(x, y),
        }
    };
}

macro_rules! impl_pair_op {
    ($name:ident, $op:path) => {
        pub fn $name(&self) -> JArray {
            impl_pair!(self, |x, y| ($op(x, y) as ArrayD<_>).into_jarray())
        }
    };
}

impl ArrayPair<'_, '_> {
    impl_pair_op!(plus, ::std::ops::Add::add);
    impl_pair_op!(minus, ::std::ops::Sub::sub);
    impl_pair_op!(star, ::std::ops::Mul::mul);
    impl_pair_op!(slash, ::std::ops::Div::div);
    impl_pair_op!(lessthan, elementwise_lt);
}

fn elementwise_lt<T: Clone + HasEmpty + PartialOrd>(
    x: &CowArrayD<T>,
    y: &CowArrayD<T>,
) -> ArrayD<i64> {
    // TODO - not quite right when x and y shapes are different, fix generically:
    // https://code.jsoftware.com/wiki/Vocabulary/Agreement
    let empty_shape = x.shape();
    let mut result: ArrayD<i64> = ArrayD::from_elem(empty_shape, HasEmpty::empty());
    azip!((a in &mut result, x in x, y in y) *a = if x < y { 1 } else { 0 });
    result
}
