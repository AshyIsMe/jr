use jr::verbs::reshape;
use jr::JArray::*;
use jr::Word::*;
use jr::{collect_nouns, resolve_names, JError, ModifierImpl, VerbImpl, Word};
use ndarray::prelude::*;
use std::collections::HashMap;

#[test]
fn test_basic_addition() {
    let words = jr::scan("2 + 2").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[1]), 4)))
    );

    let words = jr::scan("1 2 3 + 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[3]), vec![5, 7, 9]).unwrap()
        ))
    );

    let words = jr::scan("1 + 3.14").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(FloatArray(Array::from_elem(IxDyn(&[1]), 1.0 + 3.14)))
    );
}

#[test]
fn test_basic_times() {
    let words = jr::scan("2 * 2").unwrap();
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[1]), 4)))
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
            Array::from_shape_vec(IxDyn(&[1]), vec![2]).unwrap(),
        )),
        Verb(String::from("+"), VerbImpl::Plus),
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
        Noun(IntArray(Array::from_elem(IxDyn(&[4]), 1)))
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
fn test_power_conjunction_bool_arg() {
    //let words = jr::scan("(*:^:2) 4").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(String::from("*:"), VerbImpl::StarCo),
        Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Noun(BoolArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        )),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 4))),
    ];
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![4, 16]).unwrap(),
        ))
    );
}

#[test]
fn test_power_conjunction_noun_arg() {
    //let words = jr::scan("(*:^:2) 4").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(String::from("*:"), VerbImpl::StarCo),
        Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 2))),
        Noun(IntArray(Array::from_elem(IxDyn(&[]), 4))),
    ];
    // TODO Should the result be an atom 256 here? rather than an array of shape 1?
    assert_eq!(
        jr::eval(words, &mut HashMap::new()).unwrap(),
        Noun(IntArray(Array::from_elem(IxDyn(&[1]), 256)))
    );

    let words = vec![
        Verb(String::from("*:"), VerbImpl::StarCo),
        Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        )),
        Noun(IntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        )),
    ];
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
            Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        )),
        Noun(ExtIntArray(
            Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        )),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(ExtIntArray(
            Array::from_shape_vec(IxDyn(&[2, 2]), vec![0, 1, 2, 3]).unwrap(),
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
            l: Box::new(Verb(String::from("+"), VerbImpl::Plus)),
            r: Box::new(Nothing),
            m: Box::new(Adverb(String::from("/"), ModifierImpl::Slash)),
        },
    );
    let words = vec![
        Verb(
            String::from("+/%#"),
            VerbImpl::Fork {
                f: Box::new(sum),
                g: Box::new(Verb(String::from("%"), VerbImpl::Percent)),
                h: Box::new(Verb(String::from("#"), VerbImpl::Number)),
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
                g: Box::new(Verb(String::from("%"), VerbImpl::Percent)),
                h: Box::new(Verb(String::from("#"), VerbImpl::Number)),
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
fn test_hook() {
    //let words = jr::scan("(i. #) 3 1 4 1 5 9").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(
            String::from("i.#"),
            VerbImpl::Hook {
                l: Box::new(Verb(String::from("i."), VerbImpl::IDot)),
                r: Box::new(Verb(String::from("#"), VerbImpl::Number)),
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
//#[test]
//fn test_idot_negative_args() {
//assert_eq!(
//jr::eval(jr::scan("i. _4").unwrap()).unwrap(),
//Noun(IntArray {
//a: Array::from_shape_vec(IxDyn(&[4]), vec![3, 2, 1, 0]).unwrap(),
//})
//);
//assert_eq!(
//jr::eval(jr::scan("i. _2 _3").unwrap()).unwrap(),
//Noun(IntArray {
//a: Array::from_shape_vec(IxDyn(&[2, 3]), vec![5, 4, 3, 2, 1, 0]).unwrap(),
//})
//);
//}

// TODO fix dyadic i.
//#[test]
//fn test_idot_dyadic() {
//assert_eq!(
//jr::eval(jr::scan("0 1 2 3 i. 4").unwrap()).unwrap(),
//Noun(IntArray {
//a: Array::from_shape_vec(IxDyn(&[1]), vec![4]).unwrap(),
//})
//);

////let words = jr::scan("(i.2 3) i. 3 4 5").unwrap(); //TODO use this when parens are implemented
//let words = vec![
//Noun(IntArray {
//a: Array::from_shape_vec(IxDyn(&[2, 3]), vec![0, 1, 2, 3, 4, 5]).unwrap(),
//}),
//Verb(String::from("i."), VerbImpl::IDot),
//Noun(IntArray {
//a: Array::from_shape_vec(IxDyn(&[3]), vec![3, 4, 5]).unwrap(),
//}),
//];
//assert_eq!(
//jr::eval(words).unwrap(),
//Noun(IntArray {
//a: Array::from_shape_vec(IxDyn(&[1]), vec![1]).unwrap(),
//})
//);
//}

#[test]
fn test_assignment() {
    let mut names = HashMap::new();
    assert_eq!(
        jr::eval(jr::scan("a =: 42").unwrap(), &mut names).unwrap(),
        Word::noun([42i64]).unwrap()
    );
    assert_eq!(
        jr::eval(jr::scan("a").unwrap(), &mut names).unwrap(),
        Word::noun([42i64]).unwrap()
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
