mod arrayable;
mod cow;
mod owned;
mod plural;
mod word;

pub use arrayable::Arrayable;
pub use cow::{CowArrayD, JArrayCow};
pub use owned::{IntoJArray, JArray};
pub use plural::{ArrayPair, JArrays};
pub use word::Word;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary

#[macro_export]
macro_rules! reduce_arrays {
    ($arr:expr, $func:expr) => {
        match $arr {
            JArrays::BoolArrays(ref a) => JArray::BoolArray($func(a)?),
            JArrays::CharArrays(ref a) => JArray::CharArray($func(a)?),
            JArrays::IntArrays(ref a) => JArray::IntArray($func(a)?),
            JArrays::ExtIntArrays(ref a) => JArray::ExtIntArray($func(a)?),
            JArrays::RationalArrays(ref a) => JArray::RationalArray($func(a)?),
            JArrays::FloatArrays(ref a) => JArray::FloatArray($func(a)?),
            JArrays::ComplexArrays(ref a) => JArray::ComplexArray($func(a)?),
            JArrays::BoxArrays(ref a) => JArray::BoxArray($func(a)?),
        }
    };
}

use std::fmt;

use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational};

#[derive(PartialEq)]
pub enum JArraysOwned {
    BoolArrays(Vec<ArrayD<u8>>),
    CharArrays(Vec<ArrayD<char>>),
    IntArrays(Vec<ArrayD<i64>>),
    ExtIntArrays(Vec<ArrayD<BigInt>>),
    RationalArrays(Vec<ArrayD<BigRational>>),
    FloatArrays(Vec<ArrayD<f64>>),
    ComplexArrays(Vec<ArrayD<Complex64>>),
    BoxArrays(Vec<ArrayD<Word>>),
}

impl fmt::Debug for JArraysOwned {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JArraysOwned::IntArrays(x) => {
                write!(f, "IntArrarys({} items: ", x.len())?;
                for (idx, item) in x.iter().enumerate() {
                    write!(f, "{idx}:{item}")?;
                    if idx != x.len() - 1 {
                        write!(f, " || ")?;
                    }
                }
                write!(f, ")")
            }
            _ => write!(f, "{{unknown JArraysOwned type}}"),
        }
    }
}

impl JArraysOwned {
    pub fn len(&self) -> usize {
        use JArraysOwned::*;
        match self {
            BoolArrays(a) => a.len(),
            CharArrays(a) => a.len(),
            IntArrays(a) => a.len(),
            ExtIntArrays(a) => a.len(),
            RationalArrays(a) => a.len(),
            FloatArrays(a) => a.len(),
            ComplexArrays(a) => a.len(),
            BoxArrays(a) => a.len(),
        }
    }
}

#[macro_export]
macro_rules! spew_arrays {
    ($arr:expr, $func:expr) => {
        match $arr {
            JArrayCow::BoolArray(a) => JArraysOwned::BoolArrays($func(a)?),
            JArrayCow::CharArray(a) => JArraysOwned::CharArrays($func(a)?),
            JArrayCow::IntArray(a) => JArraysOwned::IntArrays($func(a)?),
            JArrayCow::ExtIntArray(a) => JArraysOwned::ExtIntArrays($func(a)?),
            JArrayCow::RationalArray(a) => JArraysOwned::RationalArrays($func(a)?),
            JArrayCow::FloatArray(a) => JArraysOwned::FloatArrays($func(a)?),
            JArrayCow::ComplexArray(a) => JArraysOwned::ComplexArrays($func(a)?),
            JArrayCow::BoxArray(a) => JArraysOwned::BoxArrays($func(a)?),
        }
    };
}

#[macro_export]
macro_rules! homowoned_array {
    ($wot:path, $iter:expr) => {
        $iter
            .map(|x| match x {
                $wot(a) => Ok(a),
                _ => Err(::anyhow::Error::from(JError::DomainError)),
            })
            .collect::<Result<Vec<_>>>()?
    };
}
