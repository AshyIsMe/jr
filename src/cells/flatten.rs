use std::cmp::max;
use std::iter;

use anyhow::{Context, Result};
use itertools::Itertools;
use ndarray::{ArcArray, IxDyn};
use num_traits::Zero;

use crate::arrays::{size_of_shape_checked, ArcArrayD, JArrayKind};
use crate::number::{infer_kind_from_boxes, Num, Promote};
use crate::verbs::VerbResult;
use crate::{map_kind, Elem, JArray, JError};

/// See [`JArray::from_fill_promote`].
pub fn fill_promote_list(items: impl IntoIterator<Item = JArray>) -> Result<JArray> {
    let vec = items.into_iter().collect_vec();
    fill_promote_reshape((vec![vec.len()], vec))
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
pub fn fill_promote_reshape((frame, data): VerbResult) -> Result<JArray> {
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

    if target_shape.iter().any(|dim| 0 == *dim) {
        return Ok(map_kind!(kind, || ArcArray::from_shape_vec(
            IxDyn(&target_shape),
            Vec::new()
        )));
    }

    return Ok(map_kind!(kind, || {
        let mut big_daddy = Vec::with_capacity(size_of_shape_checked(&IxDyn(&target_shape))?);
        for arr in results {
            if arr.shape() == target_inner_shape {
                push_all(&mut big_daddy, arr);
                continue;
            }

            push_with_shape(&mut big_daddy, &target_inner_shape, arr)?;
        }
        Ok::<_, anyhow::Error>(ArcArrayD::from_shape_vec(target_shape, big_daddy)?.into())
    }));
}

fn rank_extend(target: usize, arr: JArray) -> JArray {
    let rank_extended_shape = (0..target - arr.shape().len())
        .map(|_| &1)
        .chain(arr.shape())
        .copied()
        .collect_vec();

    arr.reshape(rank_extended_shape)
        .expect("rank extension is always valid")
}

// recursive implementation; lops off the start of the dims, recurses on that, then later fills
fn push_with_shape<T: Promote>(out: &mut Vec<T>, target: &[usize], arr: JArray) -> Result<()> {
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
        push_all(out, arr);
    } else {
        let children = arr.outer_iter();
        assert_eq!(initial_size, children.len());
        for item in children {
            push_with_shape(out, remaining_dims, item.to_owned())?;
        }
    }

    let fill = T::promote(Elem::Num(Num::zero()));
    let fills_needed = this_dim_size - initial_size;
    let fills_per_dim = remaining_dims.iter().product::<usize>();

    out.extend(iter::repeat(fill).take(fills_per_dim * fills_needed));

    Ok(())
}

fn push_all<T: Promote>(out: &mut Vec<T>, arr: JArray) {
    macro_rules! conv {
        ($t:ty) => {
            |v| <$t>::promote(Elem::from(v))
        };
    }

    // couldn't get impl_array to work here, which would be less gross
    match arr {
        JArray::BoolArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::CharArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::IntArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::ExtIntArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::RationalArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::FloatArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::ComplexArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
        JArray::BoxArray(arr) => out.extend(arr.into_iter().map(conv!(T))),
    }
}

#[cfg(test)]
mod tests {
    use crate::arrays::ArcArrayD;
    use crate::{arr0ad, JArray};
    use ndarray::array;

    fn push(target: &[usize], arr: ArcArrayD<i64>) -> Vec<i64> {
        let mut out: Vec<i64> = Vec::new();
        let arr = super::rank_extend(target.len(), JArray::IntArray(arr));
        super::push_with_shape(&mut out, target, arr).expect("push success");
        out.into_iter().collect()
    }

    #[test]
    fn test_push_1d() {
        // not sure an atomic output is really legal, currently broken
        // assert_eq!(vec![5], push(&[], arr0d(5)));
        assert_eq!(vec![5], push(&[1], arr0ad(5)));
        assert_eq!(vec![5, 0], push(&[2], arr0ad(5)));
        assert_eq!(
            vec![5, 2, 0, 0, 0, 0],
            push(&[6], array![5, 2].into_dyn().into_shared())
        );
    }

    #[test]
    fn test_push_2d() {
        assert_eq!(
            vec![1, 2, 3, 4, 0, 0],
            push(&[3, 2], array![[1, 2], [3, 4]].into_dyn().into_shared())
        );
    }

    #[test]
    fn test_push_multi_expand_2d() {
        assert_eq!(
            vec![1, 2, 0, 3, 4, 0, 0, 0, 0],
            push(&[3, 3], array![[1, 2], [3, 4]].into_dyn().into_shared())
        );
    }
}
