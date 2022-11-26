mod maff;
mod ranks;

use std::cmp::Ordering;
use std::fmt;
use std::fmt::Debug;
use std::ops::Deref;

use crate::arrays::Arrayable;
use crate::number::{promote_to_array, Num};
use crate::{arr0d, impl_array, IntoJArray, JArray, JError, Word};

use anyhow::{anyhow, bail, ensure, Context, Result};
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};
use num_traits::{FloatConst, Zero};
use rand::prelude::*;

use crate::cells::{apply_cells, flatten, generate_cells, monad_apply, monad_cells};
use crate::JError::DomainError;
use JArray::*;
use Word::*;

use maff::*;
pub use ranks::Rank;

#[derive(Copy, Clone)]
pub struct Monad {
    // TODO: NOT public
    pub f: fn(&JArray) -> Result<Word>,
    pub rank: Rank,
}

pub type DyadF = fn(&JArray, &JArray) -> Result<Word>;
pub type DyadRank = (Rank, Rank);

#[derive(Copy, Clone)]
pub struct Dyad {
    pub f: DyadF,
    pub rank: DyadRank,
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

pub fn exec_monad(f: impl Fn(&JArray) -> Result<Word>, rank: Rank, y: &JArray) -> Result<Word> {
    if rank.is_infinite() {
        return f(y).context("infinite monad shortcut");
    }

    let (cells, common_frame) = monad_cells(y, rank)?;

    let results = monad_apply(&cells, |y| {
        Ok(match f(y)? {
            Word::Noun(arr) => arr,
            other => bail!("not handling non-noun outputs {other:?}"),
        })
    })?;

    let results = flatten(&common_frame, &[], &[results])?;

    Ok(Word::Noun(results))
}

pub fn exec_dyad(
    f: impl Fn(&JArray, &JArray) -> Result<Word>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<Word> {
    if Rank::infinite_infinite() == rank {
        return (f)(x, y).context("infinite dyad shortcut");
    }
    let (cells, common_frame, surplus_frame) =
        generate_cells(x.clone(), y.clone(), rank).context("generating cells")?;

    let application_result = apply_cells(&cells, f, rank).context("applying function to cells")?;
    debug!("application_result: {:?}", application_result);

    let flat = flatten(&common_frame, &surplus_frame, &application_result)?;

    Ok(Word::Noun(flat))
}

impl VerbImpl {
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word> {
        self.exec_ranked(x, y, None)
    }
    pub fn exec_ranked(
        &self,
        x: Option<&Word>,
        y: &Word,
        rank: Option<(Rank, Rank, Rank)>,
    ) -> Result<Word> {
        match self {
            VerbImpl::Primitive(imp) => match (x, y) {
                (None, Word::Noun(y)) => exec_monad(imp.monad.f, imp.monad.rank, y)
                    .with_context(|| anyhow!("monadic {:?}", imp.name)),
                (Some(Word::Noun(x)), Word::Noun(y)) => {
                    let dyad = imp
                        .dyad
                        .ok_or(JError::DomainError)
                        .with_context(|| anyhow!("there is no dyadic {:?}", imp.name))?;
                    exec_dyad(dyad.f, rank.map(|r| (r.1, r.2)).unwrap_or(dyad.rank), x, y)
                        .with_context(|| anyhow!("dyadic {:?}", imp.name))
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
                    log::warn!("Fork {:?} {:?} {:?}", f, g, h);
                    log::warn!("{:?} {:?} {:?}:\n{:?}", x, f, y, f.exec(x, y));
                    log::warn!("{:?} {:?} {:?}:\n{:?}", x, h, y, h.exec(x, y));
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
pub fn v_equal(x: &JArray, y: &JArray) -> Result<Word> {
    rank0eb(x, y, |x, y| x == y)
}

pub fn atom_aware_box(y: &JArray) -> JArray {
    JArray::BoxArray(if y.shape().is_empty() {
        arr0d(Word::Noun(y.clone()))
    } else {
        array![Noun(y.clone())].into_dyn()
    })
}

/// < (monad)
pub fn v_box(y: &JArray) -> Result<Word> {
    Ok(Word::Noun(atom_aware_box(y)))
}
/// < (dyad)
pub fn v_less_than(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Less) => Ok(Num::Bool(1)),
        None => Err(JError::DomainError).context("non-comparable number"),
        _ => Ok(Num::Bool(0)),
    })
}

/// <. (monad)
pub fn v_floor(y: &JArray) -> Result<Word> {
    use Num::*;
    m0nrn(y, |y| {
        Ok(match y {
            Bool(x) => Bool(x),
            Int(x) => Int(x),
            ExtInt(x) => ExtInt(x),
            Rational(x) => Rational(x.floor()),
            Float(x) => Num::float_or_int(x.floor()),
            Complex(_) => {
                return Err(JError::NonceError)
                    .context("floor of a complex number is a complex subject")
            }
        })
    })
}
/// <. (dyad)
pub fn v_lesser_of_min(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Less) | Some(Ordering::Equal) => Ok(x),
        Some(Ordering::Greater) => Ok(y),
        None => Err(JError::DomainError).context("non-comparable number"),
    })
}

/// <: (monad)
pub fn v_decrement(y: &JArray) -> Result<Word> {
    m0nn(y, |y| y - Num::one())
}
/// <: (dyad)
pub fn v_less_or_equal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// > (monad)
pub fn v_open(y: &JArray) -> Result<Word> {
    match y {
        BoxArray(y) => match y.len() {
            1 => Ok(y.iter().next().expect("just checked").clone()),
            _ => bail!("todo: unbox BoxArray"),
        },
        y => Ok(Noun(y.clone())),
    }
}
/// > (dyad)
pub fn v_larger_than(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Greater) => Ok(Num::Bool(1)),
        None => Err(JError::DomainError).context("non-comparable number"),
        _ => Ok(Num::Bool(0)),
    })
}

/// >. (monad)
pub fn v_ceiling(y: &JArray) -> Result<Word> {
    use Num::*;
    m0nrn(y, |y| {
        Ok(match y {
            Bool(x) => Bool(x),
            Int(x) => Int(x),
            ExtInt(x) => ExtInt(x),
            Rational(x) => Rational(x.ceil()),
            Float(x) => Num::float_or_int(x.ceil()),
            Complex(_) => {
                return Err(JError::NonceError)
                    .context("ceil of a complex number is a complex subject")
            }
        })
    })
}
/// >. (dyad)
pub fn v_larger_of_max(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| match x.partial_cmp(&y) {
        Some(Ordering::Greater) | Some(Ordering::Equal) => Ok(x),
        Some(Ordering::Less) => Ok(y),
        None => Err(JError::DomainError).context("non-comparable number"),
    })
}

/// >: (monad)
pub fn v_increment(y: &JArray) -> Result<Word> {
    m0nn(y, |y| y + Num::one())
}
/// >: (dyad)
pub fn v_larger_or_equal(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// + (monad)
pub fn v_conjugate(y: &JArray) -> Result<Word> {
    use Num::*;
    m0nn(y, |y| match y {
        Complex(c) => Complex(c.conj()),
        other => other,
    })
}
/// + (dyad)
pub fn v_plus(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| Ok(x + y))
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
pub fn v_double(y: &JArray) -> Result<Word> {
    m0nn(y, |y| Num::Int(2) * y)
}
/// +: (dyad)
pub fn v_not_or(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| match (x.value_bool(), y.value_bool()) {
        (Some(x), Some(y)) => Ok(Num::bool(!(x || y))),
        _ => Err(JError::DomainError).context("boolean operators only except zeros and ones"),
    })
}

/// * (monad)
pub fn v_signum(y: &JArray) -> Result<Word> {
    use Num::*;
    m0nrn(y, |y| {
        Ok(match y {
            Complex(_) => {
                return Err(JError::NonceError)
                    .context("floor of a complex number is a complex subject")
            }
            // dumb, so dumb
            n @ Bool(_) => n,
            n if n < Num::zero() => Int(-1),
            n if n.is_zero() => Int(0),
            n if n > Num::zero() => Int(1),
            _ => return Err(JError::NaNError).context("should be able to compare with zero"),
        })
    })
}
/// * (dyad)
pub fn v_times(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| Ok(x * y))
}

/// *. (monad)
pub fn v_length_angle(y: &JArray) -> Result<Word> {
    use Num::*;
    m0nj(y, |y| {
        let pair = match y {
            Complex(c) => {
                let len = ((c.im * c.im) + (c.re * c.re)).sqrt();
                let ang = (c.im / c.re).atan();
                [len, ang]
            }
            other => [other.approx_f64().expect("complex covered above"), 0.],
        };

        pair.into_array()
            .expect("infalliable for fixed arrays")
            .into_jarray()
    })
}
/// *. (dyad)
pub fn v_lcm_and(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// *: (monad)
pub fn v_square(y: &JArray) -> Result<Word> {
    // TODO: not clone?
    m0nn(y, |y| y.clone() * y)
}
/// *: (dyad)
pub fn v_not_and(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| match (x.value_bool(), y.value_bool()) {
        (Some(x), Some(y)) => Ok(Num::bool(!(x && y))),
        _ => Err(JError::DomainError).context("boolean operators only except zeros and ones"),
    })
}

/// - (monad)
pub fn v_negate(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// - (dyad)
pub fn v_minus(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| Ok(x - y))
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
pub fn v_reciprocal(y: &JArray) -> Result<Word> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("reciprocal expects a number")?;
    Ok(Word::Noun((Num::one() / y).into()))
}
/// % (dyad)
pub fn v_divide(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| Ok(x / y))
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
    // weird promotion rules here; 2 %: 16 is 4. (float), 2x %: 16 is 4x (extended)
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
    match x.to_i64() {
        Some(x) => {
            if x.product() < 0 {
                Err(JError::DomainError).context("cannot reshape to negative shapes")
            } else {
                debug!("v_shape: x: {x}, y: {y}");
                impl_array!(y, |y| reshape(&x.to_owned(), y).map(|x| x.into_noun()))
            }
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("shapes must appear to be integers, {x:?}")),
    }
}

/// ~: (monad)
pub fn v_nub_sieve(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}
/// ~: (dyad)
pub fn v_not_equal(x: &JArray, y: &JArray) -> Result<Word> {
    rank0eb(x, y, |x, y| x != y)
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

pub fn atom_to_singleton<T: Clone>(y: ArrayD<T>) -> ArrayD<T> {
    if !y.shape().is_empty() {
        y
    } else {
        y.to_shape(IxDyn(&[1]))
            .expect("checked it was an atom")
            .to_owned()
    }
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
        (x, BoxArray(y)) => match atom_aware_box(x) {
            BoxArray(x) => Ok(Word::noun(
                concatenate(
                    Axis(0),
                    &[
                        atom_to_singleton(x).view(),
                        atom_to_singleton(y.clone()).view(),
                    ],
                )
                .context("concatenate")?,
            )
            .context("noun")?),
            _ => bail!("invalid types v_semi({:?}, {:?})", x, y),
        },
        (x, y) => Ok(Word::noun([Noun(x.clone()), Noun(y.clone())])?),
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
    Ok(Word::from(
        i64::try_from(y.len()).map_err(|_| JError::LimitError)?,
    ))
}
/// # (dyad)
pub fn v_copy(x: &JArray, y: &JArray) -> Result<Word> {
    if x.shape().is_empty() && x.len() == 1 && y.shape().len() == 1 {
        if let Some(i) = x.to_i64() {
            let repetitions = i.iter().copied().next().expect("checked");
            ensure!(repetitions > 0, "unimplemented: {repetitions} repetitions");
            let mut output = Vec::new();
            for item in y.clone().into_elems() {
                for _ in 0..repetitions {
                    output.push(item.clone());
                }
            }
            Ok(Word::Noun(promote_to_array(output)?))
        } else {
            Err(JError::NonceError).context("single-int # list-of-nums only")
        }
    } else {
        Err(JError::NonceError).context("non-atom # non-list")
    }
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
pub fn v_head(y: &JArray) -> Result<Word> {
    let res = v_take(&JArray::from(1i64), y)?;
    // ({. 1 2 3) is a different shape to (1 {. 1 2 3)
    match res {
        Noun(a) => {
            if a.shape().len() > 0 {
                let s = &a.shape()[1..];
                Ok(Noun(JArray::from(a.clone().to_shape(s).unwrap())))
            } else {
                Ok(Noun(a))
            }
        }
        _ => Err(JError::NonceError.into()),
    }
}

/// {. (dyad)
pub fn v_take(x: &JArray, y: &JArray) -> Result<Word> {
    match x {
        CharArray(_) => Err(JError::DomainError.into()),
        RationalArray(_) => Err(JError::DomainError.into()),
        FloatArray(_) => Err(JError::DomainError.into()),
        ComplexArray(_) => Err(JError::DomainError.into()),
        BoxArray(_) => Err(JError::DomainError.into()),

        _ => impl_array!(x, |xarr: &ArrayD<_>| {
            match xarr.shape().len() {
                0 => impl_array!(y, |arr: &ArrayD<_>| {
                    let x = x.to_i64().unwrap().into_owned().into_raw_vec()[0];
                    Ok(match x.cmp(&0) {
                        Ordering::Equal => todo!("v_take(): return empty array of type y"),
                        Ordering::Less => {
                            // negative x (take from right)
                            if x.abs() == 1 {
                                match arr.shape() {
                                    [] => {
                                        let s: Vec<usize> = vec![x.abs() as usize];
                                        arr.clone().into_shape(s)?.into_owned().into_noun()
                                    }
                                    _ => {
                                        let i = arr.len_of(Axis(0)) - x.abs() as usize;
                                        let ixs: Vec<usize> =
                                            (i..arr.len_of(Axis(0))).map(|i| i as usize).collect();
                                        arr.select(Axis(0), &ixs).into_owned().into_noun()
                                    }
                                }
                            } else {
                                let i = arr.len_of(Axis(0)) - x.abs() as usize;
                                let ixs: Vec<usize> =
                                    (i..arr.len_of(Axis(0))).map(|i| i as usize).collect();
                                arr.select(Axis(0), &ixs).into_owned().into_noun()
                            }
                        }
                        Ordering::Greater => {
                            if x == 1 {
                                match arr.shape() {
                                    [] => {
                                        let s: Vec<usize> = vec![x as usize];
                                        arr.clone().into_shape(s)?.into_owned().into_noun()
                                    }
                                    _ => arr
                                        .slice_axis(Axis(0), Slice::from(..1usize))
                                        .into_owned()
                                        .into_noun(),
                                }
                            } else {
                                let ixs: Vec<usize> = (0..x).map(|i| i as usize).collect();
                                arr.select(Axis(0), &ixs).into_owned().into_noun()
                            }
                        }
                    })
                }),
                _ => Err(JError::LengthError.into()),
            }
        }),
    }
}

/// {: (monad)
pub fn v_tail(y: &JArray) -> Result<Word> {
    let res = v_take(&JArray::from(-1i64), y)?;
    //    ({: 1 2 3) NB. similar to v_head() where it drops the leading shape rank
    // 3  NB. atom not a single element list
    match res {
        Noun(a) => {
            if a.shape().len() > 0 {
                let s = &a.shape()[1..];
                Ok(Noun(JArray::from(a.clone().to_shape(s).unwrap())))
            } else {
                Ok(Noun(a))
            }
        }
        _ => Err(JError::NonceError.into()),
    }
}

/// }: (monad)
pub fn v_curtail(y: &JArray) -> Result<Word> {
    v_drop(&JArray::from(-1i64), y)
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
pub fn v_drop(x: &JArray, y: &JArray) -> Result<Word> {
    match x {
        CharArray(_) => Err(JError::DomainError.into()),
        RationalArray(_) => Err(JError::DomainError.into()),
        FloatArray(_) => Err(JError::DomainError.into()),
        ComplexArray(_) => Err(JError::DomainError.into()),
        BoxArray(_) => Err(JError::DomainError.into()),

        _ => impl_array!(x, |xarr: &ArrayD<_>| {
            match xarr.shape().len() {
                0 => impl_array!(y, |arr: &ArrayD<_>| {
                    let x = x.to_i64().unwrap().into_owned().into_raw_vec()[0];
                    Ok(match x.cmp(&0) {
                        Ordering::Equal => arr.clone().into_owned().into_noun(),
                        Ordering::Less => {
                            //    (_2 }. 1 2 3 4)  NB. equivalent to (2 {. 1 2 3 4)
                            // 3 4
                            let new_x = y.len_of(Axis(0)) as i64 - x.abs();
                            v_take(&JArray::from(new_x), y)?
                        }
                        Ordering::Greater => {
                            let new_x = arr.len_of(Axis(0)) as i64 - x.abs();
                            if new_x < 0 {
                                todo!("return empty array of type arr")
                            } else {
                                v_take(&JArray::from(new_x * -1), y)?
                            }
                        }
                    })
                }),
                _ => Err(JError::LengthError.into()),
            }
        }),
    }
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
pub fn v_roll(y: &JArray) -> Result<Word> {
    let y = y
        .single_math_num()
        .and_then(|v| v.value_len())
        .ok_or(JError::DomainError)
        .context("expecting zero or a positive integer")?;
    let y = i64::try_from(y)
        .map_err(|_| JError::DomainError)
        .context("must fit in an int")?;
    let mut rng = thread_rng();
    Ok(Word::Noun(match y {
        0 => JArray::from(rng.gen::<f64>()),
        limit => JArray::from(rng.gen_range(0..limit)),
    }))
}
/// ? (dyad)
pub fn v_deal(x: &JArray, y: &JArray) -> Result<Word> {
    let x = x
        .single_math_num()
        .and_then(|n| n.value_len())
        .ok_or(JError::DomainError)
        .context("expecting an usize-like x")?;
    // going via. value_len to elide floats and ban negatives
    let y = y
        .single_math_num()
        .and_then(|n| n.value_len())
        .ok_or(JError::DomainError)
        .context("expecting an usize-like y")?;
    if x > y {
        return Err(JError::DomainError).context("can't pick more items than we have");
    }
    let y = i64::try_from(y)
        .map_err(|_| JError::DomainError)
        .context("must fit in an int")?;
    let mut rng = rand::thread_rng();
    let mut chosen = (0..y).choose_multiple(&mut rng, x);
    chosen.shuffle(&mut rng);
    Ok(chosen.into_array()?.into_noun())
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
pub fn v_imaginary(y: &JArray) -> Result<Word> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'y'")?;

    Ok(Word::Noun((y * Num::i()).into()))
}
/// j. (dyad)
pub fn v_complex(x: &JArray, y: &JArray) -> Result<Word> {
    rank0(x, y, |x, y| Ok(x + (Num::i() * y)))
}

/// o. (monad)
pub fn v_pi_times(y: &JArray) -> Result<Word> {
    m0nn(y, |y| y * Num::Float(f64::PI()))
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
        return Err(JError::RankError).context("num denum requires atomic x");
    }
    let mode = match x.to_i64() {
        Some(x) => x.into_iter().next().expect("len == 1"),
        None => return Err(JError::DomainError).context("num denom requires int-like x"),
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
            None => Err(JError::NonceError).context("expecting a rational input"),
        },
        1 => Err(JError::NonceError).context("mode one unimplemented"),
        x if x < 0 => Err(JError::NonceError).context("negative modes unimplemented"),
        _ => Err(JError::DomainError).context("other modes do not exist"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reshape_helper() {
        let y = Array::from_elem(IxDyn(&[1]), 1);
        let r = reshape(&Array::from_elem(IxDyn(&[1]), 4), &y).unwrap();
        assert_eq!(r, Array::from_elem(IxDyn(&[4]), 1));
    }
}
