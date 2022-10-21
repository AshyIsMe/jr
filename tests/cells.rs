use anyhow::{anyhow, bail, Context, Result};
use ndarray::{arr0, array, ArrayD};

use jr::{args_to_macrocells, arrays, IntoJArray, JArray, JError};

fn arr0d<T>(x: T) -> ArrayD<T> {
    arr0(x).into_dyn()
}

fn j(l: impl IntoJArray, r: impl IntoJArray) -> (JArray, JArray) {
    (l.into_jarray(), r.into_jarray())
}

#[test]
fn test_gen_macrocells_plus_one() -> Result<()> {
    let r = args_to_macrocells(
        arr0d(5i64).into_noun(),
        array![1i64, 2, 3].into_dyn().into_noun(),
        [0, 0],
    )?;
    assert_eq!(
        r,
        vec![
            j(arr0d(5i64), arr0d(1i64)),
            j(arr0d(5i64), arr0d(2i64)),
            j(arr0d(5i64), arr0d(3i64)),
        ]
    );
    Ok(())
}

#[test]
fn test_gen_macrocells_plus_same() -> Result<()> {
    let r = args_to_macrocells(
        array![10i64, 20, 30].into_dyn().into_noun(),
        array![1i64, 2, 3].into_dyn().into_noun(),
        [0, 0],
    )?;

    todo!();
    // assert_eq!(x, IntArrays(vec![arr0d(10), arr0d(20), arr0d(30)]));
    // assert_eq!(y, IntArrays(vec![arr0d(1), arr0d(2), arr0d(3)]));
    Ok(())
}

#[test]
fn test_gen_macrocells_plus_i() -> Result<()> {
    let r = args_to_macrocells(
        array![100i64, 200].into_dyn().into_noun(),
        array![[0i64, 1, 2], [3, 4, 5]].into_dyn().into_noun(),
        [0, 0],
    )?;

    todo!();
    // assert_eq!(x, IntArrays(vec![arr0d(100i64), arr0d(200)]));
    // assert_eq!(
    //     y,
    //     IntArrays(vec![array![0, 1, 2].into_dyn(), array![3, 4, 5].into_dyn()])
    // );
    Ok(())
}

#[test]
fn test_gen_macrocells_hash() -> Result<()> {
    let r = args_to_macrocells(
        array![24i64, 60, 61].into_dyn().into_noun(),
        array![1800i64, 7200].into_dyn().into_noun(),
        [1, 0],
    )?;
    assert_eq!(
        r,
        vec![
            j(array![24i64, 60, 61].into_dyn(), arr0d(1800i64)),
            j(array![24i64, 60, 61].into_dyn(), arr0d(7200i64)),
        ]
    );
    Ok(())
}
