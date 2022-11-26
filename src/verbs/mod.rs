mod impl_impl;
mod impl_maths;
mod impl_shape;
mod maff;
mod ranks;

use crate::number::Num;
use crate::{IntoJArray, JArray, JError, Word};

use anyhow::{bail, Context, Result};
use ndarray::prelude::*;
use ndarray::Axis;
use num_traits::FloatConst;

use JArray::*;
use Word::*;

use maff::*;
pub use ranks::Rank;

pub use impl_impl::*;
pub use impl_maths::*;
pub use impl_shape::*;

pub fn v_not_implemented_monad(_y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

pub fn v_not_implemented_dyad(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
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

/// -. (dyad)
pub fn v_less(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// -: (dyad)
pub fn v_match(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
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

/// ,. (monad)
pub fn v_ravel_items(_y: &JArray) -> Result<Word> {
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

/// {:: (monad)
pub fn v_map(_y: &JArray) -> Result<Word> {
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
                Ok(ArrayD::from_shape_vec(shape, values)?.into_noun())
            }
            None => Err(JError::NonceError).context("expecting a rational input"),
        },
        1 => Err(JError::NonceError).context("mode one unimplemented"),
        x if x < 0 => Err(JError::NonceError).context("negative modes unimplemented"),
        _ => Err(JError::DomainError).context("other modes do not exist"),
    }
}
