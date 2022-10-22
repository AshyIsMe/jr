use anyhow::{anyhow, bail, Context, Result};
use ndarray::{arr0, array, ArrayD, IxDyn};
use std::collections::HashMap;

use jr::JArray::*;
use jr::Word::*;
use jr::{args_to_macrocells, arrays, check_agreement, IntoJArray, JArray, JError, Word};

fn arr0d<T>(x: T) -> ArrayD<T> {
    arr0(x).into_dyn()
}

fn j(l: impl IntoJArray, r: impl IntoJArray) -> (JArray, JArray) {
    (l.into_jarray(), r.into_jarray())
}

pub fn idot(s: &[usize]) -> Word {
    let p = s.iter().map(|i| *i as i64).product();
    Noun(IntArray(
        ArrayD::from_shape_vec(IxDyn(&s), (0..p).collect()).unwrap(),
    ))
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
        idot(&[2, 3]),
        [0, 0],
    )?;

    assert_eq!(
        r,
        vec![
            j(arr0d(100i64), arr0d(0i64)),
            j(arr0d(100i64), arr0d(1i64)),
            j(arr0d(100i64), arr0d(2i64)),
            j(arr0d(200i64), arr0d(3i64)),
            j(arr0d(200i64), arr0d(4i64)),
            j(arr0d(200i64), arr0d(5i64)),
        ]
    );
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

#[test]
fn test_agreement_basics_idot_2_3_ranks_1_1() -> Result<()> {
    let r = args_to_macrocells(idot(&[2, 3]), idot(&[2, 3]), [1, 1])?;
    assert_eq!(
        r,
        vec![
            j(array![0i64, 1, 2].into_dyn(), array![0i64, 1, 2].into_dyn()),
            j(array![3i64, 4, 5].into_dyn(), array![3i64, 4, 5].into_dyn()),
        ]
    );
    Ok(())
}

#[test]
fn test_agreement_basics_idot_2_3_ranks_0_1() -> Result<()> {
    let r = args_to_macrocells(idot(&[2, 3]), idot(&[2, 3]), [0, 1])?;
    //println!("r: {:?}", r);
    println!("r:");
    for t in r.iter() {
        println!("{}, {}", t.0, t.1);
    }
    assert_eq!(
        r,
        vec![
            j(arr0d(0i64), array![0i64, 1, 2].into_dyn()),
            j(arr0d(1i64), array![0i64, 1, 2].into_dyn()),
            j(arr0d(2i64), array![0i64, 1, 2].into_dyn()),
            j(arr0d(3i64), array![3i64, 4, 5].into_dyn()),
            j(arr0d(4i64), array![3i64, 4, 5].into_dyn()),
            j(arr0d(5i64), array![3i64, 4, 5].into_dyn()),
        ]
    );
    Ok(())
}

#[test]
fn test_agreement_basics_idot_2_3_ranks_1_0() -> Result<()> {
    let r = args_to_macrocells(idot(&[2, 3]), idot(&[2, 3]), [1, 0])?;
    assert_eq!(
        r,
        vec![
            j(array![0i64, 1, 2].into_dyn(), arr0d(0i64)),
            j(array![0i64, 1, 2].into_dyn(), arr0d(1i64)),
            j(array![0i64, 1, 2].into_dyn(), arr0d(2i64)),
            j(array![3i64, 4, 5].into_dyn(), arr0d(3i64)),
            j(array![3i64, 4, 5].into_dyn(), arr0d(4i64)),
            j(array![3i64, 4, 5].into_dyn(), arr0d(5i64)),
        ]
    );
    Ok(())
}

#[test]
#[ignore]
fn test_agreement_plus() {
    let words = jr::scan("1 2 + i.2 3").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![1, 2, 3, 5, 6, 7]).unwrap(),
        ))
    );
}

#[test]
#[ignore]
fn test_agreement_plus_length_error() -> Result<()> {
    let err = jr::eval(jr::scan("1 2 3 + i. 2 3")?, &mut HashMap::new()).unwrap_err();
    let root = dbg!(err.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(root, JError::LengthError));
    Ok(())
}

#[test]
fn test_check_agreement() {
    let x = Word::noun([1i64, 2]).unwrap();
    let y = Noun(IntArray(
        ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![1, 2, 3, 5, 6, 7]).unwrap(),
    ));

    let r1 = check_agreement(x.clone(), y.clone(), [0, 0]).unwrap();
    assert!(r1);
    let r2 = check_agreement(x.clone(), y.clone(), [0, 1]).unwrap();
    assert!(r2);

    let x = Word::noun([24i64, 60, 60]).unwrap();
    let y = Word::noun([1800i64, 7200]).unwrap();
    let r3 = check_agreement(x.clone(), y.clone(), [1, 0]).unwrap();
    assert!(r3);

    let x = Noun(IntArray(
        ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![0, 1, 2, 3, 4, 5]).unwrap(),
    ));
    let y = Noun(IntArray(
        ArrayD::from_shape_vec(IxDyn(&[2, 4]), vec![0, 1, 2, 3, 4, 5, 6, 7]).unwrap(),
    ));
    let r4 = check_agreement(x.clone(), y.clone(), [0, 0]).unwrap();
    assert!(!r4); // should be false (length error)
}

#[test]
fn test_jarray_to_cells() {
    let a = IntArray(ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![1, 2, 3, 5, 6, 7]).unwrap());
    // for i in a.to_cells(1).unwrap() {
    //     println!("{}", i);
    // }
    assert!(a.to_cells(0).unwrap().len() == 6);
    assert!(a.to_cells(1).unwrap().len() == 2);
    assert!(a.to_cells(2).unwrap().len() == 1);
}

#[test]
fn test_args_to_macrocells() {
    let x = Word::noun([24i64, 60, 60]).unwrap();
    let y = Word::noun([1800i64, 7200]).unwrap();

    let r1 = args_to_macrocells(x, y, [1, 0]).unwrap();
    // for t in r1.clone().into_iter() {
    //     println!("{}, {}", t.0, t.1);
    // }
    //assert!(false);             // Force fail to see println! output
    assert!(r1.len() == 2);

    let x = Noun(IntArray(
        ArrayD::from_shape_vec(IxDyn(&[2, 3]), vec![0, 1, 2, 3, 4, 5]).unwrap(),
    ));
    let y = Noun(IntArray(
        ArrayD::from_shape_vec(IxDyn(&[2, 4]), vec![0, 1, 2, 3, 4, 5, 6, 7]).unwrap(),
    ));
    let r2 = args_to_macrocells(x, y, [0, 0]).unwrap_err();
    let err = dbg!(r2.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(err, JError::LengthError));
}
