//! Implementations of verbs which primary alter the shape of the input

use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter;

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;
use ndarray::{concatenate, Axis, Slice};

use crate::arrays::Arrayable;
use crate::number::Num;
use crate::{arr0d, impl_array, IntoJArray, JArray, JError};

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
    JArray::BoxArray(arr0d(y.clone()))
}

/// < (monad)
pub fn v_box(y: &JArray) -> Result<JArray> {
    Ok(atom_aware_box(y))
}

/// > (monad)
pub fn v_open(y: &JArray) -> Result<JArray> {
    match y {
        JArray::BoxArray(y) => match y.len() {
            0 => Ok(JArray::BoolArray(ArrayD::from_shape_vec(IxDyn(&[0]), Vec::new()).expect("static shape"))),
            1 => Ok(y.iter().next().expect("just checked").clone()),
            _ => bail!("todo: unbox BoxArray: {y:?}"),
        },
        y => Ok(y.clone()),
    }
}

/// $ (monad)
pub fn v_shape_of(y: &JArray) -> Result<JArray> {
    Ok(y.shape().into_array()?.into_jarray())
}

/// $ (dyad)
pub fn v_shape(x: &JArray, y: &JArray) -> Result<JArray> {
    match x.to_i64() {
        Some(x) => {
            if x.product() < 0 {
                Err(JError::DomainError).context("cannot reshape to negative shapes")
            } else {
                debug!("v_shape: x: {x}, y: {y}");
                impl_array!(y, |y| reshape(&x.to_owned(), y).map(|x| x.into_jarray()))
            }
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("shapes must appear to be integers, {x:?}")),
    }
}

/// , (dyad)
pub fn v_append(x: &JArray, y: &JArray) -> Result<JArray> {
    if !x.shape().is_empty() || !y.shape().is_empty() || x.is_empty() || y.is_empty() {
        return Err(JError::NonceError).context("can only append atoms");
    }

    // maybe decay to elements sand just send it?
    match (x, y) {
        (JArray::CharArray(l), JArray::CharArray(r)) => Ok(vec![
            *l.iter().next().expect("checked"),
            *r.iter().next().expect("checked"),
        ]
        .into_array()?
        .into_jarray()),
        _ => return Err(JError::NonceError).context("can only append chars"),
    }
}

/// ,. (dyad)
pub fn v_stitch(_x: &JArray, _y: &JArray) -> Result<JArray> {
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
pub fn v_link(x: &JArray, y: &JArray) -> Result<JArray> {
    match (x, y) {
        // link: https://code.jsoftware.com/wiki/Vocabulary/semi#dyadic
        // always box x, only box y if not already boxed
        (x, JArray::BoxArray(y)) => match atom_aware_box(x) {
            JArray::BoxArray(x) => Ok(concatenate(
                Axis(0),
                &[
                    atom_to_singleton(x).view(),
                    atom_to_singleton(y.clone()).view(),
                ],
            )
            .context("concatenate")?
            .into_array()
            .context("noun")?
            .into_jarray()),
            _ => bail!("invalid types v_semi({:?}, {:?})", x, y),
        },
        (x, y) => Ok([x.clone(), y.clone()].into_array()?.into_jarray()),
    }
}

/// # (dyad) (1, _)
pub fn v_copy(x: &JArray, y: &JArray) -> Result<JArray> {
    assert!(x.shape().len() <= 1);
    if x.is_empty() || y.is_empty() {
        return Err(JError::NonceError).context("empty copy");
    }

    // x is a list of offsets
    let mut x = x
        .clone()
        .into_nums()
        .ok_or(JError::DomainError)
        .context("non-numerics as indexes")?
        .into_iter()
        .map(|x| x.value_len())
        .collect::<Option<Vec<usize>>>()
        .ok_or(JError::DomainError)
        .context("non-sizes as indexes")?;

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
        Ok(output.into_jarray())
    })
}

/// {. (monad)
pub fn v_head(y: &JArray) -> Result<JArray> {
    let a = v_take(&JArray::from(Num::from(1i64)), y)?;
    // ({. 1 2 3) is a different shape to (1 {. 1 2 3)
    if !a.shape().is_empty() {
        let s = &a.shape()[1..];
        Ok(JArray::from(a.clone().to_shape(s).unwrap()))
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
            Ok(match x.cmp(&0) {
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
            })
        }
        _ => Err(JError::LengthError)
            .with_context(|| anyhow!("expected an atomic x, got a shape of {:?}", x.len())),
    }
}

/// {: (monad)
pub fn v_tail(y: &JArray) -> Result<JArray> {
    let a = v_take(&JArray::from(Num::from(-1i64)), y)?;
    //    ({: 1 2 3) NB. similar to v_head() where it drops the leading shape rank
    // 3  NB. atom not a single element list
    if !a.shape().is_empty() {
        let s = &a.shape()[1..];
        Ok(JArray::from(a.clone().to_shape(s).unwrap()))
    } else {
        Ok(a)
    }
}

/// }: (monad)
pub fn v_curtail(y: &JArray) -> Result<JArray> {
    v_drop(&JArray::from(Num::from(-1i64)), y)
}

/// {:: (dyad)
pub fn v_fetch(_x: &JArray, _y: &JArray) -> Result<JArray> {
    Err(JError::NonceError.into())
}

/// }. (monad)
pub fn v_behead(y: &JArray) -> Result<JArray> {
    impl_array!(y, |arr: &ArrayD<_>| Ok(arr
        .slice_axis(Axis(0), Slice::from(1isize..))
        .into_owned()
        .into_jarray()))
}
/// }. (dyad)
pub fn v_drop(x: &JArray, y: &JArray) -> Result<JArray> {
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
                        Ordering::Equal => arr.clone().into_owned().into_jarray(),
                        Ordering::Less => {
                            //    (_2 }. 1 2 3 4)  NB. equivalent to (2 {. 1 2 3 4)
                            // 3 4
                            let new_x = y.len_of(Axis(0)) as i64 - x.abs();
                            v_take(&JArray::from(Num::from(new_x)), y)?
                        }
                        Ordering::Greater => {
                            let new_x = arr.len_of(Axis(0)) as i64 - x.abs();
                            if new_x < 0 {
                                todo!("return empty array of type arr")
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
