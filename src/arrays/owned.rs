use std::{fmt, iter};

use anyhow::{bail, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::IntoDimension;
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::ToPrimitive;

use super::{CowArrayD, JArrayCow};
use crate::JError;
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

    // AA TODO: rank_iter() Iterator
    pub fn rank_iter(&self, rank: u8) -> Vec<JArray> {
        // Similar to ndarray::axis_chunks_iter but j style ranks.
        // ndarray Axis(0) is the largest axis whereas for j 0 is atoms, 1 is lists etc
        if rank as usize > self.shape().len() {
            vec![*self]
        } else if rank == 0 {
            impl_array!(self, |x: &ArrayBase<_, _>| x
                .into_raw_vec()
                .iter()
                .map(JArray::from)
                .collect::<Vec<JArray>>())
        } else if rank == 1 {
            // AA DEBUG testing rank 1
            let r = (self.shape().len() as i16 - rank as i16) as usize;
            impl_array!(self, |x: &ArrayBase<_, _>| x
                .axis_chunks_iter(Axis(r), 1)
                .map(|x| Self::from(x.into_owned()))
                .collect())
        // } else if rank < 0 {
        //     todo!("negative rank")
        } else {
            todo!("axis_chunks_iter properly")
        }
    }

    // pub fn to_cells<'s>(&'s self, rank: u8) -> Result<Vec<Self>> {
    //     if rank == 0 {
    //         Ok(impl_array!(self, |a: &'s ArrayBase<_, _>| {
    //             a.iter().map(|i| Self::from(i)).collect::<Vec<JArray>>()
    //         }))
    //     } else if rank > 0 {
    //         if rank > (self.shape().len() as u8) {
    //             Ok(vec![self.clone()])
    //         } else {
    //             let shape = self.shape();
    //             let (common, surplus) = shape.split_at(shape.len() - rank as usize);
    //             let p = common.iter().product::<usize>();
    //             let new_shape = iter::once(p).chain(surplus.iter().copied()).collect_vec();
    //             debug!("new_shape: {:?}, self: {}", new_shape, self);

    //             Ok(impl_array!(self, |a: &ArrayBase<_, _>| {
    //                 a.into_shape(new_shape)
    //                     .unwrap()
    //                     .outer_iter()
    //                     .map(|i| i.into()) // AA TODO get this into to work
    //                     .collect::<Vec<Self>>()
    //             }))
    //         }
    //     } else {
    //         todo!("negative rank")
    //     }
    // }

    pub fn choppo(&self, nega_rank: usize) -> Result<JArrayCow> {
        let shape = self.shape();
        debug!("shape: {:?}", shape);

        if nega_rank >= shape.len() {
            // bail!("cannot ({}) given a shape of {:?}", nega_rank, shape);
            // rank larger than shape is just the whole shape as is
            self.to_shape(shape)
        } else {
            let (common, surplus) = shape.split_at(shape.len() - nega_rank);
            let p = common.iter().product::<usize>();
            let new_shape = iter::once(p).chain(surplus.iter().copied()).collect_vec();

            debug!("new_shape: {:?}, self: {}", new_shape, self);
            self.to_shape(new_shape)
        }
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

impl fmt::Display for JArray {
    // TODO - match the real j output format style.
    // ie. 1 2 3 4 not [1, 2, 3, 4]
    // TODO - proper box array display:
    //    < 1 2 3
    //┌─────┐
    //│1 2 3│
    //└─────┘
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use JArray::*;
        match self {
            BoxArray(_) => impl_array!(self, |a: &ArrayBase<_, _>| write!(f, "|{}|", a)),
            _ => impl_array!(self, |a: &ArrayBase<_, _>| write!(f, "{}", a)),
        }
    }
}

pub trait IntoJArray {
    fn into_jarray(self) -> JArray;
    fn into_noun(self) -> Word
    where
        Self: Sized,
    {
        Word::Noun(self.into_jarray())
    }
}

macro_rules! impl_into_jarray {
    ($t:ty, $j:path) => {
        impl IntoJArray for $t {
            /// free for ArrayD<>, clones for unowned CowArrayD<>
            fn into_jarray(self) -> JArray {
                $j(self.into_owned())
            }
        }
    };
}

// these also cover the CowArrayD<> conversions because both are just aliases
// for ArrayBase<T> and the compiler lets us get away without lifetimes for some reason.
impl_into_jarray!(ArrayD<u8>, JArray::BoolArray);
impl_into_jarray!(ArrayD<char>, JArray::CharArray);
impl_into_jarray!(ArrayD<i64>, JArray::IntArray);
impl_into_jarray!(ArrayD<BigInt>, JArray::ExtIntArray);
impl_into_jarray!(ArrayD<BigRational>, JArray::RationalArray);
impl_into_jarray!(ArrayD<f64>, JArray::FloatArray);
impl_into_jarray!(ArrayD<Complex64>, JArray::ComplexArray);
impl_into_jarray!(ArrayD<Word>, JArray::BoxArray);

macro_rules! impl_from_atom {
    ($t:ty, $j:path) => {
        impl From<$t> for JArray {
            fn from(value: $t) -> JArray {
                $j(ArrayD::from(ArrayD::from_elem(IxDyn(&[]), value)))
            }
        }
    };
}
impl_from_atom!(u8, JArray::BoolArray);
impl_from_atom!(char, JArray::CharArray);
impl_from_atom!(i64, JArray::IntArray);
impl_from_atom!(BigInt, JArray::ExtIntArray);
impl_from_atom!(BigRational, JArray::RationalArray);
impl_from_atom!(f64, JArray::FloatArray);
impl_from_atom!(Complex64, JArray::ComplexArray);
impl_from_atom!(Word, JArray::BoxArray);

macro_rules! impl_from_atom_ref {
    ($t:ty, $j:path) => {
        impl From<$t> for JArray {
            fn from(value: $t) -> JArray {
                $j(ArrayD::from(ArrayD::from_elem(IxDyn(&[]), value.clone())))
            }
        }
    };
}
impl_from_atom_ref!(&u8, JArray::BoolArray);
impl_from_atom_ref!(&char, JArray::CharArray);
impl_from_atom_ref!(&i64, JArray::IntArray);
impl_from_atom_ref!(&BigInt, JArray::ExtIntArray);
impl_from_atom_ref!(&BigRational, JArray::RationalArray);
impl_from_atom_ref!(&f64, JArray::FloatArray);
impl_from_atom_ref!(&Complex64, JArray::ComplexArray);
impl_from_atom_ref!(&Word, JArray::BoxArray);
