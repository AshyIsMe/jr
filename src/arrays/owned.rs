use std::iter;

use anyhow::{bail, Result};
use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::IntoDimension;
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::ToPrimitive;

use super::{CowArrayD, JArrayCow};
use crate::Word;

#[derive(Clone, Debug, PartialEq)]
pub enum JArray {
    BoolArray(ArrayD<u8>),
    CharArray(ArrayD<char>),
    IntArray(ArrayD<i64>),
    ExtIntArray(ArrayD<BigInt>),
    RationalArray(ArrayD<BigRational>),
    FloatArray(ArrayD<f64>),
    ComplexArray(ArrayD<Complex64>),
    BoxArray(ArrayD<Word>),
}

// TODO: not exported?
#[macro_export]
macro_rules! impl_array {
    ($arr:ident, $func:expr) => {
        match $arr {
            JArray::BoolArray(a) => $func(a),
            JArray::CharArray(a) => $func(a),
            JArray::IntArray(a) => $func(a),
            JArray::ExtIntArray(a) => $func(a),
            JArray::RationalArray(a) => $func(a),
            JArray::FloatArray(a) => $func(a),
            JArray::ComplexArray(a) => $func(a),
            JArray::BoxArray(a) => $func(a),
        }
    };
}

impl JArray {
    pub fn len(&self) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| a.len())
    }

    pub fn len_of(&self, axis: Axis) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| a.len_of(axis))
    }

    pub fn shape<'s>(&'s self) -> &[usize] {
        impl_array!(self, |a: &'s ArrayBase<_, _>| a.shape())
    }

    pub fn to_shape(&self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArrayCow> {
        use JArray::*;
        Ok(match self {
            BoolArray(a) => JArrayCow::BoolArray(a.to_shape(shape)?),
            CharArray(a) => JArrayCow::CharArray(a.to_shape(shape)?),
            IntArray(a) => JArrayCow::IntArray(a.to_shape(shape)?),
            ExtIntArray(a) => JArrayCow::ExtIntArray(a.to_shape(shape)?),
            RationalArray(a) => JArrayCow::RationalArray(a.to_shape(shape)?),
            FloatArray(a) => JArrayCow::FloatArray(a.to_shape(shape)?),
            ComplexArray(a) => JArrayCow::ComplexArray(a.to_shape(shape)?),
            BoxArray(a) => JArrayCow::BoxArray(a.to_shape(shape)?),
        })
    }

    pub fn choppo(&self, nega_rank: usize) -> Result<JArrayCow> {
        let shape = self.shape();

        if nega_rank > shape.len() {
            bail!("cannot ({}) given a shape of {:?}", nega_rank, shape);
        }

        let (common, surplus) = shape.split_at(shape.len() - nega_rank);
        let p = common.iter().product::<usize>();
        let new_shape = iter::once(p).chain(surplus.iter().copied()).collect_vec();

        self.to_shape(new_shape)
    }
}

impl JArray {
    pub fn approx(&self) -> Option<ArrayD<f32>> {
        use JArray::*;
        Some(match self {
            BoolArray(a) => a.map(|&v| v as f32),
            CharArray(a) => a.map(|&v| v as u32 as f32),
            IntArray(a) => a.map(|&v| v as f32),
            ExtIntArray(a) => a.map(|v| v.to_f32().unwrap_or(f32::NAN)),
            RationalArray(a) => a.map(|v| v.to_f32().unwrap_or(f32::NAN)),
            FloatArray(a) => a.map(|&v| v as f32),
            _ => return None,
        })
    }

    pub fn to_i64(&self) -> Option<CowArrayD<i64>> {
        use JArray::*;
        Some(match self {
            BoolArray(a) => a.map(|&v| i64::from(v)).into(),
            CharArray(a) => a.map(|&v| i64::from(v as u32)).into(),
            IntArray(a) => a.into(),
            // TODO: attempt coercion of other types? .map(try_from).collect::<Result<ArrayD<>>>?
            _ => return None,
        })
    }

    pub fn to_rat(&self) -> Option<CowArrayD<BigRational>> {
        use JArray::*;
        Some(match self {
            IntArray(a) => a.map(|&v| BigRational::new(v.into(), 1.into())).into(),
            RationalArray(a) => a.into(),
            // TODO: entirely missing other implementations
            _ => return None,
        })
    }

    pub fn to_c64(&self) -> Option<CowArrayD<Complex64>> {
        use JArray::*;
        Some(match self {
            BoolArray(a) => a.map(|&v| Complex64::new(f64::from(v), 0.)).into(),
            CharArray(a) => a.map(|&v| Complex64::new(f64::from(v as u32), 0.)).into(),
            IntArray(a) => a.map(|&v| Complex64::new(v as f64, 0.)).into(),
            ExtIntArray(a) => a
                .map(|v| Complex64::new(v.to_f64().unwrap_or(f64::NAN), 0.))
                .into(),
            // I sure expect absolutely no issues with NaNs creeping in here
            RationalArray(a) => a
                .map(|v| Complex64::new(v.to_f64().unwrap_or(f64::NAN), 0.))
                .into(),
            FloatArray(a) => a.map(|&v| Complex64::new(v, 0.)).into(),
            ComplexArray(a) => a.into(),
            // ??
            BoxArray(_) => return None,
        })
    }
}
