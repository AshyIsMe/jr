//! Implementations of verbs which primary alter the shape of the input

use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter;

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};

use crate::arrays::{len_of_0, ArcArrayD, IntoVec};
use crate::number::Num;
use crate::{arr0ad, impl_array, impl_homo, JArray, JError};

pub fn reshape(shape: &[usize], arr: &JArray) -> Result<JArray> {
    impl_array!(arr, |arr| reshape_nd(&shape, arr).map(|x| x.into()))
}

pub fn reshape_nd<T>(shape: &[usize], arr: &ArcArrayD<T>) -> Result<ArcArrayD<T>>
where
    T: Debug + Clone,
{
    // get shape of y cells
    // get new shape: concat x with sy
    // flatten y -> into_shape(ns)
    // TODO: This whole section should be x.outer_iter() and then
    // collected.
    let ns: Vec<usize> = shape
        .iter()
        .copied()
        .chain(arr.shape().iter().skip(1).copied())
        .collect();
    let flat_len = ns.iter().product();
    let flat_y = Array::from_iter(arr.iter().cloned().cycle().take(flat_len));
    debug!("ns: {:?}, flat_y: {:?}", ns, flat_y);
    Ok(ArcArray::from_shape_vec(IxDyn(&ns), flat_y.into_raw_vec())?)
}

/// < (monad)
pub fn v_box(y: &JArray) -> Result<JArray> {
    Ok(JArray::BoxArray(arr0ad(y.clone())))
}

/// > (monad)
pub fn v_open(y: &JArray) -> Result<JArray> {
    match y {
        JArray::BoxArray(y) => match y.len() {
            0 => Ok(JArray::BoolArray(
                ArcArrayD::from_shape_vec(IxDyn(&[0]), Vec::new()).expect("static shape"),
            )),
            1 => Ok(y.iter().next().expect("just checked").clone()),
            _ => JArray::from_fill_promote(y.clone()),
        },
        y => Ok(y.clone()),
    }
}

/// |: (monad) (_)
pub fn v_transpose(y: &JArray) -> Result<JArray> {
    Ok(y.transpose())
}

/// |: (dyad) (1, _)
pub fn v_transpose_dyad(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError).context("transpose dyad")
}

/// $ (monad)
pub fn v_shape_of(y: &JArray) -> Result<JArray> {
    Ok(y.shape()
        .iter()
        .map(|v| Ok(i64::try_from(*v)?))
        .collect::<Result<Vec<i64>>>()?
        .into_array()
        .into())
}

/// $ (dyad) (1 _)
pub fn v_shape(x: &JArray, y: &JArray) -> Result<JArray> {
    let x = x.approx_i64_list().context("reshape takes an int list")?;
    if x.iter().product::<i64>() < 0 {
        Err(JError::DomainError).context("cannot reshape to negative shapes")
    } else {
        debug!("v_shape: x: {x:?}, y: {y}");
        reshape(
            &x.iter()
                .map(|x| Ok(usize::try_from(*x)?))
                .collect::<Result<Vec<_>>>()?,
            y,
        )
    }
}

pub fn append_nd(x: &JArray, y: &JArray) -> Result<JArray> {
    impl_homo!(x, y, |x: &ArrayBase<_, _>,
                      y: &ArrayBase<_, _>|
     -> Result<_> {
        Ok(concatenate(Axis(0), &[x.view(), y.view()])?.into_shared())
    })
}

/// , (dyad) (_, _)
pub fn v_append(x: &JArray, y: &JArray) -> Result<JArray> {
    if x.is_empty() {
        return Ok(y.clone().atom_to_singleton());
    }
    if y.is_empty() {
        return Ok(x.clone().atom_to_singleton());
    }

    let x_rank = x.shape().len();
    let y_rank = y.shape().len();

    let (x, y) = match (x_rank, y_rank) {
        (0, 0) => return JArray::from_fill_promote([x.clone(), y.clone()]),
        (0, _) => (reshape(&y.shape()[1..], &x)?, y.clone()),
        (_, 0) => (x.clone(), reshape(&x.shape()[1..], &y)?),
        (_, _) => (x.clone(), y.clone()),
    };

    let target_rank = 1.max(x_rank).max(y_rank);
    let x = x.rank_extend(target_rank);
    let y = y.rank_extend(target_rank);

    JArray::from_fill_promote(x.outer_iter().chain(y.outer_iter()))
}

/// ,. (dyad)
pub fn v_stitch(x: &JArray, y: &JArray) -> Result<JArray> {
    let items = x
        .outer_iter()
        .zip(y.outer_iter())
        .map(|(x, y)| v_append(&x, &y))
        .collect::<Result<Vec<_>>>()?;
    JArray::from_fill_promote(items)
}

/// ; (dyad) (_, _)
pub fn v_link(x: &JArray, y: &JArray) -> Result<JArray> {
    match (x, y) {
        // link: https://code.jsoftware.com/wiki/Vocabulary/semi#dyadic
        // always box x, only box y if not already boxed
        (x, JArray::BoxArray(y)) if y.shape().is_empty() && !y.is_empty() => {
            Ok(vec![x.clone(), y.iter().cloned().next().expect("len == 1")]
                .into_array()
                .into())
        }
        (x, JArray::BoxArray(y)) => Ok(JArray::from_list(
            iter::once(x.clone())
                .chain(
                    y.outer_iter()
                        .into_iter()
                        .map(|x| x.iter().next().expect("non-empty").clone()),
                )
                .collect_vec(),
        )),
        (x, y) => Ok(JArray::from_list([x.clone(), y.clone()])),
    }
}

/// # (dyad) (1, _)
pub fn v_copy(x: &JArray, y: &JArray) -> Result<JArray> {
    assert!(x.shape().len() <= 1);
    if x.is_empty() || y.is_empty() {
        return Err(JError::NonceError).context("empty copy");
    }

    // x is a list of offsets
    let mut x = x.approx_usize_list().context("copy's x")?;

    impl_array!(y, |y: &ArrayBase<_, _>| {
        // y is a list of items
        let mut y = match y.shape().len() {
            0 => vec![y.view()],
            _ => y.outer_iter().collect(),
        };

        // TODO: treats single-item lists as atoms, like other code, but not like this function, apparently
        //    (1$1) # 'abc'
        // |length error
        match (x.len(), y.len()) {
            (1, y) => {
                x = x.into_iter().cycle().take(y).collect();
            }
            (x, 1) => {
                y = y.into_iter().cycle().take(x).collect();
            }
            (x, y) if x == y => (),
            _ => return Err(JError::LengthError).context("unmatched copy arguments"),
        }

        assert_eq!(x.len(), y.len());
        let shape = iter::once(0usize)
            .chain(y[0].shape().iter().copied())
            .collect_vec();

        let mut output: ArrayD<_> =
            ArrayD::from_shape_vec(IxDyn(&shape), vec![]).context("template array")?;
        for (x, y) in x.into_iter().zip(y.into_iter()) {
            for _ in 0..x {
                output
                    .push(Axis(0), y.view())
                    .with_context(|| anyhow!("push: {y:?})"))?;
            }
        }
        Ok(output.into())
    })
}

/// {. (monad)
pub fn v_head(y: &JArray) -> Result<JArray> {
    let a = v_take(&JArray::from(Num::from(1i64)), y)?;
    // ({. 1 2 3) is a different shape to (1 {. 1 2 3)
    if !a.shape().is_empty() {
        let s = a.shape()[1..].to_vec();
        Ok(a.reshape(s)?)
    } else {
        Ok(a)
    }
}

/// {. (dyad)
pub fn v_take(x: &JArray, y: &JArray) -> Result<JArray> {
    assert!(
        x.shape().len() <= 1,
        "agreement guarantee x: {:?}",
        x.shape()
    );

    if x.is_empty() {
        return v_shape(&JArray::from(Num::from(0i64)), y);
    }

    let x = x.approx_i64_list().context("take's x")?;

    if 1 != x.len() {
        return Err(JError::NonceError)
            .with_context(|| anyhow!("expected an atomic x, got a shape of {:?}", x.len()));
    }

    let x = x[0];
    Ok(match x.cmp(&0) {
        Ordering::Equal => y.create_cleared(),
        Ordering::Less => {
            // negative x (take from right)
            let x = usize::try_from(x.abs())
                .map_err(|_| JError::NaNError)
                .context("offset doesn't fit in memory")?;
            let y_len_zero = y.len_of_0();

            if x == 1 {
                match y.shape() {
                    [] => y.reshape(vec![x])?,
                    _ => y.select(Axis(0), &((y_len_zero - x)..y_len_zero).collect_vec()),
                }
            } else {
                if x <= y_len_zero {
                    y.select(Axis(0), &((y_len_zero - x)..y_len_zero).collect_vec())
                } else {
                    return Err(JError::NonceError).context("negative overtake");
                }
            }
        }
        Ordering::Greater => {
            let x = usize::try_from(x)
                .map_err(|_| JError::NaNError)
                .context("offset doesn't fit in memory")?;

            if x == 1 {
                match (y.is_empty(), y.shape()) {
                    (true, _) => JArray::atomic_zero(),
                    (false, []) => y.reshape(vec![x])?,
                    _ => y.slice_axis(Axis(0), Slice::from(..1usize))?,
                }
            } else {
                let y_len_zero = y.len_of_0();
                if x <= y_len_zero {
                    y.select(Axis(0), &(0..x).collect_vec())
                } else {
                    JArray::from_fill_promote(
                        y.outer_iter()
                            // we can't use empty() here as its rank is higher than arr0, which matters
                            .chain(iter::repeat(JArray::atomic_zero()))
                            .take(x),
                    )?
                }
            }
        }
    })
}

/// {: (monad)
pub fn v_tail(y: &JArray) -> Result<JArray> {
    let a = v_take(&JArray::from(Num::from(-1i64)), y)?;
    //    ({: 1 2 3) NB. similar to v_head() where it drops the leading shape rank
    // 3  NB. atom not a single element list
    if !a.shape().is_empty() {
        let s = a.shape()[1..].to_vec();
        Ok(a.reshape(s)?)
    } else {
        Ok(a)
    }
}

/// }: (monad)
pub fn v_curtail(y: &JArray) -> Result<JArray> {
    v_drop(&JArray::from(Num::from(-1i64)), y)
}

/// {:: (dyad)
pub fn v_fetch(x: &JArray, y: &JArray) -> Result<JArray> {
    if !x.shape().is_empty() {
        return Err(JError::NonceError).context("only atomic x (please box lists)");
    }
    let x = match x {
        // an atomic box containing a list of integers
        JArray::BoxArray(b) => b
            .iter()
            .next()
            .expect("just checked: atomic")
            .approx_usize_list()?,
        // a list of integers of length 1
        _ => vec![x.approx_usize_list()?[0]],
    };

    let mut here = y.clone();
    for x in x {
        // let JArray::BoxArray(next) = here else { return Err(JError::NonceError).context("boxed component"); };
        let next = here
            .outer_iter()
            .nth(x)
            .ok_or(JError::IndexError)
            .context("path component past end")?;
        here = next.to_owned();
    }

    if here.shape().is_empty() {
        if let JArray::BoxArray(arr) = here {
            return Ok(arr.into_iter().next().expect("just checked"));
        }
    }
    Ok(here)
}

/// }. (monad)
pub fn v_behead(y: &JArray) -> Result<JArray> {
    if y.is_empty() {
        return Ok(y.clone());
    }

    impl_array!(y, |arr: &ArcArrayD<_>| Ok(arr
        .slice_axis(Axis(0), Slice::from(1isize..))
        .to_shared()
        .into()))
}
/// }. (dyad)
pub fn v_drop(x: &JArray, y: &JArray) -> Result<JArray> {
    use JArray::*;

    if x.is_empty() {
        return Ok(y.clone());
    }

    match x {
        CharArray(_) => Err(JError::DomainError.into()),
        RationalArray(_) => Err(JError::DomainError.into()),
        FloatArray(_) => Err(JError::DomainError.into()),
        ComplexArray(_) => Err(JError::DomainError.into()),
        BoxArray(_) => Err(JError::DomainError.into()),

        _ => impl_array!(x, |xarr: &ArcArrayD<_>| {
            match xarr.shape().len() {
                0 => impl_array!(y, |arr: &ArcArrayD<_>| {
                    let x = x.to_i64().unwrap().into_owned().into_raw_vec()[0];
                    Ok(match x.cmp(&0) {
                        Ordering::Equal => arr.clone().into_owned().into(),
                        Ordering::Less => {
                            //    (_2 }. 1 2 3 4)  NB. equivalent to (2 {. 1 2 3 4)
                            // 3 4
                            let new_x = len_of_0(arr) as i64 - x.abs();
                            v_take(&JArray::from(Num::from(new_x)), y)?
                        }
                        Ordering::Greater => {
                            let new_x = len_of_0(arr) as i64 - x.abs();
                            if new_x < 0 {
                                // todo!("return empty array of type arr")
                                v_take(&JArray::from(Num::from(0i64)), y)?
                            } else {
                                v_take(&JArray::from(Num::from(-new_x)), y)?
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
        let y = ArcArray::from_elem(IxDyn(&[1]), 1);
        let r = reshape_nd(&vec![4], &y).unwrap();
        assert_eq!(r, ArcArray::from_elem(IxDyn(&[4]), 1));
    }
}
