use core::cmp::min;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};
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

use crate::JError::DomainError;
use JArray::*;
use Word::*;

#[derive(Copy, Clone)]
pub struct SimpleImpl {
    name: &'static str,
    monad: fn(&JArray) -> Result<Word>,
    dyad: Option<fn(&JArray, &JArray) -> Result<Word>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Simple(SimpleImpl),

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
            VerbImpl::Simple(imp) => match (x, y) {
                (None, Word::Noun(y)) => {
                    (imp.monad)(y).with_context(|| anyhow!("monadic {:?}", imp.name))
                }
                (Some(Word::Noun(x)), Word::Noun(y)) => {
                    imp.dyad
                        .ok_or(JError::DomainError)
                        .with_context(|| anyhow!("dyadic {:?}", imp.name))?(x, y)
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

impl SimpleImpl {
    pub fn monad(name: &'static str, f: fn(&JArray) -> Result<Word>) -> Self {
        Self {
            name,
            monad: f,
            dyad: None,
        }
    }

    pub const fn new(
        name: &'static str,
        monad: fn(&JArray) -> Result<Word>,
        dyad: fn(&JArray, &JArray) -> Result<Word>,
    ) -> Self {
        Self {
            name,
            monad,
            dyad: Some(dyad),
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

            let common_frame = &x_frame[0..common_frame_len]; // grab from either x_frame or y_frame
            let x_surplus_frame = &x_frame[common_frame_len..];
            let y_surplus_frame = &y_frame[common_frame_len..];

            if common_frame_len != shortest_frame_len {
                bail!(JError::LengthError)
            } else {
                // calculate macrocells of x and y
                let x_macrocells = x.to_cells(ranks[0]).unwrap();
                let y_macrocells = y.to_cells(ranks[1]).unwrap();

                match (x_macrocells.len(), y_macrocells.len()) {
                    (0, 0) => bail!(JError::Legacy("empty macrocells?".to_string())),
                    (1, _) => Ok(
                        zip(repeat(x_macrocells[0].clone()), y_macrocells.into_iter()).collect(),
                    ),
                    (_, 1) => Ok(
                        zip(x_macrocells.into_iter(), repeat(y_macrocells[0].clone())).collect(),
                    ),
                    _ => bail!(JError::LengthError),
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
    Ok(Word::Noun(prohomo(x, y)?.plus()))
}

/// +. (monad)
pub fn v_real_imaginary(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
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

/// ~ (monad)
pub fn v_reflex(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ~ (dyad)
pub fn v_passive_evoke(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
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

/// . (monad)
pub fn v_determinant(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// . (dyad)
pub fn v_dot_product(_x: &JArray, _y: &JArray) -> Result<Word> {
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

/// / (monad)
pub fn v_insert(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// / (dyad)
pub fn v_table(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// /. (monad)
pub fn v_oblique(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// /. (dyad)
pub fn v_key(_x: &JArray, _y: &JArray) -> Result<Word> {
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

/// \ (monad)
pub fn v_prefix(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// \ (dyad)
pub fn v_infix(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// \. (monad)
pub fn v_suffix(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// \. (dyad)
pub fn v_outfix(_x: &JArray, _y: &JArray) -> Result<Word> {
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
/// {: (dyad)
pub fn v_map_fetch(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// } (monad)
pub fn v_item_amend(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// } (dyad)
pub fn v_amend_m_u(_x: &JArray, _y: &JArray) -> Result<Word> {
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
