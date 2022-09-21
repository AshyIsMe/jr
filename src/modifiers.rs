use std::iter;

use crate::Word;
use crate::{homo_array, JArray, JError};

use ndarray::prelude::*;
use num_traits::Zero;

// Implementations for Adverbs and Conjuntions
// https://code.jsoftware.com/wiki/Vocabulary/Modifiers
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModifierImpl {
    NotImplemented,

    //adverbs
    Slash,
    CurlyRt,

    //conjunctions
    HatCo,
}

impl ModifierImpl {
    pub fn exec<'a>(
        &'a self,
        x: Option<&Word>,
        u: &Word,
        v: &Word,
        y: &Word,
    ) -> Result<Word, JError> {
        match self {
            ModifierImpl::NotImplemented => a_not_implemented(x, u, y),
            ModifierImpl::Slash => a_slash(x, u, y),
            ModifierImpl::CurlyRt => a_curlyrt(x, u, y),
            ModifierImpl::HatCo => c_hatco(x, u, v, y),
        }
    }
}

pub fn a_not_implemented(_x: Option<&Word>, _u: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError::custom("adverb not implemented yet"))
}

pub fn a_slash(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word, JError> {
    match x {
        None => match u {
            Word::Verb(_, u) => match y {
                Word::Noun(_) => y
                    .to_cells()?
                    .into_iter()
                    .map(Ok)
                    .reduce(|x, y| u.exec(Some(&x?), &y?))
                    .ok_or(JError::DomainError)?,
                _ => Err(JError::custom("noun expected")),
            },
            _ => Err(JError::DomainError),
        },
        Some(_x) => Err(JError::custom("dyadic / not implemented yet")),
    }
}

pub fn a_curlyrt(_x: Option<&Word>, _u: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError::custom("adverb not implemented yet"))
}

pub fn c_hatco(x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word, JError> {
    match (u, v) {
        (Word::Verb(_, u), Word::Noun(JArray::IntArray { a: n })) => {
            // TODO framing fill properly https://code.jsoftware.com/wiki/Vocabulary/FramingFill
            Ok(collect_nouns(
                n.iter()
                    .map(|i| {
                        let mut t = y.clone();
                        for _ in 0..*i {
                            t = u.exec(x, &t).unwrap();
                        }
                        t
                    })
                    .collect(),
            )?)
        }
        (Word::Verb(_, _), Word::Verb(_, _)) => todo!("power conjunction verb right argument"),
        _ => Err(JError::DomainError),
    }
}

pub fn collect_nouns(n: Vec<Word>) -> Result<Word, JError> {
    // Collect a Vec<Word::Noun> into a single Word::Noun.
    // Must all be the same JArray type. ie. IntArray, etc

    let arr = n
        .iter()
        .map(|w| match w {
            Word::Noun(arr) => Ok(arr),
            _ => Err(JError::DomainError),
        })
        .collect::<Result<Vec<_>, JError>>()?;

    use JArray::*;

    let new_array = match arr.iter().next().ok_or(JError::DomainError)? {
        BoolArray { .. } => BoolArray {
            a: collect(&homo_array!(BoolArray, arr.iter()))?,
        },
        IntArray { .. } => IntArray {
            a: collect(&homo_array!(IntArray, arr.iter()))?,
        },
        ExtIntArray { .. } => ExtIntArray {
            a: collect(&homo_array!(ExtIntArray, arr.iter()))?,
        },
        FloatArray { .. } => FloatArray {
            a: collect(&homo_array!(FloatArray, arr.iter()))?,
        },
        CharArray { .. } => todo!("char isn't Zero, so we can't create an array of it"),
    };

    Ok(Word::Noun(new_array))
}

fn collect<T: Clone + Zero>(arr: &[&ArrayD<T>]) -> Result<ArrayD<T>, JError> {
    let cell_shape = arr
        .iter()
        .map(|arr| arr.shape())
        .max()
        .ok_or(JError::DomainError)?;
    let empty_shape = iter::once(0)
        .chain(cell_shape.iter().copied())
        .collect::<Vec<_>>();

    let mut result = Array::zeros(empty_shape);
    for item in arr {
        result
            .push(Axis(0), item.view())
            .map_err(JError::ShapeError)?;
    }
    Ok(result)
}
