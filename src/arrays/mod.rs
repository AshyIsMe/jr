mod arrayable;
mod cow;
mod owned;

use std::fmt;

use crate::impl_array;
pub use crate::modifiers::*;
pub use crate::verbs::*;

use anyhow::Result;
use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational, Zero};
use thiserror::Error;

pub use arrayable::Arrayable;
pub use cow::{CowArrayD, JArrayCow};
pub use owned::JArray;

// TODO: https://code.jsoftware.com/wiki/Vocabulary/ErrorMessages
#[derive(Debug, Error)]
pub enum JError {
    #[error("Your assert. line did not produce (a list of all) 1 (true)")]
    AssertionFailure,
    #[error("You interrupted execution with the JBreak icon")]
    Break,
    #[error("While loading script: bad use of if. else. end. etc")]
    ControlError,
    #[error(
        "Invalid valence: The verb doesn't have a definition for the valence it was executed with"
    )]
    // #[error("Invalid value: An argument or operand has an invalid value")] ,
    // #[error("Invalid public assignment: You've used both (z=:) and (z=.) for some name z")] ,
    // #[error("Pun in definitions: A name was referred to as one part of speech, but the definition was later changed to another part of speech")] ,
    DomainError,
    #[error("nonexistent device or file")]
    FileNameError,
    #[error("no file open with that number")]
    FileNumberError,
    #[error("your Fold did not terminate when you expected")]
    FoldLimit,
    #[error("Invalid underscores in a name")]
    IllFormedName,
    #[error("A word starting with a number is not a valid number")]
    IllFormedNumber,
    #[error("accessing out of bounds of your array")]
    IndexError,
    #[error("illegal filename or request")]
    InterfaceError,
    #[error("x and y do not agree, or an argument has invalid length")]
    LengthError,
    #[error("You tried to use an expired locale")]
    LocaleError,
    #[error("number is beyond J's limit")]
    LimitError,
    #[error("result is not a valid number")]
    NaNError,
    #[error("feature not supported yet")]
    NonceError,
    #[error(
        "You attempted an operation on a sparse array that would have required expanding the array"
    )]
    NonUniqueSparseElements,
    #[error("Verbs, and test blocks within explicit definitions, must produce noun results")]
    NounResultWasRequired,
    #[error("string started but not ended")]
    OpenQuote,
    #[error("noun too big for computer")]
    OutOfMemory,
    #[error("operand can't have that rank")]
    RankError,
    #[error("J has attempted something insecure after you demanded heightened security")]
    SecurityViolation,
    #[error("You've . or : in the wrong place")]
    SpellingError,
    // #[error("During debugging: You tried to change the definition of a suspended entity")]
    #[error("Any time: Too many recursions took place")]
    StackError,
    #[error("Sentence has an unexecutable phrase")]
    SyntaxError,
    #[error("Execution took too long")]
    TimeLimit,
    #[error("There was no catcht. block to pick up your throw")]
    UncaughtThrow,
    #[error("that name has no value yet")]
    ValueError,

    #[error("shape error: {0}")]
    ShapeError(#[from] ndarray::ShapeError),

    #[error("{0} (legacy)")]
    Legacy(String),
}

impl JError {
    pub(crate) fn custom(message: impl ToString) -> anyhow::Error {
        anyhow::Error::from(JError::Legacy(message.to_string()))
    }
}

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
pub enum ArrayPair<'l, 'r> {
    BoolPair(CowArrayD<'l, u8>, CowArrayD<'r, u8>),
    IntPair(CowArrayD<'l, i64>, CowArrayD<'r, i64>),
    ExtIntPair(CowArrayD<'l, BigInt>, CowArrayD<'r, BigInt>),
    FloatPair(CowArrayD<'l, f64>, CowArrayD<'r, f64>),
    // CharArray(..) // char, again, lacks maths operators, making this annoying
}

pub enum JArrays<'a> {
    BoolArrays(Vec<&'a ArrayD<u8>>),
    CharArrays(Vec<&'a ArrayD<char>>),
    IntArrays(Vec<&'a ArrayD<i64>>),
    ExtIntArrays(Vec<&'a ArrayD<BigInt>>),
    RationalArrays(Vec<&'a ArrayD<BigRational>>),
    FloatArrays(Vec<&'a ArrayD<f64>>),
    ComplexArrays(Vec<&'a ArrayD<Complex64>>),
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
            RationalArray(_) => {
                JArrays::RationalArrays(crate::homo_array!(RationalArray, arrs.iter()))
            }
            FloatArray(_) => JArrays::FloatArrays(crate::homo_array!(FloatArray, arrs.iter())),
            ComplexArray(_) => {
                JArrays::ComplexArrays(crate::homo_array!(ComplexArray, arrs.iter()))
            }
            BoxArray(_) => JArrays::BoxArrays(crate::homo_array!(BoxArray, arrs.iter())),
        })
    }
}

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
impl_empty!(BigInt, BigInt::zero());
impl_empty!(BigRational, BigRational::zero());
impl_empty!(f64, 0.);
impl_empty!(Complex64, Complex64::zero());
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
impl_into_jarray!(ArrayD<BigInt>, JArray::ExtIntArray);
impl_into_jarray!(ArrayD<BigRational>, JArray::RationalArray);
impl_into_jarray!(ArrayD<f64>, JArray::FloatArray);
impl_into_jarray!(ArrayD<Complex64>, JArray::ComplexArray);
impl_into_jarray!(ArrayD<Word>, JArray::BoxArray);

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
