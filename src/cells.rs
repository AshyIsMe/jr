use anyhow::{anyhow, ensure, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;

use crate::{reduce_arrays, JArray, JArrayCow, JArrays, JError, Rank, Word};

pub fn result_shape<'s>(x: &'s JArray, y: &'s JArray) -> &'s [usize] {
    let x_shape = x.shape();
    let y_shape = y.shape();
    if x_shape.len() > y_shape.len() {
        x_shape
    } else {
        y_shape
    }
}

pub fn common_dims(x: &[usize], y: &[usize]) -> usize {
    x.iter()
        .zip(y)
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| x.len().min(y.len()))
}

fn frame_of(shape: &[usize], rank: Rank) -> Result<&[usize]> {
    Ok(match rank.usize() {
        None => shape,
        Some(rank) => {
            ensure!(rank <= shape.len(), "rank {rank:?} higher than {shape:?}");
            &shape[..shape.len() - rank]
        }
    })
}

fn cells_of(a: &JArray, arg_rank: Rank, surplus_rank: usize) -> Result<JArrayCow> {
    Ok(match arg_rank.usize() {
        None => JArrayCow::from(a),
        Some(arg_rank) => a.choppo(surplus_rank + arg_rank)?,
    })
}

pub fn generate_cells<'x, 'y>(
    x: &'x JArray,
    y: &'y JArray,
    (x_arg_rank, y_arg_rank): (Rank, Rank),
) -> Result<(JArrayCow<'x>, JArrayCow<'y>)> {
    let x_shape = x.shape();
    let y_shape = y.shape();
    debug!("x_shape: {:?}", x_shape);
    debug!("y_shape: {:?}", y_shape);

    let x_rank = x_shape.len();
    let y_rank = y_shape.len();

    let min_rank = x_rank.min(y_rank);

    let x_frame = frame_of(x_shape, x_arg_rank)?;
    let y_frame = frame_of(y_shape, y_arg_rank)?;
    debug!("x_frame: {:?}", x_frame);
    debug!("y_frame: {:?}", y_frame);

    let common_dims = common_dims(x_frame, y_frame);
    let common_frame = &x_shape[..common_dims];
    let surplus_frame = if x_frame.len() > y_frame.len() {
        &x_shape[common_dims..]
    } else {
        &y_shape[common_dims..]
    };

    debug!("common_frame: {:?}", common_frame);
    debug!("surplus_frame: {:?}", surplus_frame);

    if common_frame.is_empty() && !x_frame.is_empty() && !y_frame.is_empty() {
        return Err(JError::LengthError).with_context(|| {
            anyhow!("common frame cannot be empty for {x_frame:?} and {y_frame:?}")
        });
    }

    // this eventually is just `min_rank - arg_rank`,
    // as `to_cells`/`choppo` re-subtract it from the rank
    let x_surplus_rank = x_rank - min_rank;
    let y_surplus_rank = y_rank - min_rank;
    debug!("x_surplus_rank: {:?}", x_surplus_rank);
    debug!("y_surplus_rank: {:?}", y_surplus_rank);

    let x_cells = cells_of(x, x_arg_rank, x_surplus_rank)?;
    let y_cells = cells_of(y, y_arg_rank, y_surplus_rank)?;
    debug!("x_cells: {:?}", x_cells);
    debug!("y_cells: {:?}", y_cells);

    Ok((x_cells, y_cells))
}

pub fn first_or_def<T>(arr: &[T], default: T) -> T
where
    T: Clone,
{
    match arr.len() > 0 {
        true => arr[0].clone(),
        false => default,
    }
}

// TODO: garbage lifetime sharing here, don't pass the CoW objects by reference
pub fn apply_cells<'v>(
    (x_cells, y_cells): (&'v JArrayCow<'v>, &'v JArrayCow<'v>),
    f: fn(&JArray, &JArray) -> Result<Word>,
) -> Result<Vec<Word>> {
    debug!(
        "x_cells.len(): {:?}, y_cells.len(): {:?}",
        x_cells.len(),
        y_cells.len()
    );
    debug!(
        "x_cells.shape(): {:?}, y_cells.shape(): {:?}",
        x_cells.shape(),
        y_cells.shape()
    );
    let x_iter = if x_cells.shape().is_empty() {
        x_cells.iter()
    } else {
        x_cells.outer_iter()
    };
    let y_iter = if y_cells.shape().is_empty() {
        y_cells.iter()
    } else {
        y_cells.outer_iter()
    };
    let x_cells_count = if x_cells.shape().is_empty() {
        1
    } else {
        x_cells.shape()[0]
    };
    let y_cells_count = if y_cells.shape().is_empty() {
        1
    } else {
        y_cells.shape()[0]
    };

    x_iter
        .into_iter()
        .cycle()
        .zip(y_iter.into_iter().cycle())
        // .take(x_cells.len().max(y_cells.len()))
        .take(x_cells_count.max(y_cells_count))
        .map(|(x, y)| f(&x.into(), &y.into()))
        .collect()
}

pub fn flatten(shape: &[usize], vecs: &[Word]) -> Result<JArray> {
    let arr = vecs
        .iter()
        .map(|w| match w {
            Word::Noun(arr) => Ok(arr),
            _ => Err(JError::DomainError).with_context(|| anyhow!("{w:?}")),
        })
        .collect::<Result<Vec<_>>>()?;
    let arrs = JArrays::from_homo(&arr)?;
    Ok(reduce_arrays!(
        arrs,
        |v: &[ArrayViewD<'_, _>]| -> Result<ArrayD<_>> {
            let vec = v
                .into_iter()
                .flat_map(|v| v.into_iter())
                .cloned()
                .collect_vec();
            Ok(ArrayD::from_shape_vec(shape, vec)?)
        }
    ))
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use ndarray::{arr0, array, ArrayD};

    use super::*;
    use crate::IntoJArray;

    fn arr0d<T>(x: T) -> ArrayD<T> {
        arr0(x).into_dyn()
    }

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
        let (x_cells, y_cells, target_shape) = generate_cells(&x, &y, Rank::zero_zero())?;
        assert_eq!(x_cells.outer_iter(), vec![arr0d(5i64).into()]);
        assert_eq!(
            y_cells.outer_iter(),
            vec![array![1i64, 2, 3].into_dyn().into()]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_same() -> Result<()> {
        // I think I'd rather the arrays came out whole in this case?
        let x = array![10i64, 20, 30].into_dyn().into_jarray();
        let y = array![1i64, 2, 3].into_dyn().into_jarray();
        let (x_cells, y_cells, target_shape) = generate_cells(&x, &y, Rank::zero_zero())?;
        assert_eq!(
            x_cells.outer_iter(),
            vec![
                arr0d(10i64).into(),
                arr0d(20i64).into(),
                arr0d(30i64).into()
            ]
        );
        assert_eq!(
            y_cells.outer_iter(),
            vec![arr0d(1i64).into(), arr0d(2i64).into(), arr0d(3i64).into()]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_i() -> Result<()> {
        let x = array![100i64, 200].into_dyn().into_jarray();
        let y = array![[0i64, 1, 2], [3, 4, 5]].into_dyn().into_jarray();
        let (x_cells, y_cells, target_shape) = generate_cells(&x, &y, Rank::zero_zero())?;
        assert_eq!(
            x_cells.outer_iter(),
            vec![arr0d(100i64).into(), arr0d(200i64).into(),]
        );
        assert_eq!(
            y_cells.outer_iter(),
            vec![
                array![0i64, 1, 2].into_dyn().into(),
                array![3i64, 4, 5].into_dyn().into()
            ]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_hash() -> Result<()> {
        let x = array![24i64, 60, 61].into_dyn().into_jarray();
        let y = array![1800i64, 7200].into_dyn().into_jarray();
        let (x_cells, y_cells, target_shape) = generate_cells(&x, &y, (Rank::one(), Rank::zero()))?;
        assert_eq!(
            x_cells.outer_iter(),
            vec![array![24i64, 60, 61].into_dyn().into(),]
        );
        assert_eq!(
            y_cells.outer_iter(),
            vec![arr0d(1800i64).into(), arr0d(7200i64).into(),]
        );
        Ok(())
    }
}
