//! Implementations of verbs which primary alter the shape of the input

use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter;

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};

use crate::arrays::{len_of_0, Arrayable};
use crate::cells::flatten_list;
use crate::number::{promote_to_array, Num};
use crate::{arr0d, impl_array, impl_homo, HasEmpty, JArray, JError};

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

/// < (monad)
pub fn v_box(y: &JArray) -> Result<JArray> {
    Ok(JArray::BoxArray(arr0d(y.clone())))
}

/// > (monad)
pub fn v_open(y: &JArray) -> Result<JArray> {
    match y {
        JArray::BoxArray(y) => match y.len() {
            0 => Ok(JArray::BoolArray(
                ArrayD::from_shape_vec(IxDyn(&[0]), Vec::new()).expect("static shape"),
            )),
            1 => Ok(y.iter().next().expect("just checked").clone()),
            _ => bail!("todo: unbox BoxArray: {y:?}"),
        },
        y => Ok(y.clone()),
    }
}

/// |: (monad) (_)
pub fn v_transpose(y: &JArray) -> Result<JArray> {
    Ok(y.transpose().into_owned())
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

/// $ (dyad)
pub fn v_shape(x: &JArray, y: &JArray) -> Result<JArray> {
    match x.to_i64() {
        Some(x) => {
            if x.product() < 0 {
                Err(JError::DomainError).context("cannot reshape to negative shapes")
            } else {
                debug!("v_shape: x: {x}, y: {y}");
                impl_array!(y, |y| reshape(&x.to_owned(), y).map(|x| x.into()))
            }
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("shapes must appear to be integers, {x:?}")),
    }
}

fn append_nd(x: &JArray, y: &JArray) -> Result<JArray> {
    impl_homo!(
        x,
        y,
        |x: &ArrayBase<_, _>, y: &ArrayBase<_, _>| concatenate(Axis(0), &[x.view(), y.view()])
    )
}

pub fn unatom(y: JArray) -> JArray {
    if y.shape().is_empty() {
        y.into_shape(IxDyn(&[1])).expect("infalliable")
    } else {
        y
    }
}

/// , (dyad) (_, _)
pub fn v_append(x: &JArray, y: &JArray) -> Result<JArray> {
    if x.shape().len() >= 1 && y.shape().len() >= 1 {
        if let Ok(arr) = append_nd(x, y) {
            return Ok(arr);
        }
    }

    if x.is_empty() {
        return Ok(unatom(y.clone()));
    }
    if y.is_empty() {
        return Ok(unatom(x.clone()));
    }

    if x.shape().len() > 1 || y.shape().len() > 1 || x.is_empty() || y.is_empty() {
        return Err(JError::NonceError)
            .with_context(|| anyhow!("can only append atoms or lists, not {x:?} {y:?}"));
    }

    // TODO: jsoft rejects (DomainError) a bunch of cases promote_to_array accepts
    promote_to_array(
        x.clone()
            .into_elems()
            .into_iter()
            .chain(y.clone().into_elems().into_iter())
            .collect(),
    )
}

/// ,. (dyad)
pub fn v_stitch(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
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
        let s = &a.shape()[1..];
        Ok(a.clone().to_shape(s).unwrap().into_owned())
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
        Ordering::Equal => JArray::empty(),
        Ordering::Less => {
            // negative x (take from right)
            let x = usize::try_from(x.abs())
                .map_err(|_| JError::NaNError)
                .context("offset doesn't fit in memory")?;
            let y_len_zero = y.len();

            if x == 1 {
                match y.shape() {
                    [] => y.to_shape(vec![x])?.into_owned(),
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
                    (false, []) => y.to_shape(vec![x])?.into_owned(),
                    _ => y.slice_axis(Axis(0), Slice::from(..1usize))?.into_owned(),
                }
            } else {
                let y_len_zero = y.len();
                if x <= y_len_zero {
                    y.select(Axis(0), &(0..x).collect_vec())
                } else {
                    flatten_list(
                        y.outer_iter()
                            .map(|cow| cow.into_owned())
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
        let s = &a.shape()[1..];
        Ok(a.clone().to_shape(s).unwrap().into_owned())
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
    let JArray::BoxArray(y) = y else { return Err(JError::NonceError).context("boxed y"); };
    if y.shape().len() > 1 {
        return Err(JError::NonceError).context("multi-dimensional shape output is missing");
    }

    let x = x
        .single_math_num()
        .ok_or(JError::NonceError)
        .context("numeric x")?
        .value_len()
        .ok_or(JError::NonceError)
        .context("positive integer x")?;

    Ok(y.iter()
        .nth(x)
        .ok_or(JError::IndexError)
        .context("x past end of atoms")?
        .to_owned())
}

/// }. (monad)
pub fn v_behead(y: &JArray) -> Result<JArray> {
    if y.is_empty() {
        return Ok(y.clone());
    }

    impl_array!(y, |arr: &ArrayD<_>| Ok(arr
        .slice_axis(Axis(0), Slice::from(1isize..))
        .into_owned()
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

        _ => impl_array!(x, |xarr: &ArrayD<_>| {
            match xarr.shape().len() {
                0 => impl_array!(y, |arr: &ArrayD<_>| {
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
        let y = Array::from_elem(IxDyn(&[1]), 1);
        let r = reshape(&Array::from_elem(IxDyn(&[1]), 4), &y).unwrap();
        assert_eq!(r, Array::from_elem(IxDyn(&[4]), 1));
    }
}
