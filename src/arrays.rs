pub use crate::modifiers::*;
pub use crate::verbs::*;

use ndarray::prelude::*;
use thiserror::Error;

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
    pub(crate) fn custom(message: impl ToString) -> JError {
        JError::Legacy(message.to_string())
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
pub enum JArray {
    BoolArray(ArrayD<u8>),
    CharArray(ArrayD<char>),
    IntArray(ArrayD<i64>),
    ExtIntArray(ArrayD<i128>), // TODO: num::bigint::BigInt
    //RationalArray { ... }, // TODO: num::rational::Rational64
    FloatArray(ArrayD<f64>),
    //ComplexArray { ... },  // TODO: num::complex::Complex64
    //EmptyArray, // How do we do this properly?
}

#[macro_export]
macro_rules! apply_array_homo {
    ($arr:ident, $func:expr) => {
        match $arr.iter().next().ok_or(JError::DomainError)? {
            JArray::BoolArray(_) => {
                JArray::BoolArray($func(&homo_array!(JArray::BoolArray, $arr.iter()))?)
            }
            JArray::IntArray(_) => {
                JArray::IntArray($func(&homo_array!(JArray::IntArray, $arr.iter()))?)
            }
            JArray::ExtIntArray(_) => {
                JArray::ExtIntArray($func(&homo_array!(JArray::ExtIntArray, $arr.iter()))?)
            }
            JArray::FloatArray(_) => {
                JArray::FloatArray($func(&homo_array!(JArray::FloatArray, $arr.iter()))?)
            }
            JArray::CharArray(_) => {
                JArray::CharArray($func(&homo_array!(JArray::CharArray, $arr.iter()))?)
            }
        }
    };
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
        }
    };
}

#[macro_export]
macro_rules! homo_array {
    ($wot:path, $iter:expr) => {
        $iter
            .map(|x| match x {
                $wot(a) => Ok(a),
                _ => Err(JError::DomainError),
            })
            .collect::<Result<Vec<_>, JError>>()?
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
}

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
            fn into_jarray(self) -> JArray {
                $j(self)
            }
        }
    };
}

impl_into_jarray!(ArrayD<u8>, JArray::BoolArray);
impl_into_jarray!(ArrayD<char>, JArray::CharArray);
impl_into_jarray!(ArrayD<i64>, JArray::IntArray);
impl_into_jarray!(ArrayD<i128>, JArray::ExtIntArray);
impl_into_jarray!(ArrayD<f64>, JArray::FloatArray);

// impl IntoJArray for ArrayD<i64> {
//     fn into_jarray(self) -> JArray {
//         JArray::IntArray(self)
//     }
// }

// like IntoIterator<Item = T> + ExactSizeIterator
pub trait Arrayable<T> {
    fn len(&self) -> usize;
    fn into_vec(self) -> Result<Vec<T>, JError>;

    fn into_array(self) -> Result<ArrayD<T>, JError>
    where
        Self: Sized,
    {
        let len = self.len();
        let vec = self.into_vec()?;
        Array::from_shape_vec(IxDyn(&[len]), vec).map_err(JError::ShapeError)
    }
}

impl<T> Arrayable<T> for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn into_vec(self) -> Result<Vec<T>, JError> {
        Ok(self)
    }
}

// This is designed for use with shape(), sorry if it caught something else.
impl Arrayable<i64> for &[usize] {
    fn len(&self) -> usize {
        <[usize]>::len(self)
    }

    fn into_vec(self) -> Result<Vec<i64>, JError> {
        self.iter()
            .map(|&v| i64::try_from(v).map_err(|_| JError::LimitError))
            .collect()
    }
}

impl<T: Clone, const N: usize> Arrayable<T> for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn into_vec(self) -> Result<Vec<T>, JError> {
        Ok(self.to_vec())
    }
}

impl Word {
    pub fn noun<T>(v: impl Arrayable<T>) -> Result<Word, JError>
    where
        ArrayD<T>: IntoJArray,
    {
        Ok(Word::Noun(v.into_array()?.into_jarray()))
    }
}

pub fn int_array(v: impl Arrayable<i64>) -> Result<Word, JError> {
    Word::noun(v)
}

pub fn char_array(x: impl AsRef<str>) -> Result<Word, JError> {
    let x = x.as_ref();
    Ok(Word::Noun(JArray::CharArray(ArrayD::from_shape_vec(
        IxDyn(&[x.chars().count()]),
        x.chars().collect(),
    )?)))
}

impl Word {
    pub fn to_cells(&self) -> Result<Vec<Word>, JError> {
        let ja = match self {
            Noun(ja) => ja,
            _ => return Err(JError::DomainError),
        };
        Ok(impl_array!(ja, |a: &ArrayBase<_, _>| a
            .outer_iter()
            .map(|a| Noun(a.into_owned().into_jarray()))
            .collect()))
    }
}
