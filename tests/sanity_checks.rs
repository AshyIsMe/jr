use num::BigInt;
use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use ndarray::prelude::*;

use jr::verbs::reshape;
use jr::JArray::*;
use jr::Word::*;
use jr::{
    arr0d, collect_nouns, eval, resolve_names, scan, IntoJArray, JArray, ModifierImpl, Rank,
    VerbImpl, Word,
};
use log::debug;

// AA TODO scan_eval and idot don't live here, was too lazy to get git to cherry pick nicely
//use jr::test_impls::{idot, scan_eval};
pub fn scan_eval(sentence: &str) -> Result<Word> {
    let tokens = crate::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    crate::eval(tokens, &mut HashMap::new()).with_context(|| anyhow!("evaluating {:?}", sentence))
}

pub fn idot(s: &[usize]) -> JArray {
    let p = s.iter().map(|i| *i as i64).product();
    //Noun(IntArray(ArrayD::from_shape_vec(IxDyn(&s), (0..p).collect()).unwrap(),))
    IntArray(ArrayD::from_shape_vec(IxDyn(&s), (0..p).collect()).unwrap())
}

#[test]
fn test_basic_addition() {
    assert_eq!(scan_eval("2 + 2").unwrap(), Word::from(4i64));

    assert_eq!(
        scan_eval("1 2 3 + 4 5 6").unwrap(),
        Word::noun([5i64, 7, 9]).unwrap()
    );

    assert_eq!(scan_eval("1 + 3.14").unwrap(), Word::from(1f64 + 3.14));
}

#[test]
fn test_basic_times() {
    assert_eq!(scan_eval("2 * 2").unwrap(), Word::from(4));

    assert_eq!(
        scan_eval("1 2 3 * 4 5 6").unwrap(),
        Word::noun([4i64, 10, 18]).unwrap()
    );
}

#[test]
fn test_parse_basics() {
    let words = vec![
        Word::from(2),
        Word::static_verb("+"),
        Word::noun([1i64, 2, 3]).unwrap(),
    ];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::noun([3i64, 4, 5]).unwrap(),
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
fn test_reshape_helper() {
    let y = Array::from_elem(IxDyn(&[1]), 1);
    let r = reshape(&Array::from_elem(IxDyn(&[1]), 4), &y).unwrap();
    assert_eq!(r, Array::from_elem(IxDyn(&[4]), 1));
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
    assert_eq!(r1, Word::noun([1i64, 2, 3, 1, 2, 3]).unwrap());
    Ok(())
}

#[test]
fn test_power_conjunction_bool_arg() {
    assert_eq!(
        scan_eval("(*:^:0 1) 4").unwrap(),
        Word::noun([4i64, 16]).unwrap()
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
    let a = vec![
        Word::noun([0i64, 1]).unwrap(),
        Word::noun([2i64, 3]).unwrap(),
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
    let a = vec![
        Word::noun([BigInt::from(0), BigInt::from(1)]).unwrap(),
        Word::noun([BigInt::from(2), BigInt::from(3)]).unwrap(),
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
    let a = vec![
        Word::noun(['a', 'b']).unwrap(),
        Word::noun(['c', 'd']).unwrap(),
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
    let words = jr::scan("(+/ % #) 1 2 3 4 5").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::from(3i64)
    );
}

#[test]
fn test_fork_noun() {
    let words = jr::scan("(15 % #) 1 2 3 4 5").unwrap();
    assert_eq!(jr::eval(words, &mut HashMap::new()).unwrap(), Word::from(3));
}

#[test]
fn test_fork_average() {
    let words = jr::scan("(+/ % #) 1 2 3").unwrap();
    assert_eq!(jr::eval(words, &mut HashMap::new()).unwrap(), Word::from(2));
}

#[test]
fn test_idot() {
    assert_eq!(
        jr::eval(jr::scan("i. 4").unwrap(), &mut HashMap::new()).unwrap(),
        Noun(idot(&[4]))
    );
    assert_eq!(
        jr::eval(jr::scan("i. 2 3").unwrap(), &mut HashMap::new()).unwrap(),
        Noun(idot(&[2, 3]))
    );
}

// TODO fix dyadic i. - this hook is equivalent to:
// (f g) y  ==> y f g y
// 3 1 4 1 5 9 i. # 3 1 4 1 5 9
#[test]
#[ignore]
fn test_hook() {
    let words = jr::scan("(i. #) 3 1 4 1 5 9").unwrap();
    assert_eq!(jr::eval(words, &mut HashMap::new()).unwrap(), Word::from(6));
}

// TODO fix dyadic i.
#[test]
#[ignore]
fn test_idot_negative_args() {
    assert_eq!(
        jr::eval(jr::scan("i. _4").unwrap(), &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[4]), vec![3, 2, 1, 0]).unwrap(),
        ))
    );
    assert_eq!(
        jr::eval(jr::scan("i. _2 _3").unwrap(), &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![5, 4, 3, 2, 1, 0]).unwrap(),
        ))
    );
}

// TODO fix dyadic i.
#[test]
#[ignore]
fn test_idot_dyadic() {
    assert_eq!(
        jr::eval(jr::scan("0 1 2 3 i. 4").unwrap(), &mut HashMap::new()).unwrap(),
        Word::from(4)
    );

    let words = jr::scan("(i.2 3) i. 3 4 5").unwrap();
    assert_eq!(jr::eval(words, &mut HashMap::new()).unwrap(), Word::from(1));
}

#[test]
fn test_assignment() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("a =: 42").unwrap(), &mut names).unwrap(),
        Word::from(42)
    );
    assert_eq!(
        jr::eval(jr::scan("a").unwrap(), &mut names).unwrap(),
        Word::from(42)
    );
}

#[test]
fn test_resolve_names() {
    let mut names = HashMap::new();
    names.insert(
        String::from("a"),
        Word::noun([3i64, 1, 4, 1, 5, 9]).unwrap(),
    );

    let words = (
        Name(String::from("a")),
        IsLocal,
        Word::noun([3i64, 1, 4, 1, 5, 9]).unwrap(),
        Nothing,
    );
    assert_eq!(resolve_names(words.clone(), names.clone()), words);

    let words2 = (
        Name(String::from("b")),
        IsLocal,
        Name(String::from("a")),
        Nothing,
    );
    assert_eq!(
        resolve_names(words2.clone(), names.clone()),
        (
            Name(String::from("b")),
            IsLocal,
            Word::noun([3i64, 1, 4, 1, 5, 9]).unwrap(),
            Nothing,
        )
    );
}

#[test]
fn test_real_imaginary() -> Result<()> {
    assert_eq!(
        jr::eval(jr::scan("+. 5j1 6 7")?, &mut HashMap::new())?,
        Word::Noun(JArray::FloatArray(ArrayD::from_shape_vec(
            IxDyn(&[3, 2]),
            vec![5., 1., 6., 0., 7., 0.]
        )?)),
    );

    assert_eq!(
        jr::eval(
            jr::scan("+. 2 3 $ 5j1 6 7j2 8j4 9 10j6")?,
            &mut HashMap::new()
        )?,
        Word::Noun(JArray::FloatArray(ArrayD::from_shape_vec(
            IxDyn(&[2, 3, 2]),
            vec![5., 1., 6., 0., 7., 2., 8., 4., 9., 0., 10., 6.]
        )?)),
    );
    Ok(())
}

#[test]
fn test_parens() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("(2 * 2) + 4").unwrap(), &mut names).unwrap(),
        Word::from(8)
    );
    assert_eq!(
        jr::eval(jr::scan("2 * 2 + 4").unwrap(), &mut names).unwrap(),
        Word::from(12)
    );
}

#[test]
fn test_num_dom() -> Result<()> {
    assert_eq!(
        jr::eval(jr::scan("2 x: 6r2 4r3 1")?, &mut HashMap::new())?,
        Word::Noun(JArray::ExtIntArray(ArrayD::from_shape_vec(
            IxDyn(&[3, 2]),
            vec![3.into(), 1.into(), 4.into(), 3.into(), 1.into(), 1.into()]
        )?)),
    );
    Ok(())
}

#[test]
fn test_behead() -> Result<()> {
    assert_eq!(
        jr::eval(jr::scan("}. 5 6 7")?, &mut HashMap::new())?,
        Word::noun([6i64, 7])?
    );

    assert_eq!(
        jr::eval(jr::scan("}. 3 2 $ i. 10")?, &mut HashMap::new())?,
        Word::Noun(JArray::IntArray(Array::from_shape_vec(
            IxDyn(&[2, 2]),
            [2, 3, 4, 5].to_vec()
        )?))
    );

    assert_eq!(
        jr::eval(jr::scan("}. 3 3 3 $ i. 30")?, &mut HashMap::new())?,
        Word::Noun(JArray::IntArray(Array::from_shape_vec(
            IxDyn(&[2, 3, 3]),
            (9..27).collect()
        )?))
    );
    Ok(())
}

#[test]
fn test_box() {
    // "$ < 42" == []
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("< 42").unwrap(), &mut names).unwrap(),
        Word::noun(arr0d(Word::from(42))).unwrap()
    );
}

#[test]
fn test_unbox() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("> < 42").unwrap(), &mut names).unwrap(),
        Word::from(42)
    );
}

#[test]
fn test_link() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("1 ; 2 ; 3").unwrap(), &mut names).unwrap(),
        Word::noun([
            Noun(BoolArray(Array::from_elem(IxDyn(&[]), 1))),
            Noun(IntArray(Array::from_elem(IxDyn(&[]), 2))),
            Noun(IntArray(Array::from_elem(IxDyn(&[]), 3))),
        ])
        .unwrap()
    );
    assert_eq!(
        jr::eval(jr::scan("1 ; 2 ; <3").unwrap(), &mut names).unwrap(),
        Word::noun([
            Noun(BoolArray(Array::from_elem(IxDyn(&[]), 1))),
            Noun(IntArray(Array::from_elem(IxDyn(&[]), 2))),
            Noun(IntArray(Array::from_elem(IxDyn(&[]), 3))),
        ])
        .unwrap()
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

    assert_eq!(
        scan_eval("+/\"1 i.2 3").unwrap(),
        Word::noun([3i64, 12]).unwrap()
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
    //    1 2 (+"0 1) 1 2 3
    let x = array![1i64, 2].into_dyn().into_noun();
    let y = Word::noun([1i64, 2, 3]).unwrap();

    use jr::verbs::*;

    // +"0 1
    let f = Word::Verb(
        "+\"0 1".to_string(),
        VerbImpl::Primitive(PrimitiveImpl::new(
            "+",
            v_conjugate,
            v_plus,
            (Rank::zero(), Rank::zero(), Rank::one()),
        )),
    );

    let words = vec![x, f, y];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
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
    let words = jr::scan("1 2 (+\"1 0) 1 2 3").unwrap();
    println!("words: {:?}", words);

    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3, 2]), vec![2, 3, 3, 4, 4, 5]).unwrap(),
        ))
    );
}
