use std::cmp::max;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use num_traits::Zero;

use crate::arrays::BoxArray;
use crate::number::{promote_to_array, Num};
use crate::{Elem, JArray, JError};

pub fn flatten(results: &BoxArray) -> Result<JArray> {
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

fn push_with_shape(out: &mut Vec<Elem>, target: &[usize], arr: JArray) -> Result<()> {
    let shape = arr.shape();
    assert_eq!(shape.len(), target.len());

    match shape.len() {
        1 => {
            let current = shape[0];
            let target = target[0];
            assert!(
                current <= target,
                "{current} <= {target}: single-dimensional fill can't see longer shapes"
            );
            out.extend(arr.to_owned().into_elems());
            for _ in current..target {
                out.push(Elem::Num(Num::zero()));
            }
        }
        2 if shape[0] < target[0] && shape[1] == target[1] => {
            for sub in arr.outer_iter() {
                let v = sub.clone().into_owned();
                out.extend(v.into_elems());
            }
            for _ in shape[0]..target[0] {
                // only tested this for shape[1] == 1 lolol
                for _ in 0..shape[1] {
                    out.push(Elem::Num(Num::zero()));
                }
            }
        }
        _ => {
            return Err(JError::NonceError).with_context(|| {
                anyhow!("can't framing fill {:?} out to {:?}", arr.shape(), target)
            });
        }
    }
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
    fn test_push() {
        // not sure an atomic output is really legal, currently broken
        // assert_eq!(vec![5], push(&[], arr0d(5)));
        assert_eq!(vec![5], push(&[1], arr0d(5)));
        assert_eq!(vec![5, 0], push(&[2], arr0d(5)));
        assert_eq!(vec![5, 2, 0, 0, 0, 0], push(&[6], array![5, 2].into_dyn()));
        assert_eq!(
            vec![1, 2, 3, 4, 0, 0],
            push(&[3, 2], array![[1, 2], [3, 4]].into_dyn())
        );
    }

    #[test]
    #[ignore]
    fn test_push_multi_expand_2d() {
        assert_eq!(
            vec![1, 2, 0, 3, 4, 0, 0, 0, 0],
            push(&[3, 3], array![[1, 2], [3, 4]].into_dyn())
        );
    }
}
