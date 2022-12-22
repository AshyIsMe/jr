use std::cmp::max;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use log::debug;

use crate::verbs::{DyadRank, Rank};
use crate::{JArray, JError};

pub fn common_dims(x: &[usize], y: &[usize]) -> usize {
    x.iter()
        .zip(y)
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| x.len().min(y.len()))
}

fn frame_of(shape: &[usize], rank: Rank) -> Result<Vec<usize>> {
    Ok(match rank.usize() {
        None => vec![],
        Some(rank) if rank > shape.len() => vec![],
        Some(rank) => shape[..shape.len() - rank].to_vec(),
    })
}

pub fn generate_cells(
    x: JArray,
    y: JArray,
    (x_arg_rank, y_arg_rank): (Rank, Rank),
) -> Result<(Vec<usize>, Vec<(JArray, JArray)>)> {
    let x_shape = x.shape();
    let y_shape = y.shape();

    let x_frame = frame_of(x_shape, x_arg_rank)?;
    let y_frame = frame_of(y_shape, y_arg_rank)?;
    debug!("x_frame: {:?}", x_frame);
    debug!("y_frame: {:?}", y_frame);

    let common_dims = common_dims(&x_frame, &y_frame);
    let common_frame = &x_shape[..common_dims];

    let x_surplus = &x_frame[common_dims..];
    let y_surplus = &y_frame[common_dims..];

    let surplus_frame = match (x_surplus.is_empty(), y_surplus.is_empty()) {
        (false, false) => {
            return Err(JError::LengthError)
                .with_context(|| anyhow!("x:{x_frame:?} y:{y_frame:?}, common: {common_frame:?}"))
        }
        (true, false) => y_surplus,
        // (true, true): it doesn't matter at all
        (true, true) | (false, true) => x_surplus,
    };

    let x_macrocells = x.dims_iter(common_dims);
    let y_macrocells = y.dims_iter(common_dims);

    assert_eq!(x_macrocells.len(), y_macrocells.len());

    let macrocells = x_macrocells.into_iter().zip(y_macrocells).collect();
    let frames = common_frame
        .iter()
        .chain(surplus_frame.iter())
        .copied()
        .collect();
    Ok((frames, macrocells))
}

pub fn monad_cells(y: &JArray, arg_rank: Rank) -> Result<(Vec<JArray>, Vec<usize>)> {
    let frame = frame_of(y.shape(), arg_rank)?;
    Ok((y.rank_iter(arg_rank.raw_u8().into()), frame))
}

pub fn monad_apply(
    macrocells: &[JArray],
    f: impl FnMut(&JArray) -> Result<JArray>,
) -> Result<Vec<JArray>> {
    macrocells.iter().map(f).collect()
}

pub fn apply_cells(
    cells: &[(JArray, JArray)],
    mut f: impl FnMut(&JArray, &JArray) -> Result<JArray>,
    (x_arg_rank, y_arg_rank): DyadRank,
) -> Result<Vec<JArray>> {
    cells
        .iter()
        .flat_map(|(x, y)| {
            let x_parts = x.rank_iter(x_arg_rank.raw_u8().into());
            let y_parts = y.rank_iter(y_arg_rank.raw_u8().into());
            match (x_parts.len(), y_parts.len()) {
                (1, _) | (_, 1) => (),
                _ => unreachable!(
                    "apply_cells can't see multi-lengthonal drifting: {x_parts:?} {y_parts:?}"
                ),
            };
            let limit = max(x_parts.len(), y_parts.len());

            x_parts
                .into_iter()
                .cycle()
                .take(limit)
                .zip(y_parts.into_iter().cycle().take(limit))
                .map(|(x, y)| f(&x, &y))
                // TODO: comedy borrow checker
                .collect_vec()
                .into_iter()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use ndarray::array;

    use super::*;
    use crate::{arr0d, IntoJArray};

    #[test]
    fn test_common_dims() {
        use common_dims as c;
        assert_eq!(1, c(&[2], &[2, 3]));
        assert_eq!(2, c(&[2, 3], &[2, 3]));
        assert_eq!(2, c(&[2, 3], &[2, 3, 4]));
        assert_eq!(0, c(&[3, 2], &[2]));
        assert_eq!(0, c(&[3, 2], &[]));

        let x = [2, 3, 4];
        let y = [2, 3];
        let split_at = c(&x, &y);
        let common_frame = &x[..split_at];
        assert_eq!(&[2, 3], common_frame);

        assert_eq!(&[4], &x[split_at..]);
        assert!(y[split_at..].is_empty());
    }

    #[test]
    fn test_gen_macrocells_plus_one() -> Result<()> {
        let x = arr0d(5i64).into_jarray();
        let y = array![1i64, 2, 3].into_dyn().into_jarray();
        let (_, cells) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            cells,
            vec![(
                arr0d(5i64).into_jarray(),
                array![1i64, 2, 3].into_dyn().into_jarray()
            )]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_same() -> Result<()> {
        // I think I'd rather the arrays came out whole in this case?
        let x = array![10i64, 20, 30].into_dyn().into_jarray();
        let y = array![1i64, 2, 3].into_dyn().into_jarray();
        let (_, cells) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            cells,
            vec![
                (arr0d(10i64).into_jarray(), arr0d(1i64).into_jarray()),
                (arr0d(20i64).into_jarray(), arr0d(2i64).into_jarray()),
                (arr0d(30i64).into_jarray(), arr0d(3i64).into_jarray()),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_two_three() -> Result<()> {
        let x = array![1i64, 2].into_dyn().into_jarray();
        let y = array![[10i64, 20, 30], [70, 80, 90]]
            .into_dyn()
            .into_jarray();
        let (_, cells) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            cells,
            vec![
                (
                    arr0d(1i64).into_jarray(),
                    array![10i64, 20, 30].into_dyn().into_jarray()
                ),
                (
                    arr0d(2i64).into_jarray(),
                    array![70i64, 80, 90].into_dyn().into_jarray()
                ),
            ],
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_i() -> Result<()> {
        let x = array![100i64, 200].into_dyn().into_jarray();
        let y = array![[0i64, 1, 2], [3, 4, 5]].into_dyn().into_jarray();
        let (_, cells) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            cells,
            vec![
                (
                    arr0d(100i64).into_jarray(),
                    array![0i64, 1, 2].into_dyn().into_jarray()
                ),
                (
                    arr0d(200i64).into_jarray(),
                    array![3i64, 4, 5].into_dyn().into_jarray()
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_hash() -> Result<()> {
        let x = array![24i64, 60, 61].into_dyn().into_jarray();
        let y = array![1800i64, 7200].into_dyn().into_jarray();
        let (_, cells) = generate_cells(x, y, (Rank::one(), Rank::zero()))?;
        assert_eq!(
            cells,
            vec![(
                array![24i64, 60, 61].into_dyn().into_jarray(),
                array![1800i64, 7200].into_dyn().into_jarray()
            )]
        );
        Ok(())
    }

    #[test]
    fn monadic_apply() -> Result<()> {
        let y = array![2i64, 3].into_dyn().into_jarray();
        let (cells, _) = monad_cells(&y, Rank::one())?;
        assert_eq!(cells, vec![y.clone()],);

        assert_eq!(
            monad_apply(&[y.clone()], |y| Ok(y.clone()))?,
            vec![y.clone()],
        );
        Ok(())
    }
}
