//! Implementations of verbs which primary alter the shape of the input

use std::cmp::Ordering;
use std::fmt::Debug;

use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};

use crate::number::promote_to_array;
use crate::{arr0d, impl_array, IntoJArray, JArray, JError, Word};

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

pub fn atom_aware_box(y: &JArray) -> JArray {
    JArray::BoxArray(if y.shape().is_empty() {
        arr0d(Word::Noun(y.clone()))
    } else {
        array![Word::Noun(y.clone())].into_dyn()
    })
}

/// < (monad)
pub fn v_box(y: &JArray) -> Result<Word> {
    Ok(Word::Noun(atom_aware_box(y)))
}

/// > (monad)
pub fn v_open(y: &JArray) -> Result<Word> {
    match y {
        JArray::BoxArray(y) => match y.len() {
            1 => Ok(y.iter().next().expect("just checked").clone()),
            _ => bail!("todo: unbox BoxArray"),
        },
        y => Ok(Word::Noun(y.clone())),
    }
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

/// , (dyad)
pub fn v_append(_x: &JArray, _y: &JArray) -> Result<Word> {
    Err(JError::NonceError.into())
}

/// ,. (dyad)
pub fn v_stitch(_x: &JArray, _y: &JArray) -> Result<Word> {
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

/// ; (dyad)
pub fn v_link(x: &JArray, y: &JArray) -> Result<Word> {
    match (x, y) {
        // link: https://code.jsoftware.com/wiki/Vocabulary/semi#dyadic
        // always box x, only box y if not already boxed
        (x, JArray::BoxArray(y)) => match atom_aware_box(x) {
            JArray::BoxArray(x) => Ok(Word::noun(
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
        (x, y) => Ok(Word::noun([Word::Noun(x.clone()), Word::Noun(y.clone())])?),
    }
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

/// {. (monad)
pub fn v_head(y: &JArray) -> Result<Word> {
    use Word::Noun;

    let res = v_take(&JArray::from(1i64), y)?;
    // ({. 1 2 3) is a different shape to (1 {. 1 2 3)
    match res {
        Noun(a) => {
            if !a.shape().is_empty() {
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
    use Word::Noun;
    let res = v_take(&JArray::from(-1i64), y)?;
    //    ({: 1 2 3) NB. similar to v_head() where it drops the leading shape rank
    // 3  NB. atom not a single element list
    match res {
        Noun(a) => {
            if !a.shape().is_empty() {
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
    use JArray::*;
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
                                v_take(&JArray::from(-new_x), y)?
                            }
                        }
                    })
                }),
                _ => Err(JError::LengthError.into()),
            }
        }),
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
