use std::{fmt, iter};

use anyhow::Result;
use log::debug;
use ndarray::prelude::*;
use ndarray::{IntoDimension, Slice};
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::ToPrimitive;

use super::{CowArrayD, JArrayCow};
use crate::arrays::elem::Elem;
use crate::number::Num;
use crate::Word;

#[derive(Clone, PartialEq)]
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

impl fmt::Debug for JArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use JArray::*;
        match self {
            BoolArray(a) => write!(f, "BoolArray({a})"),
            CharArray(a) => write!(f, "CharArray({a})"),
            IntArray(a) => write!(f, "IntArray({a})"),
            ExtIntArray(a) => write!(f, "ExtIntArray({a})"),
            RationalArray(a) => write!(f, "RationalArray({a})"),
            FloatArray(a) => write!(f, "FloatArray({a})"),
            ComplexArray(a) => write!(f, "ComplexArray({a})"),
            BoxArray(a) => write!(f, "BoxArray({a})"),
        }
    }
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

macro_rules! map_array {
    ($arr:ident, $func:expr) => {
        match $arr {
            JArray::BoolArray(a) => JArray::BoolArray($func(a)),
            JArray::CharArray(a) => JArray::CharArray($func(a)),
            JArray::IntArray(a) => JArray::IntArray($func(a)),
            JArray::ExtIntArray(a) => JArray::ExtIntArray($func(a)),
            JArray::RationalArray(a) => JArray::RationalArray($func(a)),
            JArray::FloatArray(a) => JArray::FloatArray($func(a)),
            JArray::ComplexArray(a) => JArray::ComplexArray($func(a)),
            JArray::BoxArray(a) => JArray::BoxArray($func(a)),
        }
    };
}

impl JArray {
    pub fn is_empty(&self) -> bool {
        impl_array!(self, |a: &ArrayBase<_, _>| { a.is_empty() })
    }

    pub fn len(&self) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| {
            match a.shape() {
                [] => 1,
                a => a[0],
            }
        })
    }

    pub fn len_of(&self, axis: Axis) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| a.len_of(axis))
    }

    pub fn shape<'s>(&'s self) -> &[usize] {
        impl_array!(self, |a: &'s ArrayBase<_, _>| a.shape())
    }

    pub fn select(&self, axis: Axis, ix: &[usize]) -> JArray {
        map_array!(self, |a: &ArrayBase<_, _>| a.select(axis, ix))
    }

    // TODO: CoW
    pub fn slice_axis(&self, axis: Axis, slice: Slice) -> JArray {
        map_array!(self, |a: &ArrayBase<_, _>| a
            .slice_axis(axis, slice)
            .to_owned())
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

    // TODO: Iterator
    pub fn outer_iter<'v>(&'v self) -> Vec<JArrayCow<'v>> {
        if self.shape().is_empty() {
            vec![JArrayCow::from(self)]
        } else {
            impl_array!(self, |x: &'v ArrayBase<_, _>| x
                .outer_iter()
                .map(JArrayCow::from)
                .collect())
        }
    }

    /// rank_iter, but the other way up, and more picky about its arguments
    pub fn dims_iter(&self, dims: usize) -> Vec<JArray> {
        assert!(
            dims <= self.shape().len(),
            "{dims} must be shorter than us: {}",
            self.shape().len()
        );
        self.rank_iter(
            (self.shape().len() - dims)
                .try_into()
                .expect("worst types; absolute worst"),
        )
    }

    // AA TODO: Real iterator instead of Vec
    pub fn rank_iter(&self, rank: i16) -> Vec<JArray> {
        // Similar to ndarray::axis_chunks_iter but j style ranks.
        // ndarray Axis(0) is the largest axis whereas for j 0 is atoms, 1 is lists etc
        debug!("rank_iter rank: {}", rank);
        if rank > self.shape().len() as i16 {
            vec![self.clone()]
        } else if rank == 0 {
            impl_array!(self, |x: &ArrayBase<_, _>| x
                .iter()
                .map(JArray::from)
                .collect::<Vec<JArray>>())
        } else {
            let shape = self.shape();
            let (leading, surplus) = if rank >= 0 {
                let (l, s) = shape.split_at(shape.len() - rank as usize);
                (l.to_vec(), s.to_vec())
            } else {
                // Negative rank is a real thing in j, it's just the same but from the left instead of the right.
                let (l, s) = shape.split_at(rank.unsigned_abs() as usize);
                (l.to_vec(), s.to_vec())
            };
            debug!("leading: {:?}, surplus: {:?}", leading, surplus);
            let iter_shape: Vec<usize> = vec![
                iter::repeat(1usize).take(leading.len()).collect(),
                surplus.clone(),
            ]
            .concat();

            impl_array!(self, |x: &ArrayBase<_, _>| x
                .exact_chunks(IxDyn(&iter_shape))
                .into_iter()
                .map(|x| x
                    .into_shape(surplus.clone())
                    .unwrap()
                    .into_owned()
                    .into_jarray())
                .collect())
        }
    }

    pub fn into_elems(self) -> Vec<Elem> {
        impl_array!(self, |a: ArrayD<_>| a.into_iter().map(Elem::from).collect())
    }

    pub fn into_nums(self) -> Option<Vec<Num>> {
        use JArray::*;
        Some(match self {
            BoolArray(a) => a.into_iter().map(|v| v.into()).collect(),
            IntArray(a) => a.into_iter().map(|v| v.into()).collect(),
            ExtIntArray(a) => a.into_iter().map(|v| v.into()).collect(),
            RationalArray(a) => a.into_iter().map(|v| v.into()).collect(),
            FloatArray(a) => a.into_iter().map(|v| v.into()).collect(),
            ComplexArray(a) => a.into_iter().map(|v| v.into()).collect(),
            CharArray(_) => return None,
            BoxArray(_) => return None,
        })
    }

    pub fn single_elem(&self) -> Option<Elem> {
        if self.len() != 1 {
            return None;
        }
        Some(
            self.clone()
                .into_elems()
                .into_iter()
                .next()
                .expect("checked"),
        )
    }

    pub fn single_math_num(&self) -> Option<Num> {
        if self.len() != 1 {
            return None;
        }
        self.clone()
            .into_nums()
            .map(|v| v.into_iter().next().expect("checked"))
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

    pub fn when_u8(&self) -> Option<&ArrayD<u8>> {
        match self {
            JArray::BoolArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_i64(&self) -> Option<&ArrayD<i64>> {
        match self {
            JArray::IntArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_f64(&self) -> Option<&ArrayD<f64>> {
        match self {
            JArray::FloatArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_bigint(&self) -> Option<&ArrayD<BigInt>> {
        match self {
            JArray::ExtIntArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_complex(&self) -> Option<&ArrayD<Complex64>> {
        match self {
            JArray::ComplexArray(arr) => Some(arr),
            _ => None,
        }
    }
    pub fn when_rational(&self) -> Option<&ArrayD<BigRational>> {
        match self {
            JArray::RationalArray(arr) => Some(arr),
            _ => None,
        }
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

impl From<Num> for JArray {
    fn from(value: Num) -> Self {
        match value {
            Num::Bool(a) => JArray::from(a),
            Num::Int(a) => JArray::from(a),
            Num::ExtInt(a) => JArray::from(a),
            Num::Rational(a) => JArray::from(a),
            Num::Float(a) => JArray::from(a),
            Num::Complex(a) => JArray::from(a),
        }
    }
}

impl From<Elem> for JArray {
    fn from(value: Elem) -> Self {
        match value {
            Elem::Char(a) => JArray::from(a),
            Elem::Boxed(a) => JArray::from(a),
            Elem::Num(a) => JArray::from(a),
        }
    }
}
