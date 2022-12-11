use num::complex::Complex64;
use num::{BigInt, BigRational};
use std::convert::From;
use std::fmt;

use anyhow::Result;
use ndarray::prelude::*;

use super::{Arrayable, IntoJArray};
use crate::modifiers::ModifierImpl;
use crate::verbs::VerbImpl;
use crate::{impl_array, primitive_verbs, JArray, JError};

// A Word is a part of speech.
#[derive(Clone, PartialEq, Debug)]
pub enum Word {
    LP,
    RP,
    StartOfLine, // used as placeholder when parsing
    Nothing,     // used as placeholder when parsing
    Name(String),

    NewLine,

    IsLocal,
    IsGlobal,
    Noun(JArray),
    Verb(String, VerbImpl),
    Adverb(String, ModifierImpl),
    Conjunction(String, ModifierImpl),

    Comment,
    DirectDefUnknown, // {{
    DirectDef(char),  // {{)n
    DirectDefEnd,     // }}
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

    pub fn is_control_symbol(&self) -> bool {
        use Word::*;
        match self {
            DirectDef(_) | DirectDefUnknown | DirectDefEnd => true,

            LP | RP | Name(_) | IsLocal | IsGlobal => false,
            Verb(_, _) | Noun(_) | Adverb(_, _) | Conjunction(_, _) => false,
            NewLine => false,
            StartOfLine | Nothing => false,
            Comment => unreachable!("should have been removed from the stream by now"),
        }
    }
}

impl Word {
    pub fn to_cells(&self) -> Result<Vec<Word>> {
        let ja = match self {
            Word::Noun(ja) => ja,
            _ => return Err(JError::DomainError.into()),
        };
        if ja.shape().is_empty() {
            return Ok(vec![Word::Noun(ja.clone())]);
        }
        Ok(impl_array!(ja, |a: &ArrayBase<_, _>| a
            .outer_iter()
            .map(|a| Word::Noun(a.into_owned().into_jarray()))
            .collect()))
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Word::Noun(a) => write!(f, "{}", a),
            Word::Verb(sv, _) => write!(f, "{}", sv),
            Word::Adverb(sa, _) => write!(f, "{}", sa),
            Word::Conjunction(sc, _) => write!(f, "{}", sc),
            //_ => write!(f, "{:+}", self),
            _ => todo!("Display for Word {:?}", self),
        }
    }
}

macro_rules! impl_from_atom {
    ($t:ty, $j:path) => {
        impl From<$t> for Word {
            fn from(value: $t) -> Word {
                Word::Noun($j(Array::from_elem(IxDyn(&[]), value.into())))
            }
        }
    };
}
impl_from_atom!(char, JArray::CharArray);
impl_from_atom!(u8, JArray::BoolArray);
impl_from_atom!(i32, JArray::IntArray);
impl_from_atom!(i64, JArray::IntArray);
impl_from_atom!(BigInt, JArray::ExtIntArray);
impl_from_atom!(BigRational, JArray::RationalArray);
impl_from_atom!(f64, JArray::FloatArray);
impl_from_atom!(Complex64, JArray::ComplexArray);
