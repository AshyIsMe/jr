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
    Star,
    Number,
    Percent,
    Dollar,
    StarCo,
    IDot,
    NotImplemented,

    //Adverb or Conjunction modified Verb eg. +/ or u^:n etc.
    //Modifiers take a left and right argument refered to as either
    //u and v if verbs or m and n if nouns (or combinations of either).
    DerivedVerb {
        l: Box<Word>,
        r: Box<Word>,
        m: Box<Word>,
    },
    Fork {
        f: Box<Word>,
        g: Box<Word>,
        h: Box<Word>,
    },
    Hook {
        l: Box<Word>,
        r: Box<Word>,
    },
}

impl VerbImpl {
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word, JError> {
        match self {
            VerbImpl::Plus => v_plus(x, y),
            VerbImpl::Minus => v_minus(x, y),
            VerbImpl::Star => v_star(x, y),
            VerbImpl::Number => v_number(x, y),
            VerbImpl::Percent => v_percent(x, y),
            VerbImpl::Dollar => v_dollar(x, y),
            VerbImpl::StarCo => v_starco(x, y),
            VerbImpl::IDot => v_idot(x, y),
            VerbImpl::NotImplemented => v_not_implemented(x, y),
            VerbImpl::DerivedVerb { l, r, m } => match (l.deref(), r.deref(), m.deref()) {
                (u @ Verb(_, _), Nothing, Adverb(_, a)) => a.exec(x, u, &Nothing, y),
                (m @ Noun(_), Nothing, Adverb(_, a)) => a.exec(x, m, &Nothing, y),
                (l, r, Conjunction(_, c))
                    if matches!(l, Noun(_) | Verb(_, _)) && matches!(r, Noun(_) | Verb(_, _)) =>
                {
                    c.exec(x, l, r, y)
                }
                _ => panic!("invalid DerivedVerb {:?}", self),
            },
            VerbImpl::Fork { f, g, h } => match (f.deref(), g.deref(), h.deref()) {
                (Verb(_, f), Verb(_, g), Verb(_, h)) => {
                    g.exec(Some(&f.exec(x, y).unwrap()), &h.exec(x, y).unwrap())
                }
                (Noun(m), Verb(_, g), Verb(_, h)) => {
                    g.exec(Some(&Noun(m.clone())), &h.exec(x, y).unwrap())
                }
                _ => panic!("invalid Fork {:?}", self),
            },
            VerbImpl::Hook { l, r } => match (l.deref(), r.deref()) {
                (Verb(_, u), Verb(_, v)) => match x {
                    None => u.exec(Some(&y), &v.exec(None, y).unwrap()),
                    Some(x) => u.exec(Some(&x), &v.exec(None, y).unwrap()),
                },
                _ => panic!("invalid Hook {:?}", self),
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

pub fn v_star(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
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
            _ => Err(JError::DomainError),
        },
    }
}

pub fn v_percent(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError::custom("monadic % not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => match promotion(x, y) {
                Ok((IntArray { a: x }, IntArray { a: y })) => Ok(Word::Noun(IntArray { a: x / y })),
                Ok((ExtIntArray { a: x }, ExtIntArray { a: y })) => {
                    Ok(Word::Noun(ExtIntArray { a: x / y }))
                }
                Ok((FloatArray { a: x }, FloatArray { a: y })) => {
                    Ok(Word::Noun(FloatArray { a: x / y }))
                }
                Err(e) => Err(e),
                _ => Err(JError::DomainError),
            },
            _ => Err(JError::DomainError),
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

pub fn v_starco(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => {
            // Square
            match y {
                Word::Noun(BoolArray { a }) => Ok(Word::Noun(BoolArray {
                    a: a.clone() * a.clone(),
                })),
                Word::Noun(IntArray { a }) => Ok(Word::Noun(IntArray {
                    a: a.clone() * a.clone(),
                })),
                Word::Noun(ExtIntArray { a }) => Ok(Word::Noun(ExtIntArray {
                    a: a.clone() * a.clone(),
                })),
                Word::Noun(FloatArray { a }) => Ok(Word::Noun(FloatArray {
                    a: a.clone() * a.clone(),
                })),
                _ => Err(JError::DomainError),
            }
        }
        Some(_x) => Err(JError::custom("dyadic # not implemented yet")), // Copy
    }
}

pub fn v_idot(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => match y {
            // monadic i.
            Word::Noun(IntArray { a }) => {
                let p = a.product();
                if p < 0 {
                    todo!("monadic i. negative args");
                } else {
                    let ints = Array::from_vec((0..p).collect());
                    Ok(Noun(IntArray {
                        a: reshape(a, &ints.into_dyn()).unwrap(),
                    }))
                }
            }
            Word::Noun(ExtIntArray { a: _ }) => {
                todo!("monadic i. ExtIntArray")
            }
            _ => Err(JError::DomainError),
        },
        Some(x) => match (x, y) {
            // dyadic i.
            (Word::Noun(x), Word::Noun(y)) => match (x, y) {
                // TODO remove code duplication: map_array!, apply_array_homo!, homo_array!, impl_array! ???
                (BoolArray { a: x }, BoolArray { a: y }) => {
                    let positions: Vec<i64> = y
                        .outer_iter()
                        .map(|i| {
                            x.outer_iter()
                                .position(|j| j == i)
                                .unwrap_or(x.len_of(Axis(0))) as i64
                        })
                        .collect();
                    Ok(Word::Noun(IntArray {
                        a: Array::from_shape_vec(IxDyn(&[positions.len()]), positions).unwrap(),
                    }))
                }
                (CharArray { a: x }, CharArray { a: y }) => {
                    let positions: Vec<i64> = y
                        .outer_iter()
                        .map(|i| {
                            x.outer_iter()
                                .position(|j| j == i)
                                .unwrap_or(x.len_of(Axis(0))) as i64
                        })
                        .collect();
                    Ok(Word::Noun(IntArray {
                        a: Array::from_shape_vec(IxDyn(&[positions.len()]), positions).unwrap(),
                    }))
                }
                (IntArray { a: x }, IntArray { a: y }) => {
                    let positions: Vec<i64> = y
                        .outer_iter()
                        .map(|i| {
                            x.outer_iter()
                                .position(|j| j == i)
                                .unwrap_or(x.len_of(Axis(0))) as i64
                        })
                        .collect();
                    Ok(Word::Noun(IntArray {
                        a: Array::from_shape_vec(IxDyn(&[positions.len()]), positions).unwrap(),
                    }))
                }
                (ExtIntArray { a: x }, ExtIntArray { a: y }) => {
                    let positions: Vec<i64> = y
                        .outer_iter()
                        .map(|i| {
                            x.outer_iter()
                                .position(|j| j == i)
                                .unwrap_or(x.len_of(Axis(0))) as i64
                        })
                        .collect();
                    Ok(Word::Noun(IntArray {
                        a: Array::from_shape_vec(IxDyn(&[positions.len()]), positions).unwrap(),
                    }))
                }
                (FloatArray { a: x }, FloatArray { a: y }) => {
                    let positions: Vec<i64> = y
                        .outer_iter()
                        .map(|i| {
                            x.outer_iter()
                                .position(|j| j == i)
                                .unwrap_or(x.len_of(Axis(0))) as i64
                        })
                        .collect();
                    Ok(Word::Noun(IntArray {
                        a: Array::from_shape_vec(IxDyn(&[positions.len()]), positions).unwrap(),
                    }))
                }
                _ => {
                    // mismatched array types
                    let xl = x.len_of(Axis(0)) as i64;
                    let yl = y.len_of(Axis(0));
                    Ok(Word::Noun(IntArray {
                        a: Array::from_elem(IxDyn(&[yl]), xl),
                    }))
                }
            },
            _ => Err(JError::DomainError),
        },
    }
}
