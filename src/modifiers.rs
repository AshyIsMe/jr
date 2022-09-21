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

    let arrays = n
        .iter()
        .map(|w| match w {
            Word::Noun(arr) => Ok(arr),
            _ => Err(JError::DomainError),
        })
        .collect::<Result<Vec<_>, JError>>()?;

    let new_array = collect_int_arrs(&arrays)?;

    Ok(Word::Noun(new_array))
}

fn do_copy<T: Clone + Zero>(
    arr: &[&ArrayD<T>],
    empty_shape: &[usize],
) -> Result<ArrayD<T>, JError> {
    let mut result = Array::zeros(empty_shape);
    for item in arr {
        result
            .push(Axis(0), item.view())
            .map_err(JError::ShapeError)?;
    }
    Ok(result)
}

fn collect_int_arrs(arr: &[&JArray]) -> Result<JArray, JError> {
    let cell_shape = arr
        .iter()
        .map(|arr| arr.shape())
        .max()
        .ok_or(JError::DomainError)?;
    let empty_shape = iter::once(0)
        .chain(cell_shape.iter().copied())
        .collect::<Vec<_>>();

    macro_rules! impl_copy {
        ($k:path) => {{
            $k {
                a: do_copy(&homo_array!($k, arr.iter()), &empty_shape)?,
            }
        }};
    }

    Ok(match arr.iter().next().expect("non-empty") {
        JArray::BoolArray { .. } => impl_copy!(JArray::BoolArray),
        JArray::IntArray { .. } => impl_copy!(JArray::IntArray),
        JArray::ExtIntArray { .. } => impl_copy!(JArray::ExtIntArray),
        JArray::FloatArray { .. } => impl_copy!(JArray::FloatArray),
        JArray::CharArray { .. } => todo!("char isn't Zero, so we can't create an array of it"),
    })
}
