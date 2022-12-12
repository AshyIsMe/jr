mod impl_impl;
mod impl_maths;
mod impl_shape;
mod maff;
mod ranks;

use std::iter::repeat;

use crate::number::{promote_to_array, Num};
use crate::{impl_array, Ctx, Elem, HasEmpty, IntoJArray, JArray, JError, Word};

use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use ndarray::prelude::*;
use ndarray::Axis;
use num_traits::FloatConst;
use try_partialord::TrySort;

use JArray::*;

use maff::*;
pub use ranks::Rank;

use crate::arrays::{Arrayable, JArrayCow};
pub use impl_impl::*;
pub use impl_maths::*;
pub use impl_shape::*;

pub fn v_not_implemented_monad(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

pub fn v_not_exist_monad(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError).context("this verb lacks a monad")
}

pub fn v_not_implemented_dyad(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

#[allow(unused_variables)]
pub fn v_plot(y: &JArray) -> Result<JArray> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ui")] {
            crate::plot::plot(y)
        } else {
            Err(JError::NonceError.into())
        }
    }
}

// (echo '<table>'; <~/Downloads/Vocabulary.html fgrep '&#149;' | sed 's/<td nowrap>/<tr><td>/g') > a.html; links -dump a.html | perl -ne 's/\s*$/\n/; my ($a,$b,$c) = $_ =~ /\s+([^\s]+) (.*?) \xc2\x95 (.+?)$/; $b =~ tr/A-Z/a-z/; $c =~ tr/A-Z/a-z/; $b =~ s/[^a-z ]//g; $c =~ s/[^a-z -]//g; $b =~ s/ +|-/_/g; $c =~ s/ +|-/_/g; print "/// $a (monad)\npub fn v_$b(y: &Word) -> Result<Word> { Err(JError::NonceError.into()) }\n/// $a (dyad)\npub fn v_$c(x: &Word, y: &Word) -> Result<Word> { Err(JError::NonceError.into()) }\n\n"'

/// = (monad)
pub fn v_self_classify(y: &JArray) -> Result<JArray> {
    let candidates = y.outer_iter();
    let nubs = nub(&candidates);
    let output_shape = [nubs.len(), candidates.len()];
    let mut output = Vec::with_capacity(output_shape[0] * output_shape[1]);
    for nub in &nubs {
        for cand in &candidates {
            let nub = &candidates[*nub];
            output.push(if nub == cand { 1u8 } else { 0u8 });
        }
    }

    Ok(ArrayD::from_shape_vec(&output_shape[..], output)
        .expect("fixed shape")
        .into_jarray())
}

/// -. (dyad)
pub fn v_less(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// -: (dyad)
pub fn v_match(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

fn nub(candidates: &Vec<JArrayCow>) -> Vec<usize> {
    let mut included = Vec::new();
    'outer: for (i, test) in candidates.iter().enumerate() {
        // if we've already seen this value, don't add it to the `included` list,
        // by continuing out of the two loops
        for seen in &included {
            if test == &candidates[*seen] {
                continue 'outer;
            }
        }
        included.push(i);
    }
    included
}

/// ~. (monad) (_)
pub fn v_nub(y: &JArray) -> Result<JArray> {
    // truly awful; missing methods on JArrayCow / JArray which need adding; select, outer_iter()
    // O(n²) 'cos of laziness around PartialEq; might be needed for tolerance

    let candidates = y.outer_iter();
    let included = nub(&candidates);

    Ok(y.select(Axis(0), &included))
}

/// ~: (monad)
pub fn v_nub_sieve(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// |. (monad)
pub fn v_reverse(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// |. (dyad)
pub fn v_rotate_shift(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// , (monad)
pub fn v_ravel(y: &JArray) -> Result<JArray> {
    impl_array!(y, |arr: &ArrayD<_>| {
        Ok(arr.clone().into_raw_vec().into_array()?.into_jarray())
    })
}

/// ,. (monad)
pub fn v_ravel_items(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ,: (monad)
pub fn v_itemize(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// ,: (dyad)
pub fn v_laminate(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ; (monad)
pub fn v_raze(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ;: (monad)
pub fn v_words(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// ;: (dyad)
pub fn v_sequential_machine(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// # (monad)
pub fn v_tally(y: &JArray) -> Result<JArray> {
    Ok(Num::from(i64::try_from(y.len()).map_err(|_| JError::LimitError)?).into())
}

/// #. (monad)
pub fn v_base_(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// #. (dyad)
pub fn v_base(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// #: (monad)
pub fn v_antibase_(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// #: (dyad)
pub fn v_antibase(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// /: (monad)
pub fn v_grade_up(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// /: (dyad)
pub fn v_sort_up(x: &JArray, y: &JArray) -> Result<JArray> {
    if x.shape().len() != 1 || y.shape().len() != 1 {
        return Err(JError::NonceError).context("sort only implemented for (1d) lists");
    }

    let mut y = y.clone().into_elems().into_iter().enumerate().collect_vec();
    y.try_sort_by_key(|(_, n)| Some(n.clone()))
        .map_err(|_| JError::NonceError)
        .context("sort only implemented for simple types")?;
    let x = x.clone().into_elems();
    if x.len() < y.len() {
        return Err(JError::IndexError).context("need more xs than ys");
    }
    // TODO: unnecessary clones, as usual
    Ok(promote_to_array(
        y.into_iter()
            .map(|(i, _)| i)
            .map(|i| x[i].clone())
            .collect(),
    )?)
}

/// \: (monad)
pub fn v_grade_down(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// \: (dyad)
pub fn v_sort_down(x: &JArray, y: &JArray) -> Result<JArray> {
    if x.shape().len() != 1 || y.shape().len() != 1 {
        return Err(JError::NonceError).context("sort only implemented for (1d) lists");
    }

    let mut y = y.clone().into_elems().into_iter().enumerate().collect_vec();
    y.try_sort_by_key(|(_, n)| Some(n.clone()))
        .map_err(|_| JError::NonceError)
        .context("sort only implemented for simple types")?;
    let x = x.clone().into_elems();
    if x.len() < y.len() {
        return Err(JError::IndexError).context("need more xs than ys");
    }
    // TODO: unnecessary clones, as usual
    Ok(promote_to_array(
        y.into_iter()
            .rev()
            .map(|(i, _)| i)
            .map(|i| x[i].clone())
            .collect(),
    )?)
}

/// \[ (monad) and ] (monad) apparently
pub fn v_same(y: &JArray) -> Result<JArray> {
    Ok(y.clone())
}
/// [ (dyad)
pub fn v_left(x: &JArray, _y: &JArray) -> Result<JArray> {
    Ok(x.clone())
}

/// ] (dyad)
pub fn v_right(_x: &JArray, y: &JArray) -> Result<JArray> {
    Ok(y.clone())
}

/// { (monad)
pub fn v_catalogue(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// { (dyad)
pub fn v_from(x: &JArray, y: &JArray) -> Result<JArray> {
    let x = x
        .single_math_num()
        .ok_or(JError::NonceError)
        .context("from single numbers only")?
        .value_i64()
        .ok_or(JError::NonceError)
        .context("from integers only")?;

    if let Ok(x) = usize::try_from(x) {
        let outer = y.outer_iter();
        outer
            .get(x)
            // TODO: pointless (but cheap for once) clone
            .cloned()
            .map(JArray::from)
            .ok_or(JError::IndexError)
            .with_context(|| anyhow!("out of bounds read, {x} is past the end of {y:?}"))
    } else {
        Err(JError::NonceError).context("negative indexes")
    }
}

/// {:: (monad)
pub fn v_map(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// ". (monad)
pub fn v_do(y: &JArray) -> Result<JArray> {
    match y {
        JArray::CharArray(jcode) if jcode.shape().len() == 1 => {
            let mut ctx = Ctx::empty();
            let word = crate::eval(
                crate::scan(&jcode.clone().into_raw_vec().iter().collect::<String>())?,
                &mut ctx,
            )
            .with_context(|| anyhow!("evaluating {:?}", jcode))?;
            Ok(match word {
                Word::Noun(arr) => arr,
                _ => JArray::empty(),
            })
        }
        JArray::CharArray(_) => {
            return Err(JError::NonceError).context("unable to handle atomic or multi-line strings")
        }
        _ => Err(JError::DomainError).context("do() expects a string"),
    }
}
/// ". (dyad)
pub fn v_numbers(x: &JArray, y: &JArray) -> Result<JArray> {
    let x = x
        .single_math_num()
        .ok_or(JError::NonceError)
        .context("atomic x")?;
    match y.shape().len() {
        2 => {
            let CharArray(arr) = y else { return Err(JError::DomainError).context("char array please") };
            let mut nums = Vec::new();
            for line in arr.outer_iter() {
                let s: String = line.iter().collect();
                // TODO: assumes x is 0
                nums.push(Elem::Num(
                    s.trim()
                        .parse::<f64>()
                        .map(Num::Float)
                        .unwrap_or_else(|_| x.clone())
                        .demote(),
                ));
            }
            promote_to_array(nums)
        }
        _ => Err(JError::NonceError).context("atomic x, table y only"),
    }
}

/// ": (monad)
pub fn v_default_format(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// ": (dyad)
pub fn v_format(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// A. (monad)
pub fn v_anagram_index(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// A. (dyad)
pub fn v_anagram(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// C. (monad)
pub fn v_cycledirect(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// C. (dyad)
pub fn v_permute(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// e. (monad)
pub fn v_raze_in(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// e. (dyad)
pub fn v_member_in(x: &JArray, y: &JArray) -> Result<JArray> {
    let ido = v_index_of(y, x).context("member in idot")?;
    let tally = Num::Int(i64::try_from(y.len())?);
    ensure!(ido.shape().len() == 1);

    // promote is laziness, it's a list of bools already
    promote_to_array(
        ido.into_nums()
            .ok_or(JError::NonceError)
            .context("v_index_of returns numbers")?
            .into_iter()
            .map(|n| Elem::Num(Num::bool(n < tally)))
            .collect(),
    )
}

/// i. (monad)
pub fn v_integers(y: &JArray) -> Result<JArray> {
    match y {
        // monadic i.
        IntArray(a) => {
            let p = a.product();
            if p < 0 {
                bail!("todo: monadic i. negative args");
            } else {
                let ints = Array::from_vec((0..p).collect());
                Ok(IntArray(reshape(a, &ints.into_dyn())?))
            }
        }
        ExtIntArray(_) => {
            bail!("todo: monadic i. ExtIntArray")
        }
        _ => Err(JError::DomainError.into()),
    }
}
/// i. (dyad)
pub fn v_index_of(x: &JArray, y: &JArray) -> Result<JArray> {
    if x.shape().len() != 1 {
        return Err(JError::NonceError).context("input x must be a list");
    }
    let x = x.clone().into_elems();
    let output_shape = y.shape();
    let y = y
        .clone()
        .into_elems()
        .into_iter()
        .map(|y| x.iter().position(|x| x == &y).unwrap_or(x.len()))
        .map(|o| Elem::from(i64::try_from(o).expect("arrays that fit in memory")))
        .collect_vec();
    Ok(JArray::from(promote_to_array(y)?.to_shape(output_shape)?))
}

/// E. (dyad) (_, _)
pub fn v_member_interval(x: &JArray, y: &JArray) -> Result<JArray> {
    if x.shape().len() != 1 || y.shape().len() != 1 {
        return Err(JError::NonceError).context("inputs must be lists");
    }
    let x = x.clone().into_elems();
    let y = y.clone().into_elems();
    ensure!(!x.is_empty());
    promote_to_array(
        y.windows(x.len())
            .map(|win| Elem::Num(Num::bool(x == win)))
            .chain(repeat(Elem::Num(Num::bool(false))).take(x.len() - 1))
            .collect(),
    )
}

/// i: (monad)
pub fn v_steps(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// i: (dyad)
pub fn v_index_of_last(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// I. (monad)
pub fn v_indices(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// I. (dyad)
pub fn v_interval_index(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// j. (monad)
pub fn v_imaginary(y: &JArray) -> Result<JArray> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'y'")?;

    Ok((y * Num::i()).into())
}
/// j. (dyad)
pub fn v_complex(x: &JArray, y: &JArray) -> Result<JArray> {
    d00nrn(x, y, |x, y| Ok(x + (Num::i() * y)))
}

/// o. (monad)
pub fn v_pi_times(y: &JArray) -> Result<JArray> {
    m0nn(y, |y| y * Num::Float(f64::PI()))
}
/// o. (dyad)
pub fn v_circle_function(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// p. (monad)
pub fn v_roots(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// p. (dyad)
pub fn v_polynomial(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// p.. (monad)
pub fn v_poly_deriv(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// p.. (dyad)
pub fn v_poly_integral(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// x: (monad)
pub fn v_extend_precision(_y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}
/// x: (dyad)
pub fn v_num_denom(x: &JArray, y: &JArray) -> Result<JArray> {
    if !x.shape().is_empty() {
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
                Ok(ArrayD::from_shape_vec(shape, values)?.into_jarray())
            }
            None => Err(JError::NonceError).context("expecting a rational input"),
        },
        1 => Err(JError::NonceError).context("mode one unimplemented"),
        x if x < 0 => Err(JError::NonceError).context("negative modes unimplemented"),
        _ => Err(JError::DomainError).context("other modes do not exist"),
    }
}
