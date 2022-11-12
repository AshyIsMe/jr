use std::collections::HashMap;

use anyhow::Result;
use ndarray::{arr0, array, Array, Axis, IxDyn};

use jr::{IntoJArray, JError, Word};

#[test]
fn array_iter_2_3() {
    let arr = array![[0, 1, 2], [3, 4, 5]].into_dyn();
    assert_eq!(&[2, 3], arr.shape());

    assert_eq!(
        vec![array![0, 1, 2].into_dyn(), array![3, 4, 5].into_dyn()],
        arr.axis_iter(Axis(0)).collect::<Vec<_>>()
    );
}

#[test]
fn array_iter_2() {
    let arr = array![100, 200].into_dyn();
    assert_eq!(&[2], arr.shape());

    assert_eq!(
        vec![arr0(100).into_dyn(), arr0(200).into_dyn()],
        arr.axis_iter(Axis(0)).collect::<Vec<_>>()
    );
}

#[test]
fn array_iter_2_3_2() -> Result<()> {
    let arr = Array::from_shape_vec(IxDyn(&[2, 3, 2]), (0..12).map(|x| x * 10).collect())?;

    assert_eq!(
        vec![
            array![[0, 10], [20, 30], [40, 50]].into_dyn(),
            array![[60, 70], [80, 90], [100, 110]].into_dyn()
        ],
        arr.axis_iter(Axis(0)).collect::<Vec<_>>()
    );

    assert_eq!(
        array![[0, 10], [20, 30], [40, 50]].into_dyn(),
        arr.index_axis(Axis(0), 0)
    );

    Ok(())
}

#[test]
fn test_agreement() {
    let words = jr::scan("10 20 + i.2 3").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        array![[10i64, 11, 12], [23, 24, 25]].into_dyn().into_noun()
    );
}

#[test]
#[ignore]
fn test_agreement_2() -> Result<()> {
    let err = jr::eval(jr::scan("1 2 3 + i. 2 3")?, &mut HashMap::new()).unwrap_err();
    let root = dbg!(err.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(root, JError::LengthError));
    Ok(())
}

#[test]
#[ignore]
fn test_agreement_3() -> Result<()> {
    let err = jr::eval(jr::scan("(2 2 $ 2 3) * i.2 3")?, &mut HashMap::new()).unwrap_err();
    let root = dbg!(err.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(root, JError::LengthError));
    Ok(())
}

#[test]
fn test_agreement_4() -> Result<()> {
    let words = jr::scan("$ (i.2 2) + i.2 2 2")?;
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        array![2i64, 2, 2].into_dyn().into_noun()
    );
    Ok(())
}

#[test]
fn test_agreement_plus_rank1() {
    // 1 2 3 +"1 i.2 3
    let x = array![1i64, 2, 3].into_dyn().into_noun();
    let y =
        Word::noun(Array::from_shape_vec(IxDyn(&[2, 3]), (0i64..6).collect()).unwrap()).unwrap();

    use jr::verbs::*;

    // +"1
    let f = Word::Verb(
        "+\"1".to_string(),
        VerbImpl::Primitive(PrimitiveImpl::new(
            "+",
            v_conjugate,
            v_plus,
            (Rank::zero(), Rank::one(), Rank::one()),
        )),
    );

    let words = vec![x, f, y];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Array::from_shape_vec(IxDyn(&[2, 3]), vec![1i64, 3, 5, 4, 6, 8])
            .unwrap()
            .into_noun()
    );
}

#[test]
fn test_jarray_choppo() -> Result<()> {
    let a = array![[1i64, 2, 3], [5, 6, 7]].into_dyn().into_jarray();
    assert_eq!(a.choppo(0)?.shape()[0], 6);
    assert_eq!(a.choppo(1)?.shape()[0], 2);
    assert_eq!(a.choppo(2)?.shape()[0], 1);
    Ok(())
}

#[test]
fn test_agreement_reshape_3() -> Result<()> {
    let r1 = jr::eval(jr::scan("6 $ i.2 3")?, &mut HashMap::new()).unwrap();
    // 6 3 $ 0 1 2 3 4 5 0 1 2 3 4 5 0 1 2 3 4 5
    let a = Array::from_shape_vec(
        IxDyn(&[6, 3]),
        vec![0i64, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5],
    )?;

    assert_eq!(r1, Word::noun(a).unwrap());

    Ok(())
}

#[test]
fn test_reshape_cycle() -> Result<()> {
    let r1 = jr::eval(jr::scan("6 $ 1 2 3")?, &mut HashMap::new()).unwrap();
    assert_eq!(r1, Word::noun([1i64, 2, 3, 1, 2, 3]).unwrap());
    Ok(())
}
