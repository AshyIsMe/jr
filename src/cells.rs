use anyhow::{anyhow, ensure, Context, Result};
use itertools::Itertools;
use log::debug;
use ndarray::prelude::*;

use crate::{reduce_arrays, DyadF, DyadRank, JArray, JArrayCow, JArrays, JError, Rank, Word};

pub fn common_dims(x: &[usize], y: &[usize]) -> usize {
    x.iter()
        .zip(y)
        .position(|(x, y)| x != y)
        .unwrap_or_else(|| x.len().min(y.len()))
}

fn frame_of(shape: &[usize], rank: Rank) -> Result<Vec<usize>> {
    Ok(match rank.usize() {
        None => vec![],
        Some(rank) => {
            ensure!(rank <= shape.len(), "rank {rank:?} higher than {shape:?}");
            shape[..shape.len() - rank].to_vec()
        }
    })
}

fn cells_of(a: &JArray, arg_rank: Rank, surplus_rank: usize) -> Result<JArrayCow> {
    Ok(match arg_rank.usize() {
        None => JArrayCow::from(a),
        Some(arg_rank) => a.choppo(surplus_rank + arg_rank)?,
    })
}

pub fn generate_cells(
    x: JArray,
    y: JArray,
    (x_arg_rank, y_arg_rank): (Rank, Rank),
) -> Result<(Vec<JArray>, Vec<JArray>, Vec<usize>, Vec<usize>)> {
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

    Ok((
        x_macrocells,
        y_macrocells,
        common_frame.to_vec(),
        surplus_frame.to_vec(),
    ))
}

pub fn apply_cells(
    (x_cells, y_cells): (&[JArray], &[JArray]),
    f: DyadF,
    rank: DyadRank,
) -> Result<Vec<Word>> {
    debug!(
        "x_cells.len(): {:?}, y_cells.len(): {:?}",
        x_cells.len(),
        y_cells.len()
    );
    // Handle infinite rank again here, replicate entire argument if so
    let x_limit = if rank.0.is_infinite() {
        y_cells.len()
    } else {
        x_cells.len()
    };
    let y_limit = if rank.1.is_infinite() {
        x_cells.len()
    } else {
        y_cells.len()
    };

    x_cells
        .iter()
        .cycle()
        .take(x_limit)
        .cycle()
        .zip(y_cells.iter().cycle().take(y_limit).cycle())
        .take(x_cells.len().max(y_cells.len()))
        .map(|(x, y)| (f)(x, y))
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
        let (x_cells, y_cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(x_cells, vec![arr0d(5i64).into_jarray()]);
        assert_eq!(y_cells, vec![array![1i64, 2, 3].into_dyn().into_jarray()]);
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_same() -> Result<()> {
        // I think I'd rather the arrays came out whole in this case?
        let x = array![10i64, 20, 30].into_dyn().into_jarray();
        let y = array![1i64, 2, 3].into_dyn().into_jarray();
        let (x_cells, y_cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            x_cells,
            vec![
                arr0d(10i64).into_jarray(),
                arr0d(20i64).into_jarray(),
                arr0d(30i64).into_jarray()
            ]
        );
        assert_eq!(
            y_cells,
            vec![
                arr0d(1i64).into_jarray(),
                arr0d(2i64).into_jarray(),
                arr0d(3i64).into_jarray()
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
        let (x_cells, y_cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            x_cells,
            vec![arr0d(1i64).into_jarray(), arr0d(2i64).into_jarray(),]
        );
        assert_eq!(
            y_cells,
            vec![
                array![10i64, 20, 30].into_dyn().into_jarray(),
                array![70i64, 80, 90].into_dyn().into_jarray(),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_i() -> Result<()> {
        let x = array![100i64, 200].into_dyn().into_jarray();
        let y = array![[0i64, 1, 2], [3, 4, 5]].into_dyn().into_jarray();
        let (x_cells, y_cells, _, _) = generate_cells(x, y, Rank::zero_zero())?;
        assert_eq!(
            x_cells,
            vec![arr0d(100i64).into_jarray(), arr0d(200i64).into_jarray()]
        );
        assert_eq!(
            y_cells,
            vec![
                array![0i64, 1, 2].into_dyn().into_jarray(),
                array![3i64, 4, 5].into_dyn().into_jarray()
            ]
        );
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_gen_macrocells_hash() -> Result<()> {
        let x = array![24i64, 60, 61].into_dyn().into_jarray();
        let y = array![1800i64, 7200].into_dyn().into_jarray();
        let (x_cells, y_cells, _, _) = generate_cells(x, y, (Rank::one(), Rank::zero()))?;
        assert_eq!(
            x_cells,
            vec![array![24i64, 60, 61].into_dyn().into_jarray()]
        );
        assert_eq!(
            y_cells,
            vec![arr0d(1800i64).into_jarray(), arr0d(7200i64).into_jarray()]
        );
        Ok(())
    }
}
