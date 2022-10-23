use anyhow::{anyhow, bail, Context, Result};
use ndarray::{arr0, array, ArrayD};

use crate::JArraysOwned::IntArrays;
use crate::{arrays, IntoJArray, JArray, JArraysOwned, JError};

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

pub fn generate_cells(
    x: &JArray,
    y: &JArray,
    (x_arg_rank, y_arg_rank): (usize, usize),
) -> Result<(JArraysOwned, JArraysOwned)> {
    let x_shape = x.shape();
    let y_shape = y.shape();

    let x_rank = x_shape.len();
    let y_rank = y_shape.len();

    let min_rank = x_rank.min(y_rank);

    let x_frame = &x_shape[..x_rank - x_arg_rank];
    let y_frame = &y_shape[..y_rank - y_arg_rank];

    let common_dims = common_dims(x_frame, y_frame);
    let common_frame = &x_shape[..common_dims];

    if common_frame.is_empty() && !x_frame.is_empty() && !y_frame.is_empty() {
        return Err(JError::LengthError).with_context(|| {
            anyhow!("common frame cannot be empty for {x_frame:?} and {y_frame:?}")
        });
    }

    // this eventually is just `min_rank - arg_rank`,
    // as `to_cells`/`choppo` re-subtract it from the rank
    let x_surplus_rank = x_rank - min_rank;
    let y_surplus_rank = y_rank - min_rank;

    let x_cells = x.to_cells(x_surplus_rank + x_arg_rank)?;
    let y_cells = y.to_cells(y_surplus_rank + y_arg_rank)?;

    Ok((x_cells, y_cells))
}

pub fn match_cells(
    (x, y): (JArraysOwned, JArraysOwned),
) -> Result<Vec<(ArrayD<i64>, ArrayD<i64>)>> {
    use JArraysOwned::*;
    let lens = x.len().max(y.len());
    Ok(match (x, y) {
        (IntArrays(x), IntArrays(y)) => enpairinate(x, y),
        (x, y) => bail!("yet another impl macro? {x:?} {y:?}"),
    })
}

fn enpairinate<X: Clone, Y: Clone>(
    x: Vec<ArrayD<X>>,
    y: Vec<ArrayD<Y>>,
) -> Vec<(ArrayD<X>, ArrayD<Y>)> {
    let lens = x.len().max(y.len());
    x.into_iter()
        .cycle()
        .zip(y.into_iter().cycle())
        .take(lens)
        .collect()
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
        use JArraysOwned::*;
        let (x, y) = generate_cells(
            &arr0d(5i64).into_jarray(),
            &array![1i64, 2, 3].into_dyn().into_jarray(),
            (0, 0),
        )?;
        assert_eq!(x, IntArrays(vec![arr0d(5)]));
        assert_eq!(y, IntArrays(vec![array![1, 2, 3].into_dyn()]));
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_same() -> Result<()> {
        // I think I'd rather the arrays came out whole in this case?
        use JArraysOwned::*;
        let (x, y) = generate_cells(
            &array![10i64, 20, 30].into_dyn().into_jarray(),
            &array![1i64, 2, 3].into_dyn().into_jarray(),
            (0, 0),
        )?;
        assert_eq!(x, IntArrays(vec![arr0d(10), arr0d(20), arr0d(30)]));
        assert_eq!(y, IntArrays(vec![arr0d(1), arr0d(2), arr0d(3)]));
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_plus_i() -> Result<()> {
        use JArraysOwned::*;
        let (x, y) = generate_cells(
            &array![100i64, 200].into_dyn().into_jarray(),
            &array![[0i64, 1, 2], [3, 4, 5]].into_dyn().into_jarray(),
            (0, 0),
        )?;
        assert_eq!(x, IntArrays(vec![arr0d(100i64), arr0d(200)]));
        assert_eq!(
            y,
            IntArrays(vec![array![0, 1, 2].into_dyn(), array![3, 4, 5].into_dyn()])
        );
        Ok(())
    }

    #[test]
    fn test_gen_macrocells_hash() -> Result<()> {
        use JArraysOwned::*;
        let (x, y) = generate_cells(
            &array![24i64, 60, 61].into_dyn().into_jarray(),
            &array![1800i64, 7200].into_dyn().into_jarray(),
            (1, 0),
        )?;
        assert_eq!(x, IntArrays(vec![array![24, 60, 61].into_dyn()]));
    assert_eq!(y, IntArrays(vec![arr0d(1800i64), arr0d(7200)]));
        Ok(())
    }
}
