use anyhow::Result;
use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use super::Word;
use crate::{JArray, JError};

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
