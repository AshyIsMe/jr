use std::{fmt, iter};

use anyhow::{anyhow, ensure, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::{IntoDimension, Slice};
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::ToPrimitive;

use super::nd_ext::len_of_0;
use crate::arrays::elem::Elem;
use crate::arrays::{display, size_of_shape_checked};
use crate::cells::fill_promote_reshape;
use crate::number::Num;
use crate::{arr0ad, IntoVec, JError};

pub type ArcArrayD<T> = ArcArray<T, IxDyn>;
pub type CowArrayD<'t, T> = CowArray<'t, T, IxDyn>;
pub type BoxArray = ArcArrayD<JArray>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum JArrayKind {
    Bool,
    Char,
    Int,
    ExtInt,
    Rational,
    Float,
    Complex,
    Box,
}

// AA TODO - impl ops::Add/Sub/Mul/Div for JArray similar to Num???
// Would need to re-implement agreement but it'd mean SIMD would work possibly

#[derive(Clone)]
pub enum JArray {
    BoolArray(ArcArrayD<u8>),
    CharArray(ArcArrayD<char>),
    IntArray(ArcArrayD<i64>),
    ExtIntArray(ArcArrayD<BigInt>),
    RationalArray(ArcArrayD<BigRational>),
    FloatArray(ArcArrayD<f64>),
    ComplexArray(ArcArrayD<Complex64>),
    BoxArray(BoxArray),
    // TODO: Some or all of these?
    // UnicodeCharArray(_)
    // Unicode4CharArray(_)
    // SymbolArray(_)
    // SparseBoolArray(_)
    // SparseCharArray(_)
    // SparseIntArray(_)
    // SparseFloatArray(_)
    // SparseComplexArray(_)
    // SparseBoxArray(_)
}

impl fmt::Debug for JArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use JArray::*;
        if f.alternate() {
            match self {
                BoolArray(a) => write!(f, "BoolArray({a:?})"),
                CharArray(a) => write!(f, "CharArray({a:?})"),
                IntArray(a) => write!(f, "IntArray({a:?})"),
                ExtIntArray(a) => write!(f, "ExtIntArray({a:?})"),
                RationalArray(a) => write!(f, "RationalArray({a:?})"),
                FloatArray(a) => write!(f, "FloatArray({a:?})"),
                ComplexArray(a) => write!(f, "ComplexArray({a:?})"),
                BoxArray(a) => write!(f, "BoxArray({a:?})"),
            }
        } else {
            match self {
                BoolArray(a) => write!(f, "BoolArray({a})"),
                CharArray(a) => {
                    if a.shape().len() <= 1 {
                        write!(f, "CharArray({:?})", a.iter().collect::<String>())
                    } else {
                        write!(f, "CharArray({a})")
                    }
                }
                IntArray(a) => write!(f, "IntArray({a})"),
                ExtIntArray(a) => write!(f, "ExtIntArray({a})"),
                RationalArray(a) => write!(f, "RationalArray({a})"),
                FloatArray(a) => write!(f, "FloatArray({a})"),
                ComplexArray(a) => write!(f, "ComplexArray({a})"),
                BoxArray(a) => write!(f, "BoxArray({a:?})"),
            }
        }
    }
}

impl PartialEq for JArray {
    fn eq(&self, other: &Self) -> bool {
        if self.shape() != other.shape() || self.len_of_0() != other.len_of_0() {
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

#[macro_export]
macro_rules! map_kind {
    ($kind:ident, $func:expr) => {
        match $kind {
            JArrayKind::Bool => JArray::BoolArray($func()?),
            JArrayKind::Char => JArray::CharArray($func()?),
            JArrayKind::Int => JArray::IntArray($func()?),
            JArrayKind::ExtInt => JArray::ExtIntArray($func()?),
            JArrayKind::Rational => JArray::RationalArray($func()?),
            JArrayKind::Float => JArray::FloatArray($func()?),
            JArrayKind::Complex => JArray::ComplexArray($func()?),
            JArrayKind::Box => JArray::BoxArray($func()?),
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

// no, I don't know either
pub trait OuterIter: ExactSizeIterator<Item = JArray> + DoubleEndedIterator {}
impl<T: ExactSizeIterator<Item = JArray> + DoubleEndedIterator> OuterIter for T {}

impl JArray {
    pub fn atomic_zero() -> JArray {
        JArray::BoolArray(arr0ad(0))
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

    pub fn transpose<'s>(&'s self) -> JArray {
        impl_array!(self, |a: &'s ArrayBase<_, _>| a.t().to_shared().into())
    }

    /// converts a singleton list into an atom (you almost certainly do not want this),
    /// returns the array unmodified if it is not a singleton
    pub fn singleton_to_atom(self) -> JArray {
        if self.shape() == [1] {
            self.reshape(IxDyn(&[]))
                .expect("stripping leading dimensions is okay")
        } else {
            self
        }
    }

    /// converts an atom into a singleton list
    /// returns the array unmodified if it is not an atom
    /// see also: [`rank_extend`]
    pub fn atom_to_singleton(self) -> JArray {
        if self.shape().is_empty() {
            self.rank_extend(1)
        } else {
            self
        }
    }

    pub fn select(&self, axis: Axis, ix: &[usize]) -> JArray {
        impl_array!(self, |a: &ArrayBase<_, _>| a
            .select(axis, ix)
            .into_shared()
            .into())
    }

    pub fn slice_axis(&self, axis: Axis, slice: Slice) -> Result<JArray> {
        let index = axis.index();
        ensure!(index < self.shape().len());
        let this_dim = self.shape()[index];
        if let Some(end) = slice.end.and_then(|i| usize::try_from(i).ok()) {
            ensure!(
                end < this_dim,
                "slice end, {end}, past end of axis {index}, of length {this_dim}"
            );
        }
        Ok(impl_array!(self, |a: &ArrayBase<_, _>| JArray::from(
            a.slice_axis(axis, slice).to_shared()
        )))
    }

    #[deprecated = "reshape() should always be sufficient?"]
    pub fn into_shape(self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArray> {
        impl_array!(self, |a: ArrayBase<_, _>| Ok(a
            .into_shape(shape)?
            .to_shared()
            .into()))
    }

    pub fn reshape(&self, shape: impl IntoDimension<Dim = IxDyn>) -> Result<JArray> {
        let dim = shape.into_dimension();
        if size_of_shape_checked(&dim)? != self.tally() {
            return Err(JError::DomainError).context("impossible reshape required");
        }
        impl_array!(self, |a: &ArrayBase<_, _>| Ok(a.reshape(dim).into()))
    }

    /// Create an array containing the same data and the same trailing shape, but with
    /// 1-shaped dimensions appended to bring the rank (the length of the shape) up to
    /// the `target`.
    ///
    /// Panics if `target > self.shape().len()`.
    pub fn rank_extend(&self, target: usize) -> JArray {
        let rank_extended_shape = (0..target - self.shape().len())
            .map(|_| &1)
            .chain(self.shape())
            .copied()
            .collect_vec();

        self.reshape(rank_extended_shape)
            .expect("rank extension is always valid")
    }

    pub fn create_cleared(&self) -> JArray {
        let empty_first = |shape: &[usize]| -> Vec<usize> {
            if shape.is_empty() {
                vec![0]
            } else {
                let mut shape = shape.to_vec();
                shape[0] = 0;
                shape
            }
        };
        map_array!(self, |a: &ArrayBase<_, _>| ArcArrayD::from_shape_vec(
            empty_first(a.shape()),
            Vec::new()
        )
        .expect("static shape"))
    }

    pub fn outer_iter<'v>(&'v self) -> Box<dyn OuterIter + 'v> {
        if self.shape().is_empty() {
            Box::new(iter::once(self.clone()))
        } else {
            impl_array!(self, |x: &'v ArrayBase<_, _>| Box::new(
                x.outer_iter().map(|v| v.to_shared().into())
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
                .map(|x| x.into_shape(surplus.clone()).unwrap().into_owned().into())
                .collect())
        }
    }

    pub fn into_elems(self) -> Vec<Elem> {
        impl_array!(self, |a: ArcArrayD<_>| a
            .into_iter()
            .map(Elem::from)
            .collect())
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
        if self.len_of_0() != 1 {
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
        use JArray::*;
        match self {
            BoolArray(a) => a.first().map(|&v| v.into()),
            IntArray(a) => a.first().map(|&v| v.into()),
            ExtIntArray(a) => a.first().map(|v| v.clone().into()),
            RationalArray(a) => a.first().map(|v| v.clone().into()),
            FloatArray(a) => a.first().map(|&v| v.into()),
            ComplexArray(a) => a.first().map(|v| v.clone().into()),
            CharArray(_) => return None,
            BoxArray(_) => return None,
        }
    }

    pub fn approx_i64_list(&self) -> Result<Vec<i64>> {
        if self.is_empty() {
            return Ok(Vec::new());
        }
        if self.shape().len() > 1 {
            return Err(JError::DomainError).context("non-list in list context");
        }

        self.clone()
            .into_nums()
            .ok_or(JError::DomainError)
            .context("expected a list of integers, found non-numbers")?
            .into_iter()
            .map(|x| x.value_i64())
            .collect::<Option<Vec<i64>>>()
            .ok_or(JError::DomainError)
            .context("expected a list of integers, found non-integers")
    }

    pub fn approx_usize_list(&self) -> Result<Vec<usize>> {
        self.approx_i64_list()?
            .into_iter()
            .map(usize_or_domain_err)
            .collect()
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
        self.approx_i64_one().and_then(usize_or_domain_err)
    }

    pub fn when_string(&self) -> Option<String> {
        if self.shape().len() > 1 {
            return None;
        }
        Some(self.when_char()?.into_iter().collect())
    }
}

fn usize_or_domain_err(v: i64) -> Result<usize> {
    usize::try_from(v)
        .map_err(|_| JError::DomainError)
        .context("unexpectedly negative")
}

impl JArray {
    /// For any of our plain data types (`i64`, `f64`, `char`, `Complex64`, ..), produce a list of plain data.
    ///
    /// This operation is cheap. [`JArray::into_shape`] on the result is cheap.
    ///
    /// This will not touch nested JArrays, and will form a `BoxArray`.
    ///
    /// This will always return a list, including an empty list, and never an atom.
    ///
    /// If you have mixed or multi-dimensional data, consider [`JArray::from_fill_promote`].
    ///
    /// ### Examples
    /// ```
    /// # use itertools::Itertools;
    /// # use ndarray::{ArcArray, array, ArrayD, IxDyn};
    /// # use jr::{arr0d, JArray};
    /// assert_eq!(
    ///     JArray::from_list([5i64, 6, 7]),
    ///     JArray::IntArray(array![5, 6, 7].into_dyn().into_shared()),
    /// );
    ///
    /// assert_eq!(
    ///     JArray::from_list(Vec::<i64>::new()),
    ///     JArray::IntArray(ArcArray::from_shape_vec(IxDyn(&[0]), Vec::new()).expect("static shape")),
    /// );
    ///
    /// // construct a box array
    /// let items = [
    ///     JArray::from(arr0d(6.3)),
    ///     JArray::from_list([5i64, 6, 7]),
    ///   ];
    /// assert_eq!(
    ///    JArray::from_list(items),
    ///    JArray::BoxArray(array![
    ///       JArray::from(arr0d(6.3)),
    ///       JArray::from_list([5i64, 6, 7]),
    ///     ].into_dyn().into_shared()),
    ///   );
    /// ```
    pub fn from_list<T>(v: impl IntoVec<T>) -> JArray
    where
        JArray: From<ArrayD<T>>,
    {
        JArray::from(v.into_array())
    }

    /// Lay out a list of `JArray`s as components in a bigger array.
    ///
    /// This "unboxes" the input, it is performing: `> (<1 2 3), (<3)` -> `2 3 $ 1 2 3 4 0 0`:
    /// ```text
    ///    > (<1 2 3), (<3)
    /// 1 2 3
    /// 4 0 0
    /// ```
    ///
    /// The input list represents the outer dimension, the returned `shape()` will
    /// always start with the len() of the input.
    ///
    /// This operation is *not* cheap, if you have plain data, please use [`JArray::from_list`].
    ///
    /// If you want to construct a box array, use [`JArray::from_list`] on the `Vec<JArray>` directly.
    ///
    /// This is multiple phases:
    /// Takes multiple arrays,
    /// fills them out to the same size,
    /// promotes them to the same type,
    /// and adds a dimension to represent the outer iterator
    ///
    /// ### Examples
    ///
    /// ```
    /// # use itertools::Itertools;
    /// use ndarray::{array, ArrayD};
    /// # use jr::{arr0d, IntoVec, JArray};
    /// # fn atom<T>(v: T) -> JArray where JArray: From<ArrayD<T>> { JArray::from(arr0d(v)) }
    /// # fn list<T: Clone>(v: &[T]) -> JArray where JArray: From<ArrayD<T>> { JArray::from_list(v.to_vec()) }
    /// let items = [atom(5i64), list(&[2i64, 3, 4])];
    /// let outer_dimension = items.len();
    /// let fpl = JArray::from_fill_promote(items).unwrap();
    /// assert_eq!(fpl.shape()[0], outer_dimension);
    /// assert_eq!(fpl.shape(), &[2, 3]);
    /// assert_eq!(
    ///     fpl,
    ///     JArray::IntArray(array![
    ///         // the atom and its fill
    ///         [5, 0, 0],
    ///         // the list, which has forced the shape of the 'inner' array
    ///         [2, 3, 4],
    ///     ].into_dyn().into_shared())
    /// );
    ///
    ///
    /// let items = [atom(6.3), atom(5i64)];
    /// let outer_dimension = items.len();
    /// let fpl = JArray::from_fill_promote(items).unwrap();
    /// assert_eq!(fpl.shape()[0], outer_dimension);
    /// assert_eq!(fpl.shape(), &[2]);
    /// assert_eq!(
    ///     fpl,
    ///     JArray::FloatArray(array![
    ///         // note, no inner array, the atoms are expanded in-place
    ///         6.3,
    ///         // the 5i64 has been promoted to a 5.0f64
    ///         5.0,
    ///     ].into_dyn().into_shared())
    /// );
    /// ```
    pub fn from_fill_promote(items: impl IntoIterator<Item = JArray>) -> Result<JArray> {
        let vec = items.into_iter().collect_vec();
        fill_promote_reshape((vec![vec.len()], vec))
    }

    /// Produce a 1D char array from a Rust String-like
    ///
    /// ### Examples
    ///
    /// ```
    /// # use ndarray::array;
    /// # use jr::JArray;
    /// assert_eq!(
    ///     JArray::from_string("hello"),
    ///     JArray::CharArray(array!['h', 'e', 'l', 'l', 'o'].into_dyn().into_shared()),
    /// );
    pub fn from_string(s: impl AsRef<str>) -> JArray {
        JArray::from_list(s.as_ref().chars().collect_vec())
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

    /// coerce bool arrays into int arrays, and return int arrays as-is; shape preserved
    pub fn to_i64(&self) -> Option<CowArrayD<i64>> {
        use JArray::*;
        Some(match self {
            BoolArray(a) => a.map(|&v| i64::from(v)).into(),
            IntArray(a) => a.into(),
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

    pub fn when_u8(&self) -> Option<&ArcArrayD<u8>> {
        match self {
            JArray::BoolArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_char(&self) -> Option<&ArcArrayD<char>> {
        match self {
            JArray::CharArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_i64(&self) -> Option<&ArcArrayD<i64>> {
        match self {
            JArray::IntArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_f64(&self) -> Option<&ArcArrayD<f64>> {
        match self {
            JArray::FloatArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_bigint(&self) -> Option<&ArcArrayD<BigInt>> {
        match self {
            JArray::ExtIntArray(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn when_complex(&self) -> Option<&ArcArrayD<Complex64>> {
        match self {
            JArray::ComplexArray(arr) => Some(arr),
            _ => None,
        }
    }
    pub fn when_rational(&self) -> Option<&ArcArrayD<BigRational>> {
        match self {
            JArray::RationalArray(arr) => Some(arr),
            _ => None,
        }
    }
    pub fn when_box(&self) -> Option<&ArcArrayD<JArray>> {
        match self {
            JArray::BoxArray(arr) => Some(arr),
            _ => None,
        }
    }
}

impl fmt::Display for JArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        display::jsoft(f, self)
    }
}

macro_rules! impl_from_nd {
    ($t:ty, $j:path) => {
        impl From<ArrayD<$t>> for JArray {
            fn from(value: ArrayD<$t>) -> JArray {
                $j(value.into_shared().into())
            }
        }
        impl From<ArcArrayD<$t>> for JArray {
            fn from(value: ArcArrayD<$t>) -> JArray {
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
            Num::Bool(a) => JArray::BoolArray(arr0ad(a)),
            Num::Int(a) => JArray::IntArray(arr0ad(a)),
            Num::ExtInt(a) => JArray::ExtIntArray(arr0ad(a)),
            Num::Rational(a) => JArray::RationalArray(arr0ad(a)),
            Num::Float(a) => JArray::FloatArray(arr0ad(a)),
            Num::Complex(a) => JArray::ComplexArray(arr0ad(a)),
        }
    }
}

impl From<Elem> for JArray {
    fn from(value: Elem) -> Self {
        match value {
            Elem::Char(a) => JArray::CharArray(arr0ad(a)),
            Elem::Boxed(a) => JArray::BoxArray(arr0ad(a)),
            Elem::Num(a) => JArray::from(a),
        }
    }
}
