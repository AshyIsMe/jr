use std::iter;

use crate::JArray::{ExtIntArray, IntArray};
use crate::Word;
use crate::{map_array, map_array_map, JArray, JError};

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
        (Word::Verb(_, u), Word::Noun(IntArray { a: n })) => {
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

    //TODO This is clearly the wrong way to do this...
    match collect_int_nouns(n.clone()) {
        Ok(n) => Ok(n),
        _ => match collect_extint_nouns(n.clone()) {
            Ok(n) => Ok(n),
            _ => todo!("collect_nouns other JArray types"),
        },
    }
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

    Ok(match arr.iter().next().expect("non-empty") {
        JArray::IntArray { .. } => {
            let items = arr
                .iter()
                .map(|x| match x {
                    JArray::IntArray { a } => Ok(a),
                    _ => Err(JError::DomainError),
                })
                .collect::<Result<Vec<_>, JError>>()?;

            JArray::IntArray {
                a: do_copy(&items, &empty_shape)?,
            }
        }
        _ => todo!(),
    })

    // Ok(map_array_map!(arr, empty_shape, |a: &mut ArrayBase<_, _>, i: &ArrayBase<_, _>| a
    //     .push(Axis(0), i.view())
    //     .map_err(JError::ShapeError)))
}

//TODO This is clearly the wrong way to do this...
pub fn collect_int_nouns(n: Vec<Word>) -> Result<Word, JError> {
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

//TODO This is clearly the wrong way to do this...
pub fn collect_extint_nouns(n: Vec<Word>) -> Result<Word, JError> {
    let mut cell_shape: &[usize] = &[];
    let cells: Result<Vec<_>, _> = n
        .iter()
        .map(|w| match w {
            Word::Noun(ExtIntArray { a }) => {
                if a.shape() > cell_shape {
                    cell_shape = a.shape();
                }
                Ok(a)
            }
            _ => Err(JError::DomainError),
        })
        .collect();
    match cells {
        Ok(cells) => {
            // result new shape
            let mut empty_shape = Vec::new();
            empty_shape.extend_from_slice(&[0]);
            empty_shape.extend_from_slice(cell_shape);

            let mut a = Array::zeros(empty_shape);
            for i in cells.iter() {
                a.push(Axis(0), i.view()).unwrap();
            }
            Ok(Word::Noun(ExtIntArray { a }))
        }
        Err(e) => Err(e),
    }
}
