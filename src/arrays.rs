use std::fmt;

pub use crate::modifiers::*;
pub use crate::verbs::*;

use anyhow::{anyhow, Context, Result};
use ndarray::prelude::*;
use thiserror::Error;

// TODO: https://code.jsoftware.com/wiki/Vocabulary/ErrorMessages
#[derive(Debug, Error)]
pub enum JError {
    #[error("AssertionFailure: Your assert. line did not produce (a list of all) 1 (true)")]
    AssertionFailure,
    #[error("Break: You interrupted execution with the JBreak icon")]
    Break,
    #[error("ControlError: While loading script: bad use of if. else. end. etc")]
    ControlError,
    #[error(
        "DomainError: Invalid valence: The verb doesn't have a definition for the valence it was executed with"
    )]
    // #[error("Invalid value: An argument or operand has an invalid value")] ,
    // #[error("Invalid public assignment: You've used both (z=:) and (z=.) for some name z")] ,
    // #[error("Pun in definitions: A name was referred to as one part of speech, but the definition was later changed to another part of speech")] ,
    DomainError,
    #[error("FileNameError: nonexistent device or file")]
    FileNameError,
    #[error("FileNumberError: no file open with that number")]
    FileNumberError,
    #[error("FoldLimit: your Fold did not terminate when you expected")]
    FoldLimit,
    #[error("IllFormedName: Invalid underscores in a name")]
    IllFormedName,
    #[error("IllFormedNumber: A word starting with a number is not a valid number")]
    IllFormedNumber,
    #[error("IndexError: accessing out of bounds of your array")]
    IndexError,
    #[error("InterfaceError: illegal filename or request")]
    InterfaceError,
    #[error("LengthError: x and y do not agree, or an argument has invalid length")]
    LengthError,
    #[error("LocaleError: You tried to use an expired locale")]
    LocaleError,
    #[error("LimitError: number is beyond J's limit")]
    LimitError,
    #[error("NaNError: result is not a valid number")]
    NaNError,
    #[error("NonceError: feature not supported yet")]
    NonceError,
    #[error( "NonUniqueSparseElements: You attempted an operation on a sparse array that would have required expanding the array")]
    NonUniqueSparseElements,
    #[error("NounResultWasRequired: Verbs, and test blocks within explicit definitions, must produce noun results")]
    NounResultWasRequired,
    #[error("OpenQuote: string started but not ended")]
    OpenQuote,
    #[error("OutOfMemory: noun too big for computer")]
    OutOfMemory,
    #[error("RankError: operand can't have that rank")]
    RankError,
    #[error("SecurityViolation: J has attempted something insecure after you demanded heightened security")]
    SecurityViolation,
    #[error("SpellingError: You've . or : in the wrong place")]
    SpellingError,
    // #[error("During debugging: You tried to change the definition of a suspended entity")]
    #[error("StackError: Any time: Too many recursions took place")]
    StackError,
    #[error("SyntaxError: Sentence has an unexecutable phrase")]
    SyntaxError,
    #[error("TimeLimit: Execution took too long")]
    TimeLimit,
    #[error("UncaughtThrow: There was no catcht. block to pick up your throw")]
    UncaughtThrow,
    #[error("ValueError: that name has no value yet")]
    ValueError,

    #[error("ShapeError: shape error: {0}")]
    ShapeError(#[from] ndarray::ShapeError),

    #[error("{0} (legacy)")]
    Legacy(String),
}

impl JError {
    pub(crate) fn custom(message: impl ToString) -> anyhow::Error {
        anyhow::Error::from(JError::Legacy(message.to_string()))
    }
}

type CowArrayD<'t, T> = CowArray<'t, T, IxDyn>;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary
// A Word is a part of speech.
#[derive(Clone, PartialEq, Debug)]
pub enum Word {
    LP,
    RP,
    StartOfLine, // used as placeholder when parsing
    Nothing,     // used as placeholder when parsing
    Name(String),

    IsLocal,
    IsGlobal,
    Noun(JArray),
    Verb(String, VerbImpl),
    Adverb(String, ModifierImpl),
    Conjunction(String, ModifierImpl),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JArray {
    BoolArray(ArrayD<u8>),
    CharArray(ArrayD<char>),
    IntArray(ArrayD<i64>),
    ExtIntArray(ArrayD<i128>), // TODO: num::bigint::BigInt
    //RationalArray { ... }, // TODO: num::rational::Rational64
    FloatArray(ArrayD<f64>),
    //ComplexArray { ... },  // TODO: num::complex::Complex64
    BoxArray(ArrayD<Word>),
    //EmptyArray, // How do we do this properly?
}

impl JArray {
    pub fn approx(&self) -> Option<ArrayD<f32>> {
        use JArray::*;
        Some(match self {
            BoolArray(a) => a.map(|&v| v as f32),
            CharArray(a) => a.map(|&v| v as u32 as f32),
            IntArray(a) => a.map(|&v| v as f32),
            ExtIntArray(a) => a.map(|&v| v as f32),
            FloatArray(a) => a.map(|&v| v as f32),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ArrayPair<'l, 'r> {
    BoolPair(CowArrayD<'l, u8>, CowArrayD<'r, u8>),
    IntPair(CowArrayD<'l, i64>, CowArrayD<'r, i64>),
    ExtIntPair(CowArrayD<'l, i128>, CowArrayD<'r, i128>),
    FloatPair(CowArrayD<'l, f64>, CowArrayD<'r, f64>),
    // CharArray(..) // char, again, lacks maths operators, making this annoying
}

pub enum JArrays<'a> {
    BoolArrays(Vec<&'a ArrayD<u8>>),
    CharArrays(Vec<&'a ArrayD<char>>),
    IntArrays(Vec<&'a ArrayD<i64>>),
    ExtIntArrays(Vec<&'a ArrayD<i128>>),
    FloatArrays(Vec<&'a ArrayD<f64>>),
    BoxArrays(Vec<&'a ArrayD<Word>>),
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

impl<'a> JArrays<'a> {
    pub fn from_homo(arrs: &[&'a JArray]) -> Result<Self> {
        use JArray::*;
        Ok(match arrs.iter().next().ok_or(JError::DomainError)? {
            BoolArray(_) => JArrays::BoolArrays(crate::homo_array!(BoolArray, arrs.iter())),
            CharArray(_) => JArrays::CharArrays(crate::homo_array!(CharArray, arrs.iter())),
            IntArray(_) => JArrays::IntArrays(crate::homo_array!(IntArray, arrs.iter())),
            ExtIntArray(_) => JArrays::ExtIntArrays(crate::homo_array!(ExtIntArray, arrs.iter())),
            FloatArray(_) => JArrays::FloatArrays(crate::homo_array!(FloatArray, arrs.iter())),
            BoxArray(_) => JArrays::BoxArrays(crate::homo_array!(BoxArray, arrs.iter())),
        })
    }
}

#[macro_export]
macro_rules! impl_array {
    ($arr:ident, $func:expr) => {
        match $arr {
            JArray::BoolArray(a) => $func(a),
            JArray::CharArray(a) => $func(a),
            JArray::IntArray(a) => $func(a),
            JArray::ExtIntArray(a) => $func(a),
            JArray::FloatArray(a) => $func(a),
            JArray::BoxArray(a) => $func(a),
        }
    };
}

#[macro_export]
macro_rules! reduce_arrays {
    ($arr:expr, $func:expr) => {
        match $arr {
            JArrays::BoolArrays(ref a) => JArray::BoolArray($func(a)?),
            JArrays::CharArrays(ref a) => JArray::CharArray($func(a)?),
            JArrays::IntArrays(ref a) => JArray::IntArray($func(a)?),
            JArrays::ExtIntArrays(ref a) => JArray::ExtIntArray($func(a)?),
            JArrays::FloatArrays(ref a) => JArray::FloatArray($func(a)?),
            JArrays::BoxArrays(ref a) => JArray::BoxArray($func(a)?),
        }
    };
}

#[macro_export]
macro_rules! homo_array {
    ($wot:path, $iter:expr) => {
        $iter
            .map(|x| match x {
                $wot(a) => Ok(a),
                _ => Err(::anyhow::Error::from(JError::DomainError)),
            })
            .collect::<Result<Vec<_>>>()?
    };
}

impl JArray {
    pub fn is_empty(&self) -> bool {
        impl_array!(self, |a: &ArrayBase<_, _>| a.is_empty())
    }

    pub fn len(&self) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| a.len())
    }

    pub fn len_of(&self, axis: Axis) -> usize {
        impl_array!(self, |a: &ArrayBase<_, _>| a.len_of(axis))
    }

    pub fn shape<'s>(&'s self) -> &[usize] {
        impl_array!(self, |a: &'s ArrayBase<_, _>| a.shape())
    }

    pub fn to_cells<'s>(&'s self, rank: usize) -> Result<Vec<JArray>> {
        impl_array!(self, |a: &'s ArrayBase<_, _>| {
            if rank > a.shape().len() {
                Ok(vec![self.clone()])
            } else {
                let p = &a.shape()[..a.shape().len() - rank]
                    .iter()
                    .product::<usize>();
                let s = vec![vec![*p], a.shape()[a.shape().len() - rank..].to_vec()].concat();
                Ok(a.clone()
                    .into_shape(s)
                    .unwrap()
                    .outer_iter()
                    .map(|i| i.to_owned().into_jarray())
                    .collect())
            }
        })
    }
}

use crate::primitive_verbs;
use JArray::*;
use Word::*;

pub trait HasEmpty {
    fn empty() -> Self;
}

macro_rules! impl_empty {
    ($t:ty, $e:expr) => {
        impl HasEmpty for $t {
            fn empty() -> $t {
                $e
            }
        }
    };
}

impl_empty!(char, ' ');
impl_empty!(u8, 0);
impl_empty!(i64, 0);
impl_empty!(i128, 0);
impl_empty!(f64, 0.);
impl_empty!(Word, Noun(BoolArray(Array::from_elem(IxDyn(&[0]), 0))));

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
impl_into_jarray!(ArrayD<i128>, JArray::ExtIntArray);
impl_into_jarray!(ArrayD<f64>, JArray::FloatArray);
impl_into_jarray!(ArrayD<Word>, JArray::BoxArray);

// like IntoIterator<Item = T> + ExactSizeIterator
pub trait Arrayable<T> {
    fn len(&self) -> usize;
    fn into_vec(self) -> Result<Vec<T>>;

    fn into_array(self) -> Result<ArrayD<T>>
    where
        Self: Sized,
    {
        let len = self.len();
        let vec = self.into_vec()?;
        Array::from_shape_vec(IxDyn(&[len]), vec)
            .map_err(JError::ShapeError)
            .context("into_array")
    }
}

impl<T> Arrayable<T> for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn into_vec(self) -> Result<Vec<T>> {
        Ok(self)
    }
}

// This is designed for use with shape(), sorry if it caught something else.
impl Arrayable<i64> for &[usize] {
    fn len(&self) -> usize {
        <[usize]>::len(self)
    }

    fn into_vec(self) -> Result<Vec<i64>> {
        self.iter()
            .map(|&v| {
                i64::try_from(v)
                    .map_err(|_| JError::LimitError)
                    .with_context(|| anyhow!("{} doesn't fit in an i64", v))
            })
            .collect()
    }
}

impl<T: Clone, const N: usize> Arrayable<T> for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn into_vec(self) -> Result<Vec<T>> {
        Ok(self.to_vec())
    }
}

impl<T> Arrayable<T> for ArrayD<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn into_vec(self) -> Result<Vec<T>> {
        Ok(self.into_raw_vec())
    }

    fn into_array(self) -> Result<ArrayD<T>> {
        Ok(self)
    }
}

impl Word {
    pub fn noun<T>(v: impl Arrayable<T>) -> Result<Word>
    where
        ArrayD<T>: IntoJArray,
    {
        Ok(Word::Noun(v.into_array()?.into_jarray()))
    }

    /// primarily intended for asserts, hence the "static", and the PANIC on invalid input
    pub fn static_verb(v: &'static str) -> Word {
        Word::Verb(
            v.to_string(),
            primitive_verbs(v).expect("static verbs should be valid"),
        )
    }
}

pub fn char_array(x: impl AsRef<str>) -> Result<Word> {
    let v: Vec<char> = x.as_ref().chars().collect();
    Word::noun(v)
}

impl Word {
    pub fn to_cells(&self) -> Result<Vec<Word>> {
        let ja = match self {
            Noun(ja) => ja,
            _ => return Err(JError::DomainError.into()),
        };
        Ok(impl_array!(ja, |a: &ArrayBase<_, _>| a
            .outer_iter()
            .map(|a| Noun(a.into_owned().into_jarray()))
            .collect()))
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
        match self {
            BoxArray(_) => impl_array!(self, |a: &ArrayBase<_, _>| write!(f, "|{}|", a)),
            _ => impl_array!(self, |a: &ArrayBase<_, _>| write!(f, "{}", a)),
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Noun(a) => write!(f, "{}", a),
            Verb(sv, _) => write!(f, "{}", sv),
            Adverb(sa, _) => write!(f, "{}", sa),
            Conjunction(sc, _) => write!(f, "{}", sc),
            //_ => write!(f, "{:+}", self),
            _ => todo!("Display for Word {:?}", self),
        }
    }
}
