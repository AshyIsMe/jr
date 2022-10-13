use ndarray::prelude::*;
use ndarray::{concatenate, Axis};
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;

use crate::impl_array;
use crate::Word;
use crate::{ArrayPair, JError};
use crate::{IntoJArray, JArray};

use anyhow::{anyhow, Context, Result};
use log::debug;

use JArray::*;
use Word::*;

#[derive(Clone)]
pub struct SimpleImpl {
    name: &'static str,
    monad: fn(&Word) -> Result<Word>,
    dyad: Option<fn(&Word, &Word) -> Result<Word>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Simple(SimpleImpl),

    Plus,
    Minus,
    Star,
    Number,
    Percent,
    Dollar,
    StarCo,
    IDot,
    Plot,
    LT,
    GT,
    Semi,
    NotImplemented(String),

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
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word> {
        match self {
            VerbImpl::Simple(imp) => match x {
                None => (imp.monad)(y),
                Some(x) => imp.dyad.ok_or_else(|| JError::DomainError)?(x, y),
            },
            VerbImpl::Plus => v_plus(x, y),
            VerbImpl::Minus => v_minus(x, y),
            VerbImpl::Star => v_star(x, y),
            VerbImpl::Number => v_number(x, y),
            VerbImpl::Percent => v_percent(x, y),
            VerbImpl::Dollar => v_dollar(x, y),
            VerbImpl::StarCo => v_starco(x, y),
            VerbImpl::IDot => v_idot(x, y),
            VerbImpl::Plot => v_plot(x, y),
            VerbImpl::LT => v_lt(x, y),
            VerbImpl::GT => v_gt(x, y),
            VerbImpl::Semi => v_semi(x, y),
            VerbImpl::NotImplemented(hint) => {
                v_not_implemented(x, y).with_context(|| anyhow!("verb {:?} not supported", hint))
            }
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
                    g.exec(Some(&f.exec(x, y)?), &h.exec(x, y)?)
                }
                (Noun(m), Verb(_, g), Verb(_, h)) => g.exec(Some(&Noun(m.clone())), &h.exec(x, y)?),
                _ => panic!("invalid Fork {:?}", self),
            },
            VerbImpl::Hook { l, r } => match (l.deref(), r.deref()) {
                (Verb(_, u), Verb(_, v)) => match x {
                    None => u.exec(Some(&y), &v.exec(None, y)?),
                    Some(x) => u.exec(Some(&x), &v.exec(None, y)?),
                },
                _ => panic!("invalid Hook {:?}", self),
            },
        }
    }
}

impl fmt::Debug for SimpleImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleImpl({})", self.name)
    }
}

impl PartialEq for SimpleImpl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

fn prohomo<'l, 'r>(x: &'l JArray, y: &'r JArray) -> Result<ArrayPair<'l, 'r>> {
    //promote_homogenous:
    //https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Automatic_Promotion_of_Argument_Precision
    use ArrayPair::*;
    Ok(match (x, y) {
        (BoolArray(x), BoolArray(y)) => IntPair(x.cast()?.into(), y.cast()?.into()),
        (BoolArray(x), IntArray(y)) => IntPair(x.cast()?.into(), y.into()),
        (IntArray(x), BoolArray(y)) => IntPair(x.into(), y.cast()?.into()),
        (BoolArray(x), FloatArray(y)) => FloatPair(x.cast()?.into(), y.into()),
        (FloatArray(x), BoolArray(y)) => FloatPair(x.into(), y.cast()?.into()),

        (IntArray(x), FloatArray(y)) => FloatPair(x.map(|i| *i as f64).into(), y.into()),
        (FloatArray(x), IntArray(y)) => FloatPair(x.into(), y.map(|i| *i as f64).into()),

        (CharArray(x), CharArray(y)) => {
            IntPair(x.map(|&i| i as i64).into(), y.map(|&i| i as i64).into())
        }
        (IntArray(x), IntArray(y)) => IntPair(x.into(), y.into()),
        (ExtIntArray(x), ExtIntArray(y)) => ExtIntPair(x.into(), y.into()),
        (FloatArray(x), FloatArray(y)) => FloatPair(x.into(), y.into()),
        _ => return Err(JError::DomainError).with_context(|| anyhow!("{x:?} {y:?}")),
    })
}

pub trait ArrayUtil<A> {
    fn cast<T: From<A>>(&self) -> Result<ArrayD<T>>;
}

impl<A: Copy> ArrayUtil<A> for ArrayD<A> {
    fn cast<T: From<A>>(&self) -> Result<ArrayD<T>> {
        Ok(self.map(|&e| T::try_from(e).expect("todo: LimitError?")))
    }
}

pub fn v_not_implemented(_x: Option<&Word>, _y: &Word) -> Result<Word> {
    Err(JError::NonceError.into())
}

pub fn v_plus(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => Err(JError::custom("monadic + not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => Ok(Word::Noun(prohomo(x, y)?.plus())),
            _ => Err(JError::custom("plus not supported for these types yet")),
        },
    }
}

pub fn v_minus(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => Err(JError::custom("monadic - not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => Ok(Word::Noun(prohomo(x, y)?.minus())),
            _ => Err(JError::custom("minus not supported for these types yet")),
        },
    }
}

pub fn v_star(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => Err(JError::custom("monadic * not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => Ok(Word::Noun(prohomo(x, y)?.star())),
            _ => Err(JError::DomainError.into()),
        },
    }
}

pub fn v_percent(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => Err(JError::custom("monadic % not implemented yet")),
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => Ok(Word::Noun(prohomo(x, y)?.slash())),
            _ => Err(JError::DomainError.into()),
        },
    }
}

pub fn v_number(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => {
            // Tally
            match y {
                Word::Noun(ja) => {
                    Word::noun([i64::try_from(ja.len()).map_err(|_| JError::LimitError)?])
                }
                _ => Err(JError::DomainError.into()),
            }
        }
        Some(_x) => Err(JError::custom("dyadic # not implemented yet")), // Copy
    }
}

pub fn v_dollar(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => {
            // Shape-of
            match y {
                Word::Noun(ja) => Word::noun(ja.shape()),
                _ => Err(JError::DomainError.into()),
            }
        }
        Some(x) => {
            // Reshape
            match x {
                Word::Noun(IntArray(x)) => {
                    if x.product() < 0 {
                        Err(JError::DomainError.into())
                    } else {
                        match y {
                            Word::Noun(ja) => {
                                impl_array!(ja, |y| reshape(x, y).map(|x| x.into_noun()))
                            }
                            _ => Err(JError::DomainError.into()),
                        }
                    }
                }
                _ => Err(JError::DomainError.into()),
            }
        }
    }
}

pub fn reshape<T>(x: &ArrayD<i64>, y: &ArrayD<T>) -> Result<ArrayD<T>>
where
    T: Debug + Clone,
{
    if x.iter().product::<i64>() < 0 {
        Err(JError::DomainError.into())
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
        Ok(Array::from_shape_vec(IxDyn(&ns), flat_y.into_raw_vec())?)
    }
}

pub fn v_starco(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => {
            // Square
            match y {
                Word::Noun(BoolArray(a)) => Ok(Word::Noun(BoolArray(a.clone() * a.clone()))),
                Word::Noun(IntArray(a)) => Ok(Word::Noun(IntArray(a.clone() * a.clone()))),
                Word::Noun(ExtIntArray(a)) => Ok(Word::Noun(ExtIntArray(a.clone() * a.clone()))),
                Word::Noun(FloatArray(a)) => Ok(Word::Noun(FloatArray(a.clone() * a.clone()))),
                _ => Err(JError::DomainError.into()),
            }
        }
        Some(_x) => Err(JError::custom("dyadic # not implemented yet")), // Copy
    }
}

pub fn v_idot(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => match y {
            // monadic i.
            Word::Noun(IntArray(a)) => {
                let p = a.product();
                if p < 0 {
                    todo!("monadic i. negative args");
                } else {
                    let ints = Array::from_vec((0..p).collect());
                    Ok(Noun(IntArray(reshape(a, &ints.into_dyn())?)))
                }
            }
            Word::Noun(ExtIntArray(_)) => {
                todo!("monadic i. ExtIntArray")
            }
            _ => Err(JError::DomainError.into()),
        },
        Some(x) => match (x, y) {
            // TODO fix for n-dimensional arguments. currently broken
            // dyadic i.
            (Word::Noun(x), Word::Noun(y)) => match (x, y) {
                // TODO remove code duplication: impl_array_pair!? impl_array_binary!?
                (BoolArray(x), BoolArray(y)) => v_idot_positions(x, y),
                (CharArray(x), CharArray(y)) => v_idot_positions(x, y),
                (IntArray(x), IntArray(y)) => v_idot_positions(x, y),
                (ExtIntArray(x), ExtIntArray(y)) => v_idot_positions(x, y),
                (FloatArray(x), FloatArray(y)) => v_idot_positions(x, y),
                _ => {
                    // mismatched array types
                    let xl = x.len_of(Axis(0)) as i64;
                    let yl = y.len_of(Axis(0));
                    Ok(Word::Noun(IntArray(Array::from_elem(IxDyn(&[yl]), xl))))
                }
            },
            _ => Err(JError::DomainError.into()),
        },
    }
}

pub fn v_plot(x: Option<&Word>, y: &Word) -> Result<Word> {
    match (x, y) {
        #[allow(unused_variables)]
        (None, Word::Noun(y)) => {
            cfg_if::cfg_if! {
                if #[cfg(feature = "ui")] {
                    crate::plot::plot(y)
                } else {
                    Err(JError::NonceError.into())
                }
            }
        }
        _ => Err(JError::NonceError.into()),
    }
}

fn v_idot_positions<T: PartialEq>(x: &ArrayD<T>, y: &ArrayD<T>) -> Result<Word> {
    Word::noun(
        y.outer_iter()
            .map(|i| {
                x.outer_iter()
                    .position(|j| j == i)
                    .unwrap_or(x.len_of(Axis(0))) as i64
            })
            .collect::<Vec<i64>>(),
    )
}

pub fn v_lt(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => match y {
            Noun(y) => Word::noun([Noun(y.clone())]),
            _ => return Err(JError::DomainError.into()),
        },
        Some(x) => match (x, y) {
            (Word::Noun(x), Word::Noun(y)) => Ok(Word::Noun(prohomo(x, y)?.lessthan())),
            _ => panic!("invalid types v_lt({:?}, {:?})", x, y),
        },
    }
}

pub fn v_gt(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        None => match y {
            Noun(BoxArray(y)) => match y.len() {
                1 => Ok(y[0].clone()),
                _ => todo!("unbox BoxArray"),
            },
            Noun(y) => Ok(Noun(y.clone())),
            _ => return Err(JError::DomainError.into()),
        },
        Some(x) => match (x, y) {
            //(Word::Noun(x), Word::Noun(y)) => Ok(Word::Noun(prohomo(x, y)?.greaterthan())),
            _ => Err(JError::custom("dyadic > not implemented yet")),
            //_ => panic!("invalid types v_gt({:?}, {:?})", x, y),
        },
    }
}

pub fn v_semi(x: Option<&Word>, y: &Word) -> Result<Word> {
    match x {
        // raze
        None => Err(JError::custom("monadic ; not implemented yet")),
        Some(x) => match (x, y) {
            // link: https://code.jsoftware.com/wiki/Vocabulary/semi#dyadic
            // always box x, only box y if not already boxed
            (Noun(x), Noun(BoxArray(y))) => match Word::noun([Noun(x.clone())]).unwrap() {
                Noun(BoxArray(x)) => {
                    Ok(Word::noun(concatenate(Axis(0), &[x.view(), y.view()]).unwrap()).unwrap())
                }
                _ => panic!("invalid types v_semi({:?}, {:?})", x, y),
            },
            (Noun(x), Noun(y)) => Ok(Word::noun([Noun(x.clone()), Noun(y.clone())]).unwrap()),
            _ => panic!("invalid types v_semi({:?}, {:?})", x, y),
        },
    }
}
