use anyhow::Result;
use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use jr::test_impls::scan_eval;
use jr::JArray::*;
use jr::Word::*;
use jr::{arr0d, collect_nouns, resolve_names, Ctx, JArray, JError, Num, Rank, Word};

pub fn scan_eval_unwrap(sentence: impl AsRef<str>) -> Word {
    let sentence = sentence.as_ref();
    scan_eval(sentence).expect("scan_eval_unwrap")
}

use scan_eval_unwrap as s;

pub fn idot(s: &[usize]) -> JArray {
    let p = s.iter().map(|i| *i as i64).product();
    //Noun(IntArray(ArrayD::from_shape_vec(IxDyn(&s), (0..p).collect()).unwrap(),))
    IntArray(ArrayD::from_shape_vec(IxDyn(&s), (0..p).collect()).unwrap())
}

#[test]
fn test_basic_addition() {
    assert_eq!(scan_eval("2 + 2").unwrap(), Word::from(4i64));

    let v = [5i64, 7, 9];
    assert_eq!(
        scan_eval("1 2 3 + 4 5 6").unwrap(),
        Word::Noun(JArray::from_list(v))
    );

    assert_eq!(scan_eval("1 + 3.14").unwrap(), Word::from(1f64 + 3.14));
}

#[test]
fn test_basic_times() {
    assert_eq!(scan_eval("2 * 2").unwrap(), Word::from(4));

    let v = [4i64, 10, 18];
    assert_eq!(
        scan_eval("1 2 3 * 4 5 6").unwrap(),
        Word::Noun(JArray::from_list(v))
    );
}

#[test]
fn test_parse_basics() {
    let v = [1i64, 2, 3];
    let words = vec![
        Word::from(2),
        Word::static_verb("+"),
        Word::Noun(JArray::from_list(v)),
    ];
    let v = [3i64, 4, 5];
    assert_eq!(
        jr::eval(words, &mut Ctx::root()).unwrap(),
        Word::Noun(JArray::from_list(v)),
    );
}

#[test]
fn test_insert_adverb() {
    assert_eq!(scan_eval("+/1 2 3").unwrap(), Word::from(6));
}

#[test]
fn test_reshape() {
    assert_eq!(
        scan_eval("2 2 $ 1 2 3 4").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![1, 2, 3, 4]).unwrap()
        ))
    );

    assert_eq!(
        scan_eval("4 $ 1").unwrap(),
        Noun(BoolArray(Array::from_elem(IxDyn(&[4]), 1)))
    );

    assert_eq!(
        scan_eval("1 2 3 $ 1 2").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[1, 2, 3]), vec![1, 2, 1, 2, 1, 2]).unwrap()
        ))
    );

    assert_eq!(
        scan_eval("3 $ 2 2 $ 0 1 2 3").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3, 2]), vec![0, 1, 2, 3, 0, 1]).unwrap()
        ))
    );
}

#[test]
fn test_reshape_outer_iter() {
    assert_eq!(
        scan_eval("2 1 $ 1 2").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 1]), vec![1, 2]).unwrap()
        ))
    );
}

#[test]
fn test_reshape_2d_match_1d() -> Result<()> {
    let r1 = scan_eval("(2 2 $ 3) $ 1").unwrap();
    let r2 = scan_eval("2 3 3 $ 1").unwrap();

    let correct_result = Noun(BoolArray(Array::from_elem(IxDyn(&[2, 3, 3]), 1)));

    assert_eq!(r1, correct_result);
    assert_eq!(r2, correct_result);
    assert_eq!(r1, r2);

    Ok(())
}

#[test]
fn test_reshape_no_change() -> Result<()> {
    let r1 = scan_eval("i.2 3 4").unwrap();
    let r2 = scan_eval("2 $ i.2 3 4").unwrap();

    assert_eq!(r1, r2);

    Ok(())
}

#[test]
fn test_agreement_reshape_3() -> Result<()> {
    let r1 = scan_eval("6 $ i.2 3").unwrap();
    // 6 3 $ 0 1 2 3 4 5 0 1 2 3 4 5 0 1 2 3 4 5
    let a = Array::from_shape_vec(
        IxDyn(&[6, 3]),
        vec![0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5],
    )?;

    assert_eq!(r1, Noun(IntArray(a)));

    Ok(())
}

#[test]
fn test_reshape_cycle() -> Result<()> {
    let r1 = scan_eval("6 $ 1 2 3").unwrap();
    let v = [1i64, 2, 3, 1, 2, 3];
    assert_eq!(r1, Word::Noun(JArray::from_list(v)));
    Ok(())
}

#[test]
fn test_power_conjunction_bool_arg() {
    let v = [4i64, 16];
    assert_eq!(
        scan_eval("(*:^:0 1) 4").unwrap(),
        Word::Noun(JArray::from_list(v))
    );
}

#[test]
fn test_power_conjunction_noun_arg() {
    assert_eq!(scan_eval("(*:^:2) 4").unwrap(), Word::from(256));

    assert_eq!(
        scan_eval("(*:^:2 3) 2 3").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![16, 81, 256, 6561]).unwrap(),
        )),
    );
}

#[test]
fn test_collect_int_nouns() {
    let v = [2i64, 3];
    let v1 = [0i64, 1];
    let a = vec![
        Word::Noun(JArray::from_list(v1)),
        Word::Noun(JArray::from_list(v)),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![0, 1, 2, 3]).unwrap(),
        )),
    );
}

#[test]
fn test_collect_extint_nouns() {
    let v = [BigInt::from(2), BigInt::from(3)];
    let v1 = [BigInt::from(0), BigInt::from(1)];
    let a = vec![
        Word::Noun(JArray::from_list(v1)),
        Word::Noun(JArray::from_list(v)),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(ExtIntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![0.into(), 1.into(), 2.into(), 3.into()])
                .unwrap(),
        )),
    );
}

#[test]
fn test_collect_char_nouns() {
    let v = ['c', 'd'];
    let v1 = ['a', 'b'];
    let a = vec![
        Word::Noun(JArray::from_list(v1)),
        Word::Noun(JArray::from_list(v)),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(CharArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec!['a', 'b', 'c', 'd']).unwrap(),
        )),
    );
}

#[test]
fn test_fork() {
    assert_eq!(scan_eval("(+/ % #) 1 2 3 4 5").unwrap(), Word::from(3i64));
}

#[test]
fn test_fork_noun() {
    assert_eq!(scan_eval("(15 % #) 1 2 3 4 5").unwrap(), Word::from(3));
}

#[test]
fn test_fork_average() {
    assert_eq!(scan_eval("(+/ % #) 1 2 3").unwrap(), Word::from(2));
}

#[test]
fn test_idot() {
    assert_eq!(scan_eval("i. 4").unwrap(), Noun(idot(&[4])));
    assert_eq!(scan_eval("i. 2 3").unwrap(), Noun(idot(&[2, 3])));
}

// TODO fix dyadic i. - this hook is equivalent to:
// (f g) y  ==> y f g y
// 3 1 4 1 5 9 i. # 3 1 4 1 5 9
#[test]
#[ignore]
fn test_hook() {
    assert_eq!(scan_eval("(i. #) 3 1 4 1 5 9").unwrap(), Word::from(6));
}

// TODO fix dyadic i.
#[test]
#[ignore]
fn test_idot_negative_args() {
    assert_eq!(
        scan_eval("i. _4").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[4]), vec![3, 2, 1, 0]).unwrap(),
        ))
    );
    assert_eq!(
        scan_eval("i. _2 _3").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![5, 4, 3, 2, 1, 0]).unwrap(),
        ))
    );
}

// TODO fix dyadic i.
#[test]
#[ignore]
fn test_idot_dyadic() {
    assert_eq!(scan_eval("0 1 2 3 i. 4").unwrap(), Word::from(4));

    assert_eq!(scan_eval("(i.2 3) i. 3 4 5").unwrap(), Word::from(1));
}

#[test]
fn test_assignment() {
    let mut ctx = Ctx::root();
    jr::eval(jr::scan("a =: 42").unwrap(), &mut ctx).unwrap();
    assert_eq!(
        jr::eval(jr::scan("a").unwrap(), &mut ctx).unwrap(),
        Word::from(42)
    );
}

#[test]
fn test_resolve_names() {
    let mut ctx = Ctx::root();
    let v = [3i64, 1, 4, 1, 5, 9];
    ctx.eval_mut()
        .locales
        .assign_global("a", Word::Noun(JArray::from_list(v)))
        .unwrap();

    let v = [3i64, 1, 4, 1, 5, 9];
    let words = (
        Name(String::from("a")),
        IsLocal,
        Word::Noun(JArray::from_list(v)),
        Nothing,
    );
    assert_eq!(resolve_names(words.clone(), &ctx).unwrap(), words);

    let words2 = (
        Name(String::from("b")),
        IsLocal,
        Name(String::from("a")),
        Nothing,
    );
    let v = [3i64, 1, 4, 1, 5, 9];
    assert_eq!(
        resolve_names(words2.clone(), &ctx).unwrap(),
        (
            Name(String::from("b")),
            IsLocal,
            Word::Noun(JArray::from_list(v)),
            Nothing,
        )
    );
}

#[test]
fn test_real_imaginary() -> Result<()> {
    assert_eq!(
        scan_eval("+. 5j1 6 7")?,
        Word::Noun(JArray::FloatArray(ArrayD::from_shape_vec(
            IxDyn(&[3, 2]),
            vec![5., 1., 6., 0., 7., 0.]
        )?)),
    );

    assert_eq!(
        scan_eval("+. 2 3 $ 5j1 6 7j2 8j4 9 10j6")?,
        Word::Noun(JArray::FloatArray(ArrayD::from_shape_vec(
            IxDyn(&[2, 3, 2]),
            vec![5., 1., 6., 0., 7., 2., 8., 4., 9., 0., 10., 6.]
        )?)),
    );
    Ok(())
}

#[test]
fn test_parens() {
    assert_eq!(scan_eval("(2 * 2) + 4").unwrap(), Word::from(8));
    assert_eq!(scan_eval("2 * 2 + 4").unwrap(), Word::from(12));
}

#[test]
fn test_num_dom() -> Result<()> {
    assert_eq!(
        scan_eval("2 x: 6r2 4r3 1")?,
        Word::Noun(JArray::ExtIntArray(ArrayD::from_shape_vec(
            IxDyn(&[3, 2]),
            vec![3.into(), 1.into(), 4.into(), 3.into(), 1.into(), 1.into()]
        )?)),
    );
    Ok(())
}

#[test]
fn test_behead() -> Result<()> {
    let v = [6i64, 7];
    assert_eq!(scan_eval("}. 5 6 7")?, Word::Noun(JArray::from_list(v)));

    assert_eq!(
        scan_eval("}. 3 2 $ i. 10")?,
        Word::Noun(JArray::IntArray(Array::from_shape_vec(
            IxDyn(&[2, 2]),
            [2, 3, 4, 5].to_vec()
        )?))
    );

    assert_eq!(
        scan_eval("}. 3 3 3 $ i. 30")?,
        Word::Noun(JArray::IntArray(Array::from_shape_vec(
            IxDyn(&[2, 3, 3]),
            (9..27).collect()
        )?))
    );
    Ok(())
}

#[test]
fn test_drop() -> Result<()> {
    //    2 }. 5 6 7
    // 7
    let v = [7i64];
    assert_eq!(scan_eval("2 }. 5 6 7")?, Word::Noun(JArray::from_list(v)));

    assert_eq!(
        scan_eval("2 }. i.3 3")?,
        Word::Noun(JArray::IntArray(Array::from_shape_vec(
            IxDyn(&[1, 3]),
            [6, 7, 8].to_vec()
        )?))
    );

    let v = ['a', 'b'];
    assert_eq!(scan_eval("_1 }. 'abc'")?, Word::Noun(JArray::from_list(v)));

    Ok(())
}

#[test]
fn test_drop_empty() -> Result<()> {
    //    10 }. 1 2 3
    //
    //    datatype 10 }. 1 2 3
    // integer
    assert_eq!(
        scan_eval("10 }. 1 2 3")?,
        Word::Noun(JArray::IntArray(Array::from_shape_vec(
            IxDyn(&[0]),
            [].to_vec()
        )?))
    );

    assert_eq!(
        scan_eval("10 }. ''")?,
        Word::Noun(JArray::CharArray(Array::from_shape_vec(
            IxDyn(&[0]),
            [].to_vec()
        )?))
    );

    Ok(())
}

#[test]
fn test_box() {
    // "$ < 42" == []
    assert_eq!(
        scan_eval("< 42").unwrap(),
        Word::Noun(JArray::from(arr0d(JArray::from(Num::from(42i64)))))
    );
}

#[test]
fn test_unbox() {
    assert_eq!(scan_eval("> < 42").unwrap(), Word::from(42));
}

#[test]
fn test_increment() {
    assert_eq!(s(">: 0"), Word::from(1i64));
    assert_eq!(s(">: 1"), Word::from(2i64));
    assert_eq!(s(">: 2"), Word::from(3i64));
    assert_eq!(s(&format!(">: {}", i64::MAX - 1)), Word::from(i64::MAX));
    assert_eq!(
        s(&format!(">: {}", i64::MAX)),
        Word::from((i64::MAX as f64) + 1.)
    );
    assert_eq!(
        s(">: 5r11"),
        Word::from(BigRational::new(16.into(), 11.into()))
    );
    assert_eq!(s(">: 2.7"), Word::from(3.7));
    assert_eq!(s(">: 2j1"), Word::from(Complex64::new(3., 1.)));
}

#[test]
fn test_link() {
    let v = [
        BoolArray(Array::from_elem(IxDyn(&[]), 1)),
        IntArray(Array::from_elem(IxDyn(&[]), 2)),
        IntArray(Array::from_elem(IxDyn(&[]), 3)),
    ];
    assert_eq!(
        scan_eval("1 ; 2 ; 3").unwrap(),
        Word::Noun(JArray::from_list(v))
    );
    let v = [
        BoolArray(Array::from_elem(IxDyn(&[]), 1)),
        IntArray(Array::from_elem(IxDyn(&[]), 2)),
        IntArray(Array::from_elem(IxDyn(&[]), 3)),
    ];
    assert_eq!(
        scan_eval("1 ; 2 ; <3").unwrap(),
        Word::Noun(JArray::from_list(v))
    );
}

#[test]
fn test_link_insert() {
    assert_eq!(
        scan_eval(";/i.5").unwrap(),
        scan_eval("(2-2);(2-1);2;3;4").unwrap()
    );
}

#[test]
fn test_box_equals() {
    assert_eq!(scan_eval(";/i.5").unwrap(), scan_eval("0;1;2;3;4").unwrap());

    assert_eq!(
        scan_eval("(;/i.5) = 0;1;2;3;4").unwrap(),
        scan_eval("1 1 1 1 1").unwrap()
    );
}

#[test]
fn test_jarray_rank_iter() {
    let a = idot(&[2, 3]);
    let v = a.rank_iter(0);
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![
            IntArray(Array::from_elem(IxDyn(&[]), 0)),
            IntArray(Array::from_elem(IxDyn(&[]), 1)),
            IntArray(Array::from_elem(IxDyn(&[]), 2)),
            IntArray(Array::from_elem(IxDyn(&[]), 3)),
            IntArray(Array::from_elem(IxDyn(&[]), 4)),
            IntArray(Array::from_elem(IxDyn(&[]), 5)),
        ]
    );

    let a = idot(&[2, 2, 3]);
    let v = a.rank_iter(1);
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![
            IntArray(Array::from_shape_vec(IxDyn(&[3]), vec![0, 1, 2]).unwrap()),
            IntArray(Array::from_shape_vec(IxDyn(&[3]), vec![3, 4, 5]).unwrap()),
            IntArray(Array::from_shape_vec(IxDyn(&[3]), vec![6, 7, 8]).unwrap()),
            IntArray(Array::from_shape_vec(IxDyn(&[3]), vec![9, 10, 11]).unwrap())
        ]
    );

    let a = idot(&[2, 2, 3]);
    let v = a.rank_iter(2);
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![
            idot(&[2, 3]),
            IntArray(6i64 + Array::from_shape_vec(IxDyn(&[2, 3]), (0..6).collect()).unwrap()),
        ]
    );

    let a = idot(&[2, 2, 3]);
    let v = a.rank_iter(3);
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(v, vec![idot(&[2, 2, 3])]);

    let a = idot(&[2, 2, 3]);
    let v = a.rank_iter(Rank::infinite().raw_u8().into());
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(v, vec![idot(&[2, 2, 3])]);
}

#[test]
fn test_rank_conjunction_1_1() {
    // Sum each row independently
    //    (+/"1) i.2 3
    // 3 12

    let v = [3i64, 12];
    assert_eq!(
        scan_eval("+/\"1 i.2 3").unwrap(),
        Word::Noun(JArray::from_list(v))
    );
}

#[test]
fn test_rank_conjunction_0_1() {
    // Add each atom of x to each vector of y
    //    1 2 (+"0 1) 1 2 3
    // 2 3 4
    // 3 4 5

    assert_eq!(
        scan_eval("1 2 (+\"0 1) 1 2 3").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![2i64, 3, 4, 3, 4, 5]).unwrap(),
        ))
    );
}

#[test]
fn test_agreement_plus_rank_0_1() {
    // Add each atom of x to each vector of y (same as test_rank_conjunction_0_1() but without the rank conjunction)
    assert_eq!(
        scan_eval("1 2 (+\"0 1) 1 2 3").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![2i64, 3, 4, 3, 4, 5]).unwrap(),
        ))
    );
}

#[test]
fn test_rank_conjunction_1_0() {
    // Add each vector of x to each atom of y
    //    1 2 (+"1 0) 1 2 3
    // 2 3
    // 3 4
    // 4 5
    assert_eq!(
        scan_eval("1 2 (+\"1 0) 1 2 3").unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3, 2]), vec![2, 3, 3, 4, 4, 5]).unwrap(),
        ))
    );
}

#[test]
fn test_head() -> Result<()> {
    assert_eq!(scan_eval("{. 'abc'")?, Word::from('a'));

    assert_eq!(scan_eval("{. 1 2 3")?, Word::from(1i64));

    let v = [0i64, 1, 2];
    assert_eq!(scan_eval("{. i.2 3")?, Word::Noun(JArray::from_list(v)));

    let v = [0i64, 1, 2];
    assert_eq!(scan_eval("{. i.3 3")?, Word::Noun(JArray::from_list(v)));
    Ok(())
}

#[test]
fn test_take() -> Result<()> {
    let v = [1i64];
    assert_eq!(scan_eval("1 {. 1 2 3")?, Word::Noun(JArray::from_list(v)));

    let v = [1u8];
    assert_eq!(scan_eval("1 {. 1")?, Word::Noun(JArray::from_list(v)));

    let v = [1i64, 2];
    assert_eq!(scan_eval("2 {. 1 2 3")?, Word::Noun(JArray::from_list(v)));

    assert_eq!(
        scan_eval("2 {. i.3 3")?,
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![0, 1, 2, 3, 4, 5]).unwrap(),
        ))
    );

    let v = [2i64, 3];
    assert_eq!(scan_eval("_2 {. 1 2 3")?, Word::Noun(JArray::from_list(v)));

    Ok(())
}

#[test]
fn test_take_agreement() -> Result<()> {
    assert_eq!(
        scan_eval("(2 1 $ 2) {. 'abcdef'")?,
        Noun(CharArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec!['a', 'b', 'a', 'b']).unwrap(),
        ))
    );

    Ok(())
}

#[test]
#[ignore]
fn test_take_framingfill() -> Result<()> {
    // TODO Fix Framing Fill here
    let v = [1i64, 0, 0];
    assert_eq!(scan_eval("3 {. 1")?, Word::Noun(JArray::from_list(v)));

    Ok(())
}

#[test]
fn test_cat() -> Result<()> {
    assert_eq!(scan_eval("+:@- 7")?, Word::from(-14i64));
    assert_eq!(scan_eval("3 +:@- 7")?, Word::from(-8i64));
    Ok(())
}

#[test]
fn test_tail() -> Result<()> {
    assert_eq!(scan_eval("{: 'abc'")?, Word::from('c'));

    assert_eq!(scan_eval("{: 1 2 3")?, Word::from(3i64));

    let v = [3i64, 4, 5];
    assert_eq!(scan_eval("{: i.2 3")?, Word::Noun(JArray::from_list(v)));

    let v = [6i64, 7, 8];
    assert_eq!(scan_eval("{: i.3 3")?, Word::Noun(JArray::from_list(v)));
    Ok(())
}

#[test]
fn test_curtail() -> Result<()> {
    let v = ['a', 'b'];
    assert_eq!(scan_eval("}: 'abc'")?, Word::Noun(JArray::from_list(v)));

    let v = [1i64, 2];
    assert_eq!(scan_eval("}: 1 2 3")?, Word::Noun(JArray::from_list(v)));

    assert_eq!(
        scan_eval("}: i.2 3")?,
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[1, 3]), vec![0, 1, 2]).unwrap(),
        ))
    );

    assert_eq!(
        scan_eval("}: i.3 3")?,
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![0, 1, 2, 3, 4, 5]).unwrap(),
        ))
    );
    Ok(())
}

#[test]
fn test_ravel() -> Result<()> {
    let v = [0i64, 1, 2, 3];
    assert_eq!(scan_eval(", i.2 2")?, Word::Noun(JArray::from_list(v)));

    assert_eq!(scan_eval(", i.2 3 4")?, scan_eval("i.24")?,);
    Ok(())
}

#[test]
fn test_user_defined_dyadic_verb() -> Result<()> {
    assert_eq!(scan_eval("2 (4 : 'x + y') 2").unwrap(), Word::from(4i64));

    let err = scan_eval("(4 : 'x + y') 2").unwrap_err();
    let root = dbg!(err.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(root, JError::DomainError));

    Ok(())
}

#[test]
fn test_user_defined_monadic_verb() -> Result<()> {
    assert_eq!(scan_eval("(3 : '2 * y') 2").unwrap(), Word::from(4i64));

    let err = scan_eval("2 (3 : '2 * y') 2").unwrap_err();
    let root = dbg!(err.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(root, JError::DomainError));

    Ok(())
}
