mod ranks;

use core::cmp::{max, min};
use std::fmt;
use std::fmt::Debug;
use std::iter::repeat;
use std::iter::zip;
use std::ops::Deref;

use crate::impl_array;
use crate::Word;
use crate::{ArrayPair, JError};
use crate::{IntoJArray, JArray};

use anyhow::{anyhow, bail, Context, Result};
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};

use crate::cells::{apply_cells, flatten, generate_cells, result_shape};
use crate::JError::DomainError;
use JArray::*;
use Word::*;

pub use ranks::Rank;

#[derive(Copy, Clone)]
pub struct Monad {
    // TODO: NOT public
    pub f: fn(&JArray) -> Result<Word>,
    pub rank: Rank,
}

#[derive(Copy, Clone)]
pub struct Dyad {
    // TODO: NOT public
    pub f: fn(&JArray, &JArray) -> Result<Word>,
    pub rank: (Rank, Rank),
}

#[derive(Copy, Clone)]
pub struct PrimitiveImpl {
    // TODO: NOT public
    pub name: &'static str,
    // TODO: NOT public
    pub monad: Monad,
    // TODO: NOT public
    pub dyad: Option<Dyad>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Primitive(PrimitiveImpl),

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

fn exec_dyad(dyad: &Dyad, x: &JArray, y: &JArray) -> Result<Word> {
    if Rank::infinite_infinite() == dyad.rank {
        return (dyad.f)(x, y).context("infinite dyad shortcut");
    }
    let target_shape = result_shape(x, y);
    let (x_cells, y_cells) = generate_cells(x, y, dyad.rank).context("generating cells")?;
    let application_result =
        apply_cells((&x_cells, &y_cells), dyad.f).context("applying function to cells")?;

    let flat = flatten(target_shape, &application_result).with_context(|| {
        // this is expensive but should only be hit on application bugs, not user code issues
        let pair_info = application_result
            .iter()
            .map(|w| match w {
                Word::Noun(n) => Some(n.shape().to_vec()),
                _ => None,
            })
            .collect::<Option<Vec<_>>>();

        anyhow!("reshaping {:?} to {target_shape:?}", pair_info)
    })?;
    Ok(Word::Noun(flat))
}

impl VerbImpl {
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word> {
        match self {
            VerbImpl::Primitive(imp) => match (x, y) {
                (None, Word::Noun(y)) => {
                    (imp.monad.f)(y).with_context(|| anyhow!("monadic {:?}", imp.name))
                }
                (Some(Word::Noun(x)), Word::Noun(y)) => {
                    let dyad = imp
                        .dyad
                        .ok_or(JError::DomainError)
                        .with_context(|| anyhow!("there is no dyadic {:?}", imp.name))?;
                    exec_dyad(&dyad, x, y).with_context(|| anyhow!("dyadic {:?}", imp.name))
                }
                _ => Err(DomainError.into()),
            },
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

impl PrimitiveImpl {
    pub fn monad(name: &'static str, f: fn(&JArray) -> Result<Word>) -> Self {
        Self {
            name,
            monad: Monad {
                f,
                rank: Rank::infinite(),
            },
            dyad: None,
        }
    }

    pub const fn new(
        name: &'static str,
        monad: fn(&JArray) -> Result<Word>,
        dyad: fn(&JArray, &JArray) -> Result<Word>,
        ranks: (Rank, Rank, Rank),
    ) -> Self {
        Self {
            name,
            monad: Monad {
                f: monad,
                rank: ranks.0,
            },
            dyad: Some(Dyad {
                f: dyad,
                rank: (ranks.1, ranks.2),
            }),
        }
    }
}

impl fmt::Debug for PrimitiveImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrimitiveImpl({})", self.name)
    }
}

impl PartialEq for PrimitiveImpl {
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

pub fn check_agreement(x: Word, y: Word, ranks: [usize; 2]) -> Result<bool> {
    // https://code.jsoftware.com/wiki/Vocabulary/Agreement
    // [x] Make it work.
    // [ ] Make it correct. (slightly closer to correct)
    // [ ] Make it fast<C-w>clean.

    match (x.clone(), y.clone()) {
        (Noun(x), Noun(y)) => {
            let x_shape = x.shape();
            let y_shape = y.shape();

            // (_1 * ({.ranks)) }. $ x
            let x_frame: Vec<&usize> = if (x_shape.len() - ranks[0]) > 0 {
                x_shape[0..x_shape.len() - ranks[0]].iter().collect()
            } else {
                Vec::new() // empty frame
            };
            // (_1 * ({:ranks)) }. $ y
            let y_frame: Vec<&usize> = if (y_shape.len() - ranks[1]) > 0 {
                y_shape[0..y_shape.len() - ranks[1]].iter().collect()
            } else {
                Vec::new() // empty frame
            };

            println!("x_frame: {:?}, y_frame: {:?}", x_frame, y_frame);

            // The frames of x and y must start identically,
            // and must match identically for the entire length of the shorter frame.
            let checks = zip(x_frame.clone(), y_frame.clone())
                .map(|t| if t.0 == t.1 { 1 } else { 0 })
                .collect::<Vec<usize>>();

            let shortest_frame_len = min(x_frame.len(), y_frame.len());
            let common_frame_len = match checks.iter().enumerate().find(|(_, i)| **i == 0usize) {
                Some((_, i)) => *i,
                _ => shortest_frame_len,
            };
            println!(
                "shortest_frame_len: {:?}, common_frame_len: {:?}",
                shortest_frame_len, common_frame_len
            );

            //AA TODO - this is only the first macrocell check
            // https://code.jsoftware.com/wiki/Vocabulary/Agreement#Agreement:_Macrocells_Of_x_and_y_Are_Matched
            // "The frames of x and y must start identically, and must match identically for the entire length
            // of the shorter frame. If they don't, the result is a length error which is signaled before the verb is executed on any cells."
            Ok(common_frame_len == shortest_frame_len)
            // AA TODO - check cell matching within macrocells
            // https://code.jsoftware.com/wiki/Vocabulary/Agreement#Cells_Are_Matched_Within_Macrocells
        }
        _ => Err(JError::DomainError).with_context(|| anyhow!("{x:?} {y:?}")),
    }
}

// AA TODO: The idea is that this returns a Vec<(x,y)> of macrocells that can be
// mapped over in VerbImpl.exec() as per the doco here:
// https://code.jsoftware.com/wiki/Vocabulary/Agreement#The_Verb_Is_Executed_And_The_Result_Collected
pub fn args_to_macrocells(x: Word, y: Word, ranks: [usize; 2]) -> Result<Vec<(JArray, JArray)>> {
    match (x.clone(), y.clone()) {
        (Noun(x), Noun(y)) => {
            println!("x:\n{}\ny:\n{}", x, y);
            println!("ranks: [{}, {}]\n", ranks[0], ranks[1]);

            let x_shape = x.shape();
            let y_shape = y.shape();

            // (_1 * ({.ranks)) }. $ x
            let x_frame: Vec<&usize> = if (x_shape.len() - ranks[0]) > 0 {
                x_shape[0..x_shape.len() - ranks[0]].iter().collect()
            } else {
                Vec::new() // empty frame
            };
            // (_1 * ({:ranks)) }. $ y
            let y_frame: Vec<&usize> = if (y_shape.len() - ranks[1]) > 0 {
                y_shape[0..y_shape.len() - ranks[1]].iter().collect()
            } else {
                Vec::new() // empty frame
            };
            println!("x_frame: {:?}, y_frame: {:?}", x_frame, y_frame);

            // The frames of x and y must start identically,
            // and must match identically for the entire length of the shorter frame.
            let checks = zip(x_frame.clone(), y_frame.clone())
                .map(|t| if t.0 == t.1 { 1 } else { 0 })
                .collect::<Vec<usize>>();

            let shortest_frame_len = min(x_frame.len(), y_frame.len());
            let common_frame_len = match checks.iter().enumerate().find(|(_, i)| **i == 0usize) {
                Some((_, i)) => *i,
                _ => shortest_frame_len,
            };

            println!(
                "shortest_frame_len: {:?}, common_frame_len: {:?}",
                shortest_frame_len, common_frame_len
            );

            let common_frame = &x_frame[0..common_frame_len]; // grab from either x_frame or y_frame
            let x_surplus_frame = &x_frame[common_frame_len..];
            let y_surplus_frame = &y_frame[common_frame_len..];
            println!("common_frame: {:?}", common_frame);
            println!(
                "x_surplus_frame: {:?}, y_surplus_frame: {:?}",
                x_surplus_frame, y_surplus_frame
            );

            if common_frame_len != shortest_frame_len {
                println!("bailing common_frame_len != shortest_frame_len");
                bail!(JError::LengthError)
            } else {
                // "The macrocells for the argument with the shorter frame will be cells of that argument,
                // while the macrocells of the operand with longer frame will be arrays of cells.
                // If the frames are identical, the macrocells are simply the cells of the arguments. "

                // AA TODO: macrocells below are not quite right:
                // calculate macrocells of x and y
                let x_macrocells = if x_frame.len() <= y_frame.len() || ranks[0] == 0 {
                    //x.to_cells(x.shape().len() - 1).unwrap()
                    x.to_cells(max(0, x.shape().len() as i64 - 1) as usize)
                        .unwrap()
                } else {
                    x.to_cells(ranks[0]).unwrap()
                };
                let y_macrocells = if y_frame.len() <= x_frame.len() || ranks[1] == 0 {
                    //y.to_cells(y.shape().len() - 1).unwrap()
                    y.to_cells(max(0, y.shape().len() as i64 - 1) as usize)
                        .unwrap()
                } else {
                    y.to_cells(ranks[1]).unwrap()
                };

                //let y_macrocells = y.to_cells(ranks[1]).unwrap();
                println!("x_macrocells:");
                for c in x_macrocells.iter() {
                    println!("{}", c);
                }
                println!("y_macrocells:");
                for c in y_macrocells.iter() {
                    println!("{}", c);
                }

                if ranks[0] < ranks[1] {
                    // repeat y macrocells
                    let mut v: Vec<(JArray, JArray)> = Vec::new();
                    for (i, xc) in x_macrocells.iter().enumerate() {
                        for (j, xi) in xc
                            .to_cells(max(0, xc.shape().len() as i64 - 1) as usize)
                            .unwrap()
                            .iter()
                            .enumerate()
                        {
                            //each xi with y_macrocells[i]
                            v.push((xi.clone(), y_macrocells[i].clone()))
                        }
                    }
                    Ok(v)
                } else if ranks[0] > ranks[1] {
                    // repeat x macrocells
                    let mut v: Vec<(JArray, JArray)> = Vec::new();
                    for (i, yc) in y_macrocells.iter().enumerate() {
                        for (j, yi) in yc
                            .to_cells(max(0, yc.shape().len() as i64 - 1) as usize)
                            .unwrap()
                            .iter()
                            .enumerate()
                        {
                            //each xi with y_macrocells[i]
                            v.push((x_macrocells[i].clone(), yi.clone()))
                        }
                    }
                    Ok(v)
                } else {
                    // match macrocells evenly
                    if x_surplus_frame.len() < y_surplus_frame.len() {
                        // repeat x macrocells
                        let mut v: Vec<(JArray, JArray)> = Vec::new();
                        let mut x_iter = x_macrocells.iter().cycle();
                        for (i, yc) in y_macrocells.iter().enumerate() {
                            for (j, yi) in yc
                                .to_cells(max(0, yc.shape().len() as i64 - 1) as usize)
                                .unwrap()
                                .iter()
                                .enumerate()
                            {
                                //each xi with y_macrocells[i]
                                //v.push((x_macrocells[i].clone(), yi.clone()))
                                v.push((x_iter.next().unwrap().clone(), yi.clone()))
                            }
                        }
                        Ok(v)
                    } else if x_surplus_frame.len() > y_surplus_frame.len() {
                        // repeat y macrocells
                        let mut v: Vec<(JArray, JArray)> = Vec::new();
                        let mut y_iter = y_macrocells.iter().cycle();
                        for (i, xc) in x_macrocells.iter().enumerate() {
                            for (j, xi) in xc
                                .to_cells(max(0, xc.shape().len() as i64 - 1) as usize)
                                .unwrap()
                                .iter()
                                .enumerate()
                            {
                                //each xi with y_macrocells[i]
                                //v.push((xi.clone(), y_macrocells[i].clone()))
                                v.push((xi.clone(), y_iter.next().unwrap().clone()));
                            }
                        }
                        Ok(v)
                    } else {
                        Ok(zip(x_macrocells.into_iter(), y_macrocells.into_iter()).collect())
                    }
                }
            }
        }
        _ => Err(JError::DomainError).with_context(|| anyhow!("{x:?} {y:?}")),
    }
}

pub trait ArrayUtil<A> {
    fn cast<T: From<A>>(&self) -> Result<ArrayD<T>>;
}

impl<A: Copy> ArrayUtil<A> for ArrayD<A> {
    fn cast<T: From<A>>(&self) -> Result<ArrayD<T>> {
        Ok(self.map(|&e| T::try_from(e).expect("todo: LimitError?")))
    }
}

pub fn v_not_implemented_monad(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

pub fn v_not_implemented_dyad(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
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

#[allow(unused_variables)]
pub fn v_plot(y: &JArray) -> Result<Word> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ui")] {
            crate::plot::plot(y)
        } else {
            Err(JError::NonceError.into())
        }
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

// (echo '<table>'; <~/Downloads/Vocabulary.html fgrep '&#149;' | sed 's/<td nowrap>/<tr><td>/g') > a.html; links -dump a.html | perl -ne 's/\s*$/\n/; my ($a,$b,$c) = $_ =~ /\s+([^\s]+) (.*?) \xc2\x95 (.+?)$/; $b =~ tr/A-Z/a-z/; $c =~ tr/A-Z/a-z/; $b =~ s/[^a-z ]//g; $c =~ s/[^a-z -]//g; $b =~ s/ +|-/_/g; $c =~ s/ +|-/_/g; print "/// $a (monad)\npub fn v_$b(y: &Word) -> Result<Word> { Err(JError::NonceError.into()) }\n/// $a (dyad)\npub fn v_$c(x: &Word, y: &Word) -> Result<Word> { Err(JError::NonceError.into()) }\n\n"'

/// = (monad)
pub fn v_self_classify(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// = (dyad)
pub fn v_equal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// < (monad)
pub fn v_box(y: &JArray) -> Result<Word> {
    Word::noun([Noun(y.clone())])
}
/// < (dyad)
pub fn v_less_than(x: &JArray, y: &JArray) -> Result<Word> {
    Ok(Word::Noun(prohomo(x, y)?.lessthan()))
}

/// <. (monad)
pub fn v_floor(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// <. (dyad)
pub fn v_lesser_of_min(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// <: (monad)
pub fn v_decrement(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// <: (dyad)
pub fn v_less_or_equal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// > (monad)
pub fn v_open(y: &JArray) -> Result<Word> {
    match y {
        BoxArray(y) => match y.len() {
            1 => Ok(y[0].clone()),
            _ => bail!("todo: unbox BoxArray"),
        },
        y => Ok(Noun(y.clone())),
    }
}
/// > (dyad)
pub fn v_larger_than(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// >. (monad)
pub fn v_ceiling(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// >. (dyad)
pub fn v_larger_of_max(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// >: (monad)
pub fn v_increment(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// >: (dyad)
pub fn v_larger_or_equal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// + (monad)
pub fn v_conjugate(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// + (dyad)
pub fn v_plus(x: &JArray, y: &JArray) -> Result<Word> {
    debug!("executing plus on {x:?} + {y:?}");
    Ok(Word::Noun(prohomo(x, y)?.plus()))
}

/// +. (monad)
pub fn v_real_imaginary(y: &JArray) -> Result<Word> {
    match y.to_c64() {
        Some(y) => {
            // y.insert_axis() ...
            let mut shape = y.shape().to_vec();
            shape.push(2);
            let values = y.iter().flat_map(|x| [x.re, x.im]).collect();
            Ok(ArrayD::from_shape_vec(shape, values)?.into_noun())
        }
        None => Err(JError::DomainError.into()),
    }
}
/// +. (dyad)
pub fn v_gcd_or(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// +: (monad)
pub fn v_double(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// +: (dyad)
pub fn v_not_or(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// * (monad)
pub fn v_signum(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// * (dyad)
pub fn v_times(x: &JArray, y: &JArray) -> Result<Word> {
    Ok(Word::Noun(prohomo(x, y)?.star()))
}

/// *. (monad)
pub fn v_lengthangle(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// *. (dyad)
pub fn v_lcm_and(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// *: (monad)
pub fn v_square(y: &JArray) -> Result<Word> {
    match y {
        BoolArray(a) => Ok(Word::Noun(BoolArray(a.clone() * a.clone()))),
        IntArray(a) => Ok(Word::Noun(IntArray(a.clone() * a.clone()))),
        ExtIntArray(a) => Ok(Word::Noun(ExtIntArray(a.clone() * a.clone()))),
        FloatArray(a) => Ok(Word::Noun(FloatArray(a.clone() * a.clone()))),
        _ => Err(JError::DomainError.into()),
    }
}
/// *: (dyad)
pub fn v_not_and(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// - (monad)
pub fn v_negate(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// - (dyad)
pub fn v_minus(x: &JArray, y: &JArray) -> Result<Word> {
    Ok(Word::Noun(prohomo(x, y)?.minus()))
}

/// -. (monad)
pub fn v_not(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// -. (dyad)
pub fn v_less(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// -: (monad)
pub fn v_halve(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// -: (dyad)
pub fn v_match(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// % (monad)
pub fn v_reciprocal(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// % (dyad)
pub fn v_divide(x: &JArray, y: &JArray) -> Result<Word> {
    Ok(Word::Noun(prohomo(x, y)?.slash()))
}

/// %. (monad)
pub fn v_matrix_inverse(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// %. (dyad)
pub fn v_matrix_divide(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// %: (monad)
pub fn v_square_root(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// %: (dyad)
pub fn v_root(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ^ (monad)
pub fn v_exponential(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ^ (dyad)
pub fn v_power(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ^. (monad)
pub fn v_natural_log(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ^. (dyad)
pub fn v_logarithm(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// $ (monad)
pub fn v_shape_of(y: &JArray) -> Result<Word> {
    Word::noun(y.shape())
}
/// $ (dyad)
pub fn v_shape(x: &JArray, y: &JArray) -> Result<Word> {
    match x {
        IntArray(x) => {
            if x.product() < 0 {
                Err(JError::DomainError.into())
            } else {
                impl_array!(y, |y| reshape(x, y).map(|x| x.into_noun()))
            }
        }
        _ => Err(JError::DomainError.into()),
    }
}

/// ~: (monad)
pub fn v_nub_sieve(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ~: (dyad)
pub fn v_not_equal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// | (monad)
pub fn v_magnitude(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// | (dyad)
pub fn v_residue(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// |. (monad)
pub fn v_reverse(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// |. (dyad)
pub fn v_rotate_shift(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// , (monad)
pub fn v_ravel(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// , (dyad)
pub fn v_append(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ,. (monad)
pub fn v_ravel_items(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ,. (dyad)
pub fn v_stitch(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ,: (monad)
pub fn v_itemize(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ,: (dyad)
pub fn v_laminate(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ; (monad)
pub fn v_raze(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ; (dyad)
pub fn v_link(x: &JArray, y: &JArray) -> Result<Word> {
    match (x, y) {
        // link: https://code.jsoftware.com/wiki/Vocabulary/semi#dyadic
        // always box x, only box y if not already boxed
        (x, BoxArray(y)) => match Word::noun([Noun(x.clone())]).unwrap() {
            Noun(BoxArray(x)) => {
                Ok(Word::noun(concatenate(Axis(0), &[x.view(), y.view()]).unwrap()).unwrap())
            }
            _ => panic!("invalid types v_semi({:?}, {:?})", x, y),
        },
        (x, y) => Ok(Word::noun([Noun(x.clone()), Noun(y.clone())]).unwrap()),
    }
}

/// ;: (monad)
pub fn v_words(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ;: (dyad)
pub fn v_sequential_machine(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// # (monad)
pub fn v_tally(y: &JArray) -> Result<Word> {
    Word::noun([i64::try_from(y.len()).map_err(|_| JError::LimitError)?])
}
/// # (dyad)
pub fn v_copy(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// #. (monad)
pub fn v_base_(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// #. (dyad)
pub fn v_base(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// #: (monad)
pub fn v_antibase_(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// #: (dyad)
pub fn v_antibase(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ! (monad)
pub fn v_factorial(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ! (dyad)
pub fn v_out_of(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// /: (monad)
pub fn v_grade_up(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// /: (dyad) and \: (dyad)
pub fn v_sort(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// \: (monad)
pub fn v_grade_down(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// \[ (monad) and ] (monad) apparently
pub fn v_same(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// [ (dyad)
pub fn v_left(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ] (dyad)
pub fn v_right(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// { (monad)
pub fn v_catalogue(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// { (dyad)
pub fn v_from(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// {. (monad)
pub fn v_head(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// {. (dyad)
pub fn v_take(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// {: (monad)
pub fn v_tail(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// {:: (monad)
pub fn v_map(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// {:: (dyad)
pub fn v_fetch(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// }. (monad)
pub fn v_behead(y: &JArray) -> Result<Word> {
    impl_array!(y, |arr: &ArrayD<_>| Ok(arr
        .slice_axis(Axis(0), Slice::from(1isize..))
        .into_owned()
        .into_noun()))
}
/// }. (dyad)
pub fn v_drop(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ". (monad)
pub fn v_do(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ". (dyad)
pub fn v_numbers(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ": (monad)
pub fn v_default_format(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ": (dyad)
pub fn v_format(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ? (monad)
pub fn v_roll(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ? (dyad)
pub fn v_deal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ?. (dyad)
pub fn v_deal_fixed_seed(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// A. (monad)
pub fn v_anagram_index(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// A. (dyad)
pub fn v_anagram(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// C. (monad)
pub fn v_cycledirect(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// C. (dyad)
pub fn v_permute(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// e. (monad)
pub fn v_raze_in(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// e. (dyad)
pub fn v_member_in(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// i. (monad)
pub fn v_integers(y: &JArray) -> Result<Word> {
    match y {
        // monadic i.
        IntArray(a) => {
            let p = a.product();
            if p < 0 {
                bail!("todo: monadic i. negative args");
            } else {
                let ints = Array::from_vec((0..p).collect());
                Ok(Noun(IntArray(reshape(a, &ints.into_dyn())?)))
            }
        }
        ExtIntArray(_) => {
            bail!("todo: monadic i. ExtIntArray")
        }
        _ => Err(JError::DomainError.into()),
    }
}
/// i. (dyad)
pub fn v_index_of(x: &JArray, y: &JArray) -> Result<Word> {
    match (x, y) {
        // TODO fix for n-dimensional arguments. currently broken
        // dyadic i.
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
    }
}

/// i: (monad)
pub fn v_steps(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// i: (dyad)
pub fn v_index_of_last(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// I. (monad)
pub fn v_indices(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// I. (dyad)
pub fn v_interval_index(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// j. (monad)
pub fn v_imaginary(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// j. (dyad)
pub fn v_complex(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// o. (monad)
pub fn v_pi_times(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// o. (dyad)
pub fn v_circle_function(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// p. (monad)
pub fn v_roots(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// p. (dyad)
pub fn v_polynomial(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// p.. (monad)
pub fn v_poly_deriv(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// p.. (dyad)
pub fn v_poly_integral(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// q: (monad)
pub fn v_prime_factors(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// q: (dyad)
pub fn v_prime_exponents(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// r. (monad)
pub fn v_angle(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// r. (dyad)
pub fn v_polar(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// x: (monad)
pub fn v_extend_precision(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// x: (dyad)
pub fn v_num_denom(x: &JArray, y: &JArray) -> Result<Word> {
    if x.shape() != [] {
        return Err(JError::RankError.into());
    }
    let mode = match x.to_i64() {
        Some(x) => x.into_iter().next().expect("len == 1"),
        None => return Err(JError::DomainError.into()),
    };

    match mode {
        2 => match y.to_rat() {
            Some(y) => {
                // same as +. for complex

                let mut shape = y.shape().to_vec();
                shape.push(2);
                let values = y
                    .iter()
                    .flat_map(|x| [x.numer().clone(), x.denom().clone()])
                    .collect();
                Ok(ArrayD::from_shape_vec(shape, values)?.into_noun())
            }
            None => Err(JError::NonceError.into()),
        },
        1 => Err(JError::NonceError.into()),
        x if x < 0 => Err(JError::NonceError.into()),
        _ => Err(JError::DomainError.into()),
    }
}
