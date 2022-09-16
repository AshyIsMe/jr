use crate::int_array;
use crate::JArray;
use crate::JError;
use crate::Word;
use log::debug;
use ndarray::prelude::*;
use std::fmt::Debug;
use std::ops::Deref;

use crate::map_array;

use JArray::*;
use Word::*;

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Plus,
    Minus,
    Times,
    Number,
    Dollar,
    NotImplemented,

    DerivedVerb {
        u: Box<Word>,
        m: Box<Word>,
        a: Box<Word>,
    }, //Adverb modified Verb eg. +/
}

impl VerbImpl {
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word, JError> {
        match self {
            VerbImpl::Plus => v_plus(x, y),
            VerbImpl::Minus => v_minus(x, y),
            VerbImpl::Times => v_times(x, y),
            VerbImpl::Number => v_number(x, y),
            VerbImpl::Dollar => v_dollar(x, y),
            VerbImpl::NotImplemented => v_not_implemented(x, y),
            VerbImpl::DerivedVerb { u, m, a } => match (u.deref(), m.deref(), a.deref()) {
                (Verb(_, _), Nothing, Adverb(_, a)) => a.exec(x, u, y),
                (Nothing, Noun(_), Adverb(_, a)) => a.exec(x, m, y),
                _ => panic!("invalid DerivedVerb {:?}", self),
            },
        }
    }
}

fn promotion(x: &JArray, y: &JArray) -> Result<(JArray, JArray), JError> {
    // https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Automatic_Promotion_of_Argument_Precision
    match (x, y) {
        (BoolArray { a: x }, BoolArray { a: y }) => Ok((
            IntArray {
                a: x.map(|i| *i as i64),
            },
            IntArray {
                a: y.map(|i| *i as i64),
            },
        )),
        (BoolArray { a: x }, IntArray { a: y }) => Ok((
            IntArray {
                a: x.map(|i| *i as i64),
            },
            IntArray { a: y.clone() },
        )),
        (IntArray { a: x }, BoolArray { a: y }) => Ok((
            IntArray { a: x.clone() },
            IntArray {
                a: y.map(|i| *i as i64),
            },
        )),
        (BoolArray { a: x }, FloatArray { a: y }) => Ok((
            FloatArray {
                a: x.map(|i| *i as f64),
            },
            FloatArray { a: y.clone() },
        )),
        (FloatArray { a: x }, BoolArray { a: y }) => Ok((
            FloatArray { a: x.clone() },
            FloatArray {
                a: y.map(|i| *i as f64),
            },
        )),

        (IntArray { a: x }, FloatArray { a: y }) => Ok((
            FloatArray {
                a: x.map(|i| *i as f64),
            },
            FloatArray { a: y.clone() },
        )),
        (FloatArray { a: x }, IntArray { a: y }) => Ok((
            FloatArray { a: x.clone() },
            FloatArray {
                a: y.map(|i| *i as f64),
            },
        )),
        _ => Ok((x.clone(), y.clone())),
    }
}

pub fn v_not_implemented(_x: Option<&Word>, _y: &Word) -> Result<Word, JError> {
    Err(JError::NonceError)
}

pub fn v_plus(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError::custom("monadic + not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => match promotion(x, y) {
                Ok((IntArray { a: x }, IntArray { a: y })) => Ok(Word::Noun(IntArray { a: x + y })),
                Ok((ExtIntArray { a: x }, ExtIntArray { a: y })) => {
                    Ok(Word::Noun(ExtIntArray { a: x + y }))
                }
                Ok((FloatArray { a: x }, FloatArray { a: y })) => {
                    Ok(Word::Noun(FloatArray { a: x + y }))
                }
                Err(e) => Err(e),
                _ => Err(JError::DomainError),
            },
            _ => Err(JError::custom("plus not supported for these types yet")),
        },
    }
}

pub fn v_minus(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError::custom("monadic - not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => match promotion(x, y) {
                Ok((IntArray { a: x }, IntArray { a: y })) => Ok(Word::Noun(IntArray { a: x - y })),
                Ok((ExtIntArray { a: x }, ExtIntArray { a: y })) => {
                    Ok(Word::Noun(ExtIntArray { a: x - y }))
                }
                Ok((FloatArray { a: x }, FloatArray { a: y })) => {
                    Ok(Word::Noun(FloatArray { a: x - y }))
                }
                Err(e) => Err(e),
                _ => Err(JError::DomainError),
            },
            _ => Err(JError::custom("minus not supported for these types yet")),
        },
    }
}

pub fn v_times(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError::custom("monadic * not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => match promotion(x, y) {
                Ok((IntArray { a: x }, IntArray { a: y })) => Ok(Word::Noun(IntArray { a: x * y })),
                Ok((ExtIntArray { a: x }, ExtIntArray { a: y })) => {
                    Ok(Word::Noun(ExtIntArray { a: x * y }))
                }
                Ok((FloatArray { a: x }, FloatArray { a: y })) => {
                    Ok(Word::Noun(FloatArray { a: x * y }))
                }
                Err(e) => Err(e),
                _ => Err(JError::DomainError),
            },
            _ => Err(JError::custom("plus not supported for these types yet")),
        },
    }
}

pub fn v_number(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => {
            // Tally
            match y {
                Word::Noun(ja) => int_array([ja.len()].as_slice()),
                _ => Err(JError::DomainError),
            }
        }
        Some(_x) => Err(JError::custom("dyadic # not implemented yet")), // Copy
    }
}

pub fn v_dollar(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => {
            // Shape-of
            match y {
                Word::Noun(ja) => int_array(ja.shape()),
                _ => Err(JError::DomainError),
            }
        }
        Some(x) => {
            // Reshape
            match x {
                Word::Noun(IntArray { a: x }) => {
                    if x.product() < 0 {
                        Err(JError::DomainError)
                    } else {
                        match y {
                            Word::Noun(ja) => Ok(Word::Noun(map_array!(ja, |y| reshape(x, y)))),
                            _ => Err(JError::DomainError),
                        }
                    }
                }
                _ => Err(JError::DomainError),
            }
        }
    }
}

pub fn reshape<T>(x: &ArrayD<i64>, y: &ArrayD<T>) -> Result<ArrayD<T>, JError>
where
    T: Debug + Clone,
{
    if x.iter().product::<i64>() < 0 {
        Err(JError::DomainError)
    } else {
        // get shape of y cells
        // get new shape: concat x with sy
        // flatten y -> into_shape(ns)
        // TODO: This whole section should be x.outer_iter() and then
        // collected.
        let ns: Vec<usize> = x
            .iter()
            .map(|&i| i as usize)
            .chain(y.shape().iter().skip(1).copied())
            .collect();
        let flat_len = ns.iter().product();
        let flat_y = Array::from_iter(y.iter().cloned().cycle().take(flat_len));
        debug!("ns: {:?}, flat_y: {:?}", ns, flat_y);
        Ok(Array::from_shape_vec(IxDyn(&ns), flat_y.into_raw_vec()).unwrap())
    }
}
