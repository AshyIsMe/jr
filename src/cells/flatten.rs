use std::cmp::max;
use std::iter;

use anyhow::{Context, Result};
use itertools::Itertools;
use num_traits::Zero;

use crate::arrays::BoxArray;
use crate::number::{promote_to_array, Num};
use crate::{Elem, HasEmpty, JArray, JError};

pub fn flatten(results: &BoxArray) -> Result<JArray> {
    if results.is_empty() {
        return Ok(JArray::empty()
            .into_shape(results.shape())
            .context("empty flatten shortcut")?);
    }

    let max_rank = results
        .iter()
        .map(|x| x.shape().len())
        .max()
        .ok_or(JError::NonceError)
        .context("non-empty macrocells")?;

    // rank extend every child
    let results = results.map(|arr| rank_extend(max_rank, arr));

    // max each dimension
    let target_inner_shape = results
        .iter()
        .map(|x| x.shape().to_vec())
        .reduce(|acc, va| {
            assert_eq!(acc.len(), va.len(), "same rank, as we rank extended above");
            // elementwise max
            acc.into_iter()
                .zip(va.into_iter())
                .map(|(l, r)| max(l, r))
                .collect()
        })
        .ok_or(JError::NonceError)
        .context("non-empty macrocells")?
        .to_vec();

    // common_frame + surplus_frame + max(all results)
    let target_shape = results
        .shape()
        .iter()
        .chain(target_inner_shape.iter())
        .copied()
        .collect_vec();

    // flatten
    let mut big_daddy: Vec<Elem> = Vec::new();
    for arr in results {
        if arr.shape() == target_inner_shape {
            // TODO: don't clone

            big_daddy.extend(arr.clone().into_elems());
            continue;
        }

        push_with_shape(&mut big_daddy, &target_inner_shape, arr)?;
    }

    let nums = promote_to_array(big_daddy).context("flattening promotion")?;
    Ok(nums
        .to_shape(target_shape)
        .context("flattening output shape")?
        .into_owned())
}

fn rank_extend(target: usize, arr: &JArray) -> JArray {
    let rank_extended_shape = (0..target - arr.shape().len())
        .map(|_| &1)
        .chain(arr.shape())
        .copied()
        .collect_vec();

    // *not* into_shape, as into_shape returns errors for e.g. reversed arrays
    arr.to_shape(rank_extended_shape)
        .expect("rank extension is always valid")
        .into_owned()
}

// recursive implementation; lops off the start of the dims, recurses on that, then later fills
fn push_with_shape(out: &mut Vec<Elem>, target: &[usize], arr: JArray) -> Result<()> {
    assert!(
        !target.is_empty(),
        "recursion has presumably gone wrong somewhere"
    );

    assert_eq!(arr.shape().len(), target.len());

    let this_dim_size = target[0];
    let initial_size = arr.shape()[0];

    let remaining_dims = &target[1..];

    assert!(
        initial_size <= this_dim_size,
        "{initial_size} <= {this_dim_size}: fill can't see longer shapes"
    );

    if remaining_dims.is_empty() {
        out.extend(arr.to_owned().into_elems());
    } else {
        let children = arr.outer_iter();
        assert_eq!(initial_size, children.len());
        for item in children {
            push_with_shape(out, remaining_dims, item.to_owned())?;
        }
    }

    let fill = Elem::Num(Num::zero());
    let fills_needed = this_dim_size - initial_size;
    let fills_per_dim = remaining_dims.iter().product::<usize>();

    out.extend(iter::repeat(fill).take(fills_per_dim * fills_needed));

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{arr0d, Elem, JArray};
    use ndarray::{array, ArrayD};

    fn push(target: &[usize], arr: ArrayD<i64>) -> Vec<i64> {
        let mut out = Vec::new();
        let arr = super::rank_extend(target.len(), &JArray::IntArray(arr));
        super::push_with_shape(&mut out, target, arr).expect("push success");
        out.into_iter()
            .map(|c| match c {
                Elem::Num(n) => n.value_i64().unwrap(),
                _ => unreachable!("i64 arrays only"),
            })
            .collect()
    }

    #[test]
    fn test_push_1d() {
        // not sure an atomic output is really legal, currently broken
        // assert_eq!(vec![5], push(&[], arr0d(5)));
        assert_eq!(vec![5], push(&[1], arr0d(5)));
        assert_eq!(vec![5, 0], push(&[2], arr0d(5)));
        assert_eq!(vec![5, 2, 0, 0, 0, 0], push(&[6], array![5, 2].into_dyn()));
    }

    #[test]
    fn test_push_2d() {
        assert_eq!(
            vec![1, 2, 3, 4, 0, 0],
            push(&[3, 2], array![[1, 2], [3, 4]].into_dyn())
        );
    }

    #[test]
    fn test_push_multi_expand_2d() {
        assert_eq!(
            vec![1, 2, 0, 3, 4, 0, 0, 0, 0],
            push(&[3, 3], array![[1, 2], [3, 4]].into_dyn())
        );
    }
}
