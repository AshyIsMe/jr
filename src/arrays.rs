pub use crate::adverbs::*;
pub use crate::verbs::*;

use ndarray::prelude::*;

// TODO: https://code.jsoftware.com/wiki/Vocabulary/ErrorMessages
#[derive(Debug)]
pub struct JError {
    message: String,
}

impl JError {
    pub(crate) fn custom(message: impl ToString) -> JError {
        JError {
            message: message.to_string(),
        }
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
    Verb(String, Box<VerbImpl>),
    Adverb(String, AdverbImpl),
    Conjunction(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JArray {
    BoolArray { a: ArrayD<u8> },
    CharArray { a: ArrayD<char> },
    IntArray { a: ArrayD<i64> },
    ExtIntArray { a: ArrayD<i128> }, // TODO: num::bigint::BigInt
    //RationalArray { ... }, // TODO: num::rational::Rational64
    FloatArray { a: ArrayD<f64> },
    //ComplexArray { ... },  // TODO: num::complex::Complex64
    //EmptyArray, // How do we do this properly?
}

use JArray::*;
use Word::*;

pub fn int_array(v: Vec<i64>) -> Result<Word, JError> {
    Ok(Word::Noun(IntArray {
        a: Array::from_shape_vec(IxDyn(&[v.len()]), v).unwrap(),
    }))
}

pub fn char_array(x: impl AsRef<str>) -> Word {
    let x = x.as_ref();
    Word::Noun(JArray::CharArray {
        a: ArrayD::from_shape_vec(IxDyn(&[x.chars().count()]), x.chars().collect()).unwrap(),
    })
}

impl Word {
    pub fn to_cells(&self) -> Result<Vec<Word>, JError> {
        match self {
            Noun(ja) => match ja {
                IntArray { a } => Ok(a
                    .outer_iter()
                    .map(|a| Noun(IntArray { a: a.into_owned() }))
                    .collect::<Vec<Word>>()),
                ExtIntArray { a } => Ok(a
                    .outer_iter()
                    .map(|a| Noun(ExtIntArray { a: a.into_owned() }))
                    .collect::<Vec<Word>>()),
                FloatArray { a } => Ok(a
                    .outer_iter()
                    .map(|a| Noun(FloatArray { a: a.into_owned() }))
                    .collect::<Vec<Word>>()),
                BoolArray { a } => Ok(a
                    .outer_iter()
                    .map(|a| Noun(BoolArray { a: a.into_owned() }))
                    .collect::<Vec<Word>>()),
                CharArray { a } => Ok(a
                    .outer_iter()
                    .map(|a| Noun(CharArray { a: a.into_owned() }))
                    .collect::<Vec<Word>>()),
            },
            _ => panic!("only nouns can be split into cells"),
        }
    }
}
