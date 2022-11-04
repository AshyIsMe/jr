use std::collections::HashMap;

use anyhow::Result;
use ndarray::prelude::*;

use jr::verbs::reshape;
use jr::JArray::*;
use jr::Rank;
use jr::Word::*;
use jr::{collect_nouns, resolve_names, JArray, ModifierImpl, VerbImpl, Word};

#[test]
fn test_basic_addition() {
    let words = jr::scan("2 + 2").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 4)))
    );

    let words = jr::scan("1 2 3 + 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::noun([5i64, 7, 9]).unwrap()
    );

    let words = jr::scan("1 + 3.14").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(FloatArray(Array::from_elem(IxDyn(&[]), 1f64 + 3.14)))
    );
}

#[test]
fn test_basic_times() {
    let words = jr::scan("2 * 2").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 4)))
    );

    let words = jr::scan("1 2 3 * 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3]), vec![4, 10, 18]).unwrap()
        ))
    );
}

#[test]
fn test_parse_basics() {
    let words = vec![
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[]), vec![2]).unwrap(),
        )),
        Word::static_verb("+"),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3]), vec![1, 2, 3]).unwrap(),
        )),
    ];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3]), vec![3, 4, 5]).unwrap()
        ))
    );
}

#[test]
fn test_insert_adverb() {
    let words = jr::scan("+/1 2 3").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 6)))
    );
}

#[test]
fn test_reshape() {
    let words = jr::scan("2 2 $ 1 2 3 4").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![1, 2, 3, 4]).unwrap()
        ))
    );

    let words = jr::scan("4 $ 1").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(BoolArray(Array::from_elem(IxDyn(&[4]), 1)))
    );

    let words = jr::scan("1 2 3 $ 1 2").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[1, 2, 3]), vec![1, 2, 1, 2, 1, 2]).unwrap()
        ))
    );

    let words = jr::scan("3 $ 2 2 $ 0 1 2 3").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
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
    let words = jr::scan("2 1 $ 1 2").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 1]), vec![1, 2]).unwrap()
        ))
    );
}

#[test]
fn test_reshape_2d_match_1d() -> Result<()> {
    let r1 = jr::eval(jr::scan("(2 2 $ 3) $ 1")?, &mut HashMap::new()).unwrap();
    let r2 = jr::eval(jr::scan("2 3 3 $ 1")?, &mut HashMap::new()).unwrap();

    let correct_result = Noun(BoolArray(Array::from_elem(IxDyn(&[2, 3, 3]), 1)));

    assert_eq!(r1, correct_result);
    assert_eq!(r2, correct_result);
    assert_eq!(r1, r2);

    Ok(())
}

#[test]
fn test_reshape_no_change() -> Result<()> {
    let r1 = jr::eval(jr::scan("i.2 3 4")?, &mut HashMap::new()).unwrap();
    let r2 = jr::eval(jr::scan("2 $ i.2 3 4")?, &mut HashMap::new()).unwrap();

    assert_eq!(r1, r2);

    Ok(())
}

#[test]
fn test_agreement_reshape_3() -> Result<()> {
    let r1 = jr::eval(jr::scan("6 $ i.2 3")?, &mut HashMap::new()).unwrap();
    // 6 3 $ 0 1 2 3 4 5 0 1 2 3 4 5 0 1 2 3 4 5
    let a = Array::from_shape_vec(
        IxDyn(&[6, 3]),
        vec![0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5, 0, 1, 2, 3, 4, 5],
    )?;

    assert_eq!(r1, Noun(IntArray(a)));

    Ok(())
}

#[test]
fn test_reshape_atoms() -> Result<()> {
    let r1 = jr::eval(jr::scan("1 $ 1")?, &mut HashMap::new()).unwrap();
    // Should be an array of length 1 containing 1
    assert_eq!(r1, Noun(IntArray(Array::from_elem(IxDyn(&[1]), 1))));
    Ok(())
}

#[test]
fn test_reshape_truncate() -> Result<()> {
    let r1 = jr::eval(jr::scan("1 $ 1 2 3")?, &mut HashMap::new()).unwrap();
    assert_eq!(r1, Noun(IntArray(Array::from_elem(IxDyn(&[]), 1))));
    Ok(())
}

#[test]
fn test_reshape_cycle() -> Result<()> {
    let r1 = jr::eval(jr::scan("6 $ 1 2 3")?, &mut HashMap::new()).unwrap();
    assert_eq!(r1, Word::noun([1i64, 2, 3, 1, 2, 3]).unwrap());
    Ok(())
}

#[test]
fn test_power_conjunction_bool_arg() {
    let words = jr::scan("(*:^:0 1) 4").unwrap();
    println!("words: {:?}", words);

    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![4, 16]).unwrap(),
        ))
    );
}

#[test]
fn test_power_conjunction_noun_arg() {
    let words = jr::scan("(*:^:2) 4").unwrap();
    println!("words: {:?}", words);
    //TODO Result should be an atom 256 here, rather than an array of shape 1.
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 256)))
    );

    let words = jr::scan("(*:^:2 3) 2 3").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![16, 81, 256, 6561]).unwrap(),
        )),
    );
}

#[test]
fn test_collect_int_nouns() {
    let a = vec![
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        )),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        )),
    ];
    //let result = collect_int_nouns(a).unwrap();
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
        Noun(ExtIntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![0.into(), 1.into()]).unwrap(),
        )),
        Noun(ExtIntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![2.into(), 3.into()]).unwrap(),
        )),
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
        Noun(CharArray(
            Array::from_shape_vec(IxDyn(&[2]), vec!['a', 'b']).unwrap(),
        )),
        Noun(CharArray(
            Array::from_shape_vec(IxDyn(&[2]), vec!['c', 'd']).unwrap(),
        )),
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
    //let words = jr::scan("(+/ % #) 1 2 3 4 5").unwrap(); //TODO use this when parens are implemented
    let sum = Verb(
        String::from("+/"),
        VerbImpl::DerivedVerb {
            l: Box::new(Word::static_verb("+")),
            r: Box::new(Nothing),
            m: Box::new(Adverb(String::from("/"), ModifierImpl::Slash)),
        },
    );
    let words = vec![
        Verb(
            String::from("+/%#"),
            VerbImpl::Fork {
                f: Box::new(sum),
                g: Box::new(Word::static_verb("%")),
                h: Box::new(Word::static_verb("#")),
            },
        ),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[5]), vec![1, 2, 3, 4, 5]).unwrap(),
        )),
    ];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::noun([3i64]).unwrap()
    );
}

#[test]
fn test_fork_noun() {
    //let words = jr::scan("(15 % #) 1 2 3 4 5").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(
            String::from("+/%#"),
            VerbImpl::Fork {
                f: Box::new(Word::noun(vec![15i64]).unwrap()),
                g: Box::new(Word::static_verb("%")),
                h: Box::new(Word::static_verb("#")),
            },
        ),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[5]), vec![1, 2, 3, 4, 5]).unwrap(),
        )),
    ];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::noun(vec![3i64]).unwrap()
    );
}

#[test]
fn test_fork_average() {
    let words = jr::scan("(+/ % #) 1 2 3").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::noun([2i64]).unwrap()
    );
}

#[test]
fn test_hook() {
    //let words = jr::scan("(i. #) 3 1 4 1 5 9").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(
            String::from("i.#"),
            VerbImpl::Hook {
                l: Box::new(Word::static_verb("i.")),
                r: Box::new(Word::static_verb("#")),
            },
        ),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[6]), vec![3, 1, 4, 1, 5, 9]).unwrap(),
        )),
    ];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Word::noun(vec![6i64]).unwrap()
    );
}

#[test]
fn test_idot() {
    assert_eq!(
        jr::eval(jr::scan("i. 4").unwrap(), &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[4]), vec![0, 1, 2, 3]).unwrap(),
        ))
    );
    assert_eq!(
        jr::eval(jr::scan("i. 2 3").unwrap(), &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2, 3]), vec![0, 1, 2, 3, 4, 5]).unwrap(),
        ))
    );
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
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[1]), vec![4]).unwrap(),
        ))
    );

    let words = jr::scan("(i.2 3) i. 3 4 5").unwrap(); //TODO use this when parens are implemented
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[1]), vec![1]).unwrap(),
        ))
    );
}

#[test]
fn test_assignment() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("a =: 42").unwrap(), &mut names).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 42)))
    );
    assert_eq!(
        jr::eval(jr::scan("a").unwrap(), &mut names).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 42)))
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
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 8)))
    );
    assert_eq!(
        jr::eval(jr::scan("2 * 2 + 4").unwrap(), &mut names).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 12)))
    );
}

#[test]
fn test_num_dom() -> Result<()> {
    assert_eq!(
        jr::eval(jr::scan("2 x: 5r1 4r2 1")?, &mut HashMap::new())?,
        Word::Noun(JArray::ExtIntArray(ArrayD::from_shape_vec(
            IxDyn(&[3, 2]),
            vec![5.into(), 1.into(), 2.into(), 1.into(), 1.into(), 1.into()]
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
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("< 42").unwrap(), &mut names).unwrap(),
        Word::noun([Noun(IntArray(Array::from_elem(IxDyn(&[]), 42)))]).unwrap()
    );
}

#[test]
fn test_unbox() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("> < 42").unwrap(), &mut names).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 42)))
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
    let a = IntArray(Array::from_shape_vec(IxDyn(&[2, 3]), (0..6).collect()).unwrap());
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

    let a = IntArray(Array::from_shape_vec(IxDyn(&[2, 2, 3]), (0..12).collect()).unwrap());
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

    let a = IntArray(Array::from_shape_vec(IxDyn(&[2, 2, 3]), (0..12).collect()).unwrap());
    let v = a.rank_iter(2);
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![
            IntArray(Array::from_shape_vec(IxDyn(&[2, 3]), (0..6).collect()).unwrap()),
            IntArray(6i64 + Array::from_shape_vec(IxDyn(&[2, 3]), (0..6).collect()).unwrap()),
        ]
    );

    let a = IntArray(Array::from_shape_vec(IxDyn(&[2, 2, 3]), (0..12).collect()).unwrap());
    let v = a.rank_iter(3);
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2, 3]), (0..12).collect()).unwrap()
        )]
    );

    let a = IntArray(Array::from_shape_vec(IxDyn(&[2, 2, 3]), (0..12).collect()).unwrap());
    let v = a.rank_iter(Rank::infinite().raw_u8());
    println!("v.len(): {}", v.len());
    println!("{:?}", v);
    assert_eq!(
        v,
        vec![IntArray(
            Array::from_shape_vec(IxDyn(&[2, 2, 3]), (0..12).collect()).unwrap()
        )]
    );
}
