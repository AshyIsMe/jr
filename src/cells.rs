use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;

use crate::{reduce_arrays, Dyad, JArray, JArrayCow, JArrays, JError, Rank, Word};

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
            // ensure!(rank <= shape.len(), "rank {rank:?} higher than {shape:?}");
            if rank <= shape.len() {
                &shape[..shape.len() - rank]
            } else {
                shape
            }
        }
    })
}

fn cells_of(a: &JArray, arg_rank: Rank, surplus_rank: usize) -> Result<JArrayCow> {
    Ok(match arg_rank.usize() {
        None => JArrayCow::from(a), // AA TODO - HERE !!!
        Some(arg_rank) => a.choppo(surplus_rank + arg_rank)?,
    })
}

pub fn generate_cells<'x, 'y>(
    x: &'x JArray,
    y: &'y JArray,
    (x_arg_rank, y_arg_rank): (Rank, Rank),
) -> Result<(JArrayCow<'x>, JArrayCow<'y>, Vec<usize>, Vec<usize>)> {
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

    // if common_frame.is_empty() && !x_frame.is_empty() && !y_frame.is_empty() {
    //     return Err(JError::LengthError).with_context(|| {
    //         anyhow!("common frame cannot be empty for {x_frame:?} and {y_frame:?}")
    //     });
    // }
    if common_frame.len() < x_frame.len().min(y_frame.len()) {
        bail!(JError::LengthError)
    }

    // this eventually is just `min_rank - arg_rank`,
    // as `to_cells`/`choppo` re-subtract it from the rank
    let x_surplus_rank = x_rank - min_rank;
    let y_surplus_rank = y_rank - min_rank;
    debug!("x_surplus_rank: {:?}", x_surplus_rank);
    debug!("y_surplus_rank: {:?}", y_surplus_rank);

    // Handle infinite ranks properly, entire argument
    let x_cells = if x_arg_rank == Rank::infinite() {
        x.into()
    } else {
        cells_of(x, x_arg_rank, x_surplus_rank)?
    };
    let y_cells = if y_arg_rank == Rank::infinite() {
        y.into()
    } else {
        cells_of(y, y_arg_rank, y_surplus_rank)?
    };
    debug!("x_cells: {:?}", x_cells);
    debug!("y_cells: {:?}", y_cells);

    Ok((x_cells, y_cells, common_frame.into(), surplus_frame.into()))
}

pub fn generate_cells_vec<'x, 'y>(
    x: &'x JArray,
    y: &'y JArray,
    (x_arg_rank, y_arg_rank): (Rank, Rank),
) -> Result<(Vec<JArray>, Vec<JArray>, Vec<usize>, Vec<usize>)> {
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

    if common_frame.len() < x_frame.len().min(y_frame.len()) {
        bail!(JError::LengthError)
    }

    // this eventually is just `min_rank - arg_rank`,
    // as `to_cells`/`choppo` re-subtract it from the rank
    let x_surplus_rank = x_rank - min_rank;
    let y_surplus_rank = y_rank - min_rank;
    debug!("x_surplus_rank: {:?}", x_surplus_rank);
    debug!("y_surplus_rank: {:?}", y_surplus_rank);

    // // Handle infinite ranks properly, entire argument
    // let x_cells = if x_arg_rank == Rank::infinite() {
    //     x.into()
    // } else {
    //     cells_of(x, x_arg_rank, x_surplus_rank)?
    // };
    // let y_cells = if y_arg_rank == Rank::infinite() {
    //     y.into()
    // } else {
    //     cells_of(y, y_arg_rank, y_surplus_rank)?
    // };

    let x_cells = x.rank_iter(x_arg_rank.raw_u8() + x_surplus_rank as u8);
    let y_cells = y.rank_iter(y_arg_rank.raw_u8() + y_surplus_rank as u8);
    debug!("x_cells: {:?}", x_cells);
    debug!("y_cells: {:?}", y_cells);

    Ok((x_cells, y_cells, common_frame.into(), surplus_frame.into()))
}

// TODO: garbage lifetime sharing here, don't pass the CoW objects by reference
pub fn apply_cells<'v>(
    (x_cells, y_cells): (&'v JArrayCow<'v>, &'v JArrayCow<'v>),
    dyad: &Dyad, // f: fn(&JArray, &JArray)
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
    // Handle infinite rank again here, replicate entire argument if so
    let x_iter = if dyad.rank.0 == Rank::infinite() {
        vec![x_cells.clone()]
    } else {
        if x_cells.shape().is_empty() {
            x_cells.iter()
        } else {
            x_cells.outer_iter()
        }
    };
    let y_iter = if dyad.rank.1 == Rank::infinite() {
        vec![y_cells.clone()]
    } else {
        if y_cells.shape().is_empty() {
            y_cells.iter()
        } else {
            y_cells.outer_iter()
        }
    };
    let x_cells_count = if x_cells.shape().is_empty() || dyad.rank.0 == Rank::infinite() {
        1
    } else {
        x_cells.shape()[0]
    };
    let y_cells_count = if y_cells.shape().is_empty() || dyad.rank.1 == Rank::infinite() {
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
        .map(|(x, y)| (dyad.f)(&x.into(), &y.into()))
        .collect()
}

pub fn apply_cells_vec(
    x_cells: Vec<JArray>,
    y_cells: Vec<JArray>,
    dyad: &Dyad,
) -> Result<Vec<Word>> {
    debug!(
        "x_cells.len(): {:?}, y_cells.len(): {:?}",
        x_cells.len(),
        y_cells.len()
    );
    // Handle infinite rank again here, replicate entire argument if so
    let x_iter = if dyad.rank.0 == Rank::infinite() {
        x_cells.iter().cycle().take(y_cells.len())
    } else {
        x_cells.iter().cycle().take(x_cells.len())
    };
    let y_iter = if dyad.rank.1 == Rank::infinite() {
        y_cells.iter().cycle().take(x_cells.len())
    } else {
        y_cells.iter().cycle().take(y_cells.len())
    };

    x_iter
        .into_iter()
        .cycle()
        .zip(y_iter.into_iter().cycle())
        .take(x_cells.len().max(y_cells.len()))
        .map(|(x, y)| (dyad.f)(x, y))
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
    fn test_gen_macrocells_plus_same() -> Result<()> {
        // I think I'd rather the arrays came out whole in this case?
        let x = array![10i64, 20, 30].into_dyn().into_jarray();
        let y = array![1i64, 2, 3].into_dyn().into_jarray();
        let (x_cells, y_cells, _common_frame, _surplus_frame) =
            generate_cells(&x, &y, Rank::zero_zero())?;
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
        let (x_cells, y_cells, _common_frame, _surplus_frame) =
            generate_cells(&x, &y, Rank::zero_zero())?;
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
}
