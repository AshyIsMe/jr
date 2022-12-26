use std::{fmt, iter};

use anyhow::{anyhow, ensure, Context, Result};
use log::debug;
use ndarray::prelude::*;
use ndarray::{IntoDimension, Slice};
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::ToPrimitive;

use super::nd_ext::len_of_0;
use super::{CowArrayD, JArrayCow};
use crate::arrays::elem::Elem;
use crate::number::Num;
use crate::{arr0d, map_to_cow, JError, Word};

pub type BoxArray = ArrayD<JArray>;

#[derive(Clone)]
pub enum JArray {
    BoolArray(ArrayD<u8>),
    CharArray(ArrayD<char>),
    IntArray(ArrayD<i64>),
    ExtIntArray(ArrayD<BigInt>),
    RationalArray(ArrayD<BigRational>),
    FloatArray(ArrayD<f64>),
    ComplexArray(ArrayD<Complex64>),
    BoxArray(BoxArray),
}

impl fmt::Debug for JArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use JArray::*;
        match self {
            BoolArray(a) => write!(f, "BoolArray({a})"),
            CharArray(a) => write!(f, "CharArray({a})"),
            IntArray(a) => write!(f, "IntArray({a:?})"),
            ExtIntArray(a) => write!(f, "ExtIntArray({a})"),
            RationalArray(a) => write!(f, "RationalArray({a})"),
            FloatArray(a) => write!(f, "FloatArray({a})"),
            ComplexArray(a) => write!(f, "ComplexArray({a})"),
            BoxArray(a) => write!(f, "BoxArray({a:?})"),
        }
    }
}

impl PartialEq for JArray {
    fn eq(&self, other: &Self) -> bool {
        if self.shape() != other.shape() || self.len() != other.len() {
            return false;
        }

        self.clone().into_elems() == other.clone().into_elems()
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

#[macro_export]
macro_rules! impl_homo {
    ($x:ident, $y:ident, $func:expr) => {
        match ($x, $y) {
            (JArray::BoolArray(x), JArray::BoolArray(y)) => Ok(JArray::BoolArray($func(x, y)?)),
            (JArray::CharArray(x), JArray::CharArray(y)) => Ok(JArray::CharArray($func(x, y)?)),
            (JArray::IntArray(x), JArray::IntArray(y)) => Ok(JArray::IntArray($func(x, y)?)),
            (JArray::ExtIntArray(x), JArray::ExtIntArray(y)) => {
                Ok(JArray::ExtIntArray($func(x, y)?))
            }
            (JArray::RationalArray(x), JArray::RationalArray(y)) => {
                Ok(JArray::RationalArray($func(x, y)?))
            }
            (JArray::FloatArray(x), JArray::FloatArray(y)) => Ok(JArray::FloatArray($func(x, y)?)),
            (JArray::ComplexArray(x), JArray::ComplexArray(y)) => {
                Ok(JArray::ComplexArray($func(x, y)?))
            }
            (JArray::BoxArray(x), JArray::BoxArray(y)) => Ok(JArray::BoxArray($func(x, y)?)),
            _ => Err(JError::DomainError).context("expected homo arrays"),
        }
    };
}

impl JArray {
    pub fn atomic_zero() -> JArray {
        JArray::BoolArray(arr0d(0))
    }

    /// does the array contain zero elements, regardless of shape
    pub fn is_empty(&self) -> bool {
        impl_array!(self, |a: &ArrayBase<_, _>| { a.is_empty() })
    }

    #[deprecated = "different from ndarray: returns len_of_0(), not tally()"]
    pub fn len(&self) -> usize {
        self.len_of_0()
    }

    /// the length of the outermost axis, the length of `outer_iter`.
    pub fn len_of_0(&self) -> usize {
        impl_array!(self, len_of_0)
    }

    /// the number of elements in the array; the product of the shape (1 for atoms)
    pub fn tally(&self) -> usize {
        impl_array!(self, ArrayBase::len)
    }

    pub fn len_of(&self, axis: Axis) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| a.len_of(axis))
    }

    pub fn shape<'s>(&'s self) -> &[usize] {
        impl_array!(self, ArrayBase::shape)
    }

    pub fn transpose<'s>(&'s self) -> JArrayCow {
        impl_array!(self, |a: &'s ArrayBase<_, _>| CowArrayD::from(a.t()).into())
    }

    pub fn select(&self, axis: Axis, ix: &[usize]) -> JArray {
        impl_array!(self, |a: &ArrayBase<_, _>| a.select(axis, ix).into())
    }

    pub fn slice_axis<'v>(&'v self, axis: Axis, slice: Slice) -> Result<JArrayCow<'v>> {
        let index = axis.index();
        ensure!(index < self.shape().len());
        let this_dim = self.shape()[index];
        if let Some(end) = slice.end.and_then(|i| usize::try_from(i).ok()) {
            ensure!(
                end < this_dim,
                "slice end, {end}, past end of axis {index}, of length {this_dim}"
            );
        }
        Ok(impl_array!(self, |a: &'v ArrayBase<_, _>| JArrayCow::from(
            a.slice_axis(axis, slice)
        )))
    }

    pub fn to_shape<'v>(&'v self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArrayCow<'v>> {
        map_to_cow!(self, |a: &'v ArrayBase<_, _>| a.to_shape(shape))
    }

    pub fn into_shape(self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArray> {
        impl_array!(self, |a: ArrayBase<_, _>| Ok(a.into_shape(shape)?.into()))
    }

    pub fn outer_iter<'v>(&'v self) -> Box<dyn ExactSizeIterator<Item = JArrayCow<'v>> + 'v> {
        if self.shape().is_empty() {
            Box::new(iter::once(JArrayCow::from(self)))
        } else {
            impl_array!(self, |x: &'v ArrayBase<_, _>| Box::new(
                x.outer_iter().map(JArrayCow::from)
            ))
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
        if rank > self.shape().len() as i16 || self.is_empty() {
            vec![self.clone()]
        } else if rank == 0 {
            impl_array!(self, |x: &ArrayBase<_, _>| x
                .iter()
                .map(Elem::from)
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
        if self.tally() != 1 {
            return None;
        }
        self.clone()
            .into_nums()
            .map(|v| v.into_iter().next().expect("checked"))
    }

    pub fn approx_i64_one(&self) -> Result<i64> {
        let tally = self.tally();
        if tally != 1 {
            return Err(JError::DomainError)
                .with_context(|| anyhow!("expected a single integer, found {tally} items"));
        }

        self.single_math_num()
            .and_then(|num| num.value_i64())
            .ok_or(JError::DomainError)
            .context("expected integers, found non-integers")
    }

    pub fn approx_usize_one(&self) -> Result<usize> {
        self.approx_i64_one().and_then(|v| {
            usize::try_from(v)
                .map_err(|_| JError::DomainError)
                .context("unexpectedly negative")
        })
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

    pub fn when_char(&self) -> Option<&ArrayD<char>> {
        match self {
            JArray::CharArray(arr) => Some(arr),
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
impl_into_jarray!(ArrayD<JArray>, JArray::BoxArray);

macro_rules! impl_from_nd {
    ($t:ty, $j:path) => {
        impl From<ArrayD<$t>> for JArray {
            fn from(value: ArrayD<$t>) -> JArray {
                $j(value.into())
            }
        }
    };
}

impl_from_nd!(u8, JArray::BoolArray);
impl_from_nd!(char, JArray::CharArray);
impl_from_nd!(i64, JArray::IntArray);
impl_from_nd!(BigInt, JArray::ExtIntArray);
impl_from_nd!(BigRational, JArray::RationalArray);
impl_from_nd!(f64, JArray::FloatArray);
impl_from_nd!(Complex64, JArray::ComplexArray);
impl_from_nd!(JArray, JArray::BoxArray);

impl From<Num> for JArray {
    fn from(value: Num) -> Self {
        match value {
            Num::Bool(a) => JArray::BoolArray(arr0d(a)),
            Num::Int(a) => JArray::IntArray(arr0d(a)),
            Num::ExtInt(a) => JArray::ExtIntArray(arr0d(a)),
            Num::Rational(a) => JArray::RationalArray(arr0d(a)),
            Num::Float(a) => JArray::FloatArray(arr0d(a)),
            Num::Complex(a) => JArray::ComplexArray(arr0d(a)),
        }
    }
}

impl From<Elem> for JArray {
    fn from(value: Elem) -> Self {
        match value {
            Elem::Char(a) => JArray::CharArray(arr0d(a)),
            Elem::Boxed(a) => JArray::BoxArray(arr0d(a)),
            Elem::Num(a) => JArray::from(a),
        }
    }
}
