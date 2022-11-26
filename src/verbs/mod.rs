mod impl_impl;
mod impl_maths;
mod maff;
mod ranks;

use std::cmp::Ordering;
use std::fmt::Debug;

use crate::number::{promote_to_array, Num};
use crate::{arr0d, impl_array, IntoJArray, JArray, JError, Word};

use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};
use num_traits::FloatConst;

use JArray::*;
use Word::*;

use maff::*;
pub use ranks::Rank;

pub use impl_impl::*;
pub use impl_maths::*;

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

/// -. (dyad)
pub fn v_less(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// -: (dyad)
pub fn v_match(_x: &JArray, _y: &JArray) -> Result<Word> {
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
    assert!(
        x.shape().len() <= 1,
        "agreement guarantee x: {:?}",
        x.shape()
    );

    let x = x
        .clone()
        .into_nums()
        .ok_or(JError::DomainError)
        .context("take expecting numeric x")?
        .into_iter()
        .map(|n| n.value_i64())
        .collect::<Option<Vec<i64>>>()
        .ok_or(JError::DomainError)
        .context("takee expecting integer-like x")?;

    match x.len() {
        1 => {
            let x = x[0];
            Ok(Word::Noun(match x.cmp(&0) {
                Ordering::Equal => bail!("v_take(): return empty array of type y"),
                Ordering::Less => {
                    // negative x (take from right)
                    let x = usize::try_from(x.abs())
                        .map_err(|_| JError::NaNError)
                        .context("offset doesn't fit in memory")?;
                    let y_len_zero = y.len_of(Axis(0));

                    if x == 1 {
                        match y.shape() {
                            [] => JArray::from(y.to_shape(vec![x])?),
                            _ => y.select(Axis(0), &((y_len_zero - x)..y_len_zero).collect_vec()),
                        }
                    } else {
                        y.select(Axis(0), &((y_len_zero - x)..y_len_zero).collect_vec())
                    }
                }
                Ordering::Greater => {
                    let x = usize::try_from(x)
                        .map_err(|_| JError::NaNError)
                        .context("offset doesn't fit in memory")?;

                    if x == 1 {
                        match y.shape() {
                            [] => y.to_shape(vec![x])?.into(),
                            _ => y.slice_axis(Axis(0), Slice::from(..1usize)),
                        }
                    } else {
                        y.select(Axis(0), &(0..x).collect_vec())
                    }
                }
            }))
        }
        _ => Err(JError::LengthError)
            .with_context(|| anyhow!("expected an atomic x, got a shape of {:?}", x.len())),
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
