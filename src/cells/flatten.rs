use std::cmp::max;
use std::iter;

use anyhow::{Context, Result};
use itertools::Itertools;
use num_traits::Zero;

use crate::arrays::JArrayCow;
use crate::number::{elems_to_jarray, infer_kind_from_boxes, Num};
use crate::verbs::VerbResult;
use crate::{Elem, JArray, JError};

/// See [`JArray::from_fill_promote`].
pub fn fill_promote_list(items: impl IntoIterator<Item = JArray>) -> Result<JArray> {
    let vec = items.into_iter().collect_vec();
    fill_promote_reshape(&(vec![vec.len()], vec))
}

/// [`fill_promote_list`] helper which takes the `Cow` version of our array.
pub fn fill_promote_list_cow(chunk: &[JArrayCow]) -> Result<JArray> {
    fill_promote_list(chunk.iter().map(|arr| arr.to_owned()))
}

// concat_promo_fill(&[JArrayCow]) -> JArray
// concat_promo_fill(x).shape()[0] == x.len()
// fn flatten_reshaping(prefix: Shape, l: &[JArrayCow]) {
//   let cpf = concat_promo_fill(l)?;
//   let s = concat_promo_fill(l)?.shape();
//   s.remove(0);
//   s.unshift(prefix);
//   cpf.into_shape(s)
// }
/// Kinda-internal version of [`fill_promote_list`] which reshapes the result to be compatible
/// with the input, which is what the agreement internals want, but probably isn't what you want.
pub fn fill_promote_reshape((frame, data): &VerbResult) -> Result<JArray> {
    let max_rank = data
        .iter()
        .map(|x| x.shape().len())
        .max()
        .ok_or(JError::NonceError)
        .context("non-empty macrocells")?;

    // rank extend every child
    let results = data
        .into_iter()
        .map(|arr| rank_extend(max_rank, arr))
        .collect_vec();

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
    let target_shape = frame
        .iter()
        .chain(target_inner_shape.iter())
        .copied()
        .collect_vec();

    let kind = infer_kind_from_boxes(&results);

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

    if target_shape.iter().any(|dim| 0 == *dim) {
        big_daddy.clear();
    }
    elems_to_jarray(kind, big_daddy)
        .context("flattening promotion")?
        .into_shape(target_shape)
        .context("flattening output shape")
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
