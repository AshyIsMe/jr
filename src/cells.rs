use std::cmp::max;

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use log::debug;

use crate::{promote_to_array, DyadF, DyadRank, JArray, JError, Num, Rank, Word};

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
) -> Result<(Vec<(JArray, JArray)>, Vec<usize>, Vec<usize>)> {
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

    debug!("common_frame: {:?}", common_frame);
    debug!("surplus_frame: {:?}", surplus_frame);

    let neg_common_frame_rank = -(common_frame.len() as i16);
    let x_macrocells = x.rank_iter(neg_common_frame_rank);
    let y_macrocells = y.dims_iter(neg_common_frame_rank);

    assert_eq!(x_macrocells.len(), y_macrocells.len());

    debug!("x_macrocells (len: {}):", x_cells.len());
    for (i, c) in x_macrocells.iter().enumerate() {
        debug!("index {}\n{}", i, c);
    }
    debug!("y_macrocells (len: {}):", y_cells.len());
    for (i, c) in y_macrocells.iter().enumerate() {
        debug!("index {}\n{}", i, c);
    }

    let macrocells = x_macrocells.into_iter().zip(y_macrocells).collect_vec();

    Ok((macrocells, common_frame.to_vec(), surplus_frame.to_vec()))
}

pub fn apply_cells(
    cells: &[(JArray, JArray)],
    f: DyadF,
    (x_arg_rank, y_arg_rank): DyadRank,
) -> Result<Vec<Vec<JArray>>> {
    let mut cell_results = Vec::new();

    for (x, y) in cells {
        let x_parts = x.rank_iter(x_arg_rank.raw_u8().into());
        let y_parts = y.rank_iter(y_arg_rank.raw_u8().into());
        match (x_parts.len(), y_parts.len()) {
            (1, _) | (_, 1) => (),
            _ => bail!("apply_cells can't see multi-lengthonal drifting: {x_parts:?} {y_parts:?}"),
        };
        let limit = max(x_parts.len(), y_parts.len());
        cell_results.push(
            x_parts
                .into_iter()
                .cycle()
                .take(limit)
                .zip(y_parts.into_iter().cycle().take(limit))
                .map(|(x, y)| f(&x, &y))
                .map(|r| {
                    r.and_then(|v| match v {
                        Word::Noun(arr) => Ok(arr),
                        other => Err(anyhow!(
                            "refusing to believe there's a {other:?} in an array of arrays"
                        )),
                    })
                })
                .collect::<Result<_>>()?,
        )
    }

    Ok(cell_results)
}

pub fn flatten(
    common_frame: &[usize],
    surplus_frame: &[usize],
    macrocell_results: &[Vec<JArray>],
) -> Result<JArray> {
    assert_eq!(
        common_frame.iter().product::<usize>(),
        macrocell_results.len()
    );
    for macrocell in macrocell_results {
        assert_eq!(surplus_frame.iter().product::<usize>(), macrocell.len());
    }

    // max(all results)
    let target_inner_shape = macrocell_results
        .iter()
        .flat_map(|one| one.iter().map(|x| x.shape()))
        .max()
        .expect("non-empty macrocells");

    // common_frame + surplus_frame + max(all results)
    let target_shape = common_frame
        .iter()
        .copied()
        .chain(surplus_frame.iter().copied())
        .chain(target_inner_shape.iter().copied())
        .collect_vec();

    // flatten
    let mut big_daddy: Vec<Num> = Vec::new();
    for macrocell in macrocell_results {
        for arr in macrocell {
            if arr.shape() == target_inner_shape {
                // TODO: don't clone

                big_daddy.extend(arr.clone().into_nums()?);
                continue;
            }

            match (arr.shape().len(), target_inner_shape.len()) {
                (1, 1) => {
                    let current = arr.shape()[0];
                    let target = target_inner_shape[0];
                    assert!(current < target, "{current} < {target}: single-dimensional fill can't see longer or equal shapes");
                    big_daddy.extend(arr.clone().into_nums()?);
                    for _ in current..target {
                        big_daddy.push(Num::Bool(0));
                    }
                }
                _ => {
                    return Err(JError::NonceError).with_context(|| {
                        anyhow!(
                            "can't framing fill {:?} out to {:?}",
                            arr.shape(),
                            target_inner_shape
                        )
                    });
                }
            }
        }
    }

    let nums = promote_to_array(big_daddy)?;
    Ok(nums.to_shape(target_shape)?.into())
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
        let (cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
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
        let (cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
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
        let (cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
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
        let (cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
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
    #[ignore]
    fn test_gen_macrocells_hash() -> Result<()> {
        let x = array![24i64, 60, 61].into_dyn().into_jarray();
        let y = array![1800i64, 7200].into_dyn().into_jarray();
        let (cells, _, _) = generate_cells(x, y, (Rank::one(), Rank::zero()))?;
        assert_eq!(
            cells,
            vec![(
                array![24i64, 60, 61].into_dyn().into_jarray(),
                array![1800i64, 7200].into_dyn().into_jarray()
            )]
        );
        Ok(())
    }
}
