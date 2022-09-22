use jr::verbs::reshape;
use jr::JArray::*;
use jr::{collect_nouns, JError, ModifierImpl, VerbImpl, Word};
use ndarray::prelude::*;

#[test]
fn test_basic_addition() {
    let words = jr::scan("2 + 2").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[1]), 4)
        })
    );

    let words = jr::scan("1 2 3 + 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![5, 7, 9]).unwrap()
        })
    );

    let words = jr::scan("1 + 3.14").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(FloatArray {
            a: Array::from_elem(IxDyn(&[1]), 1.0 + 3.14)
        })
    );
}

#[test]
fn test_basic_times() {
    let words = jr::scan("2 * 2").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[1]), 4)
        })
    );

    let words = jr::scan("1 2 3 * 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![4, 10, 18]).unwrap()
        })
    );
}

#[test]
fn test_parse_basics() {
    let words = vec![
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[1]), vec![2]).unwrap(),
        }),
        Word::Verb(String::from("+"), VerbImpl::Plus),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![1, 2, 3]).unwrap(),
        }),
    ];
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![3, 4, 5]).unwrap()
        })
    );
}

#[test]
fn test_insert_adverb() {
    let words = jr::scan("+/1 2 3").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 6)
        })
    );
}

#[test]
fn test_reshape() {
    let words = jr::scan("2 2 $ 1 2 3 4").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![1, 2, 3, 4]).unwrap()
        })
    );

    let words = jr::scan("4 $ 1").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[4]), 1)
        })
    );

    let words = jr::scan("1 2 3 $ 1 2").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[1, 2, 3]), vec![1, 2, 1, 2, 1, 2]).unwrap()
        })
    );

    let words = jr::scan("3 $ 2 2 $ 0 1 2 3").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3, 2]), vec![0, 1, 2, 3, 0, 1]).unwrap()
        })
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
        Word::Verb(String::from("*:"), VerbImpl::StarCo),
        Word::Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Word::Noun(BoolArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        }),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 4),
        }),
    ];
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![4, 16]).unwrap(),
        })
    );
}

#[test]
fn test_power_conjunction_noun_arg() {
    //let words = jr::scan("(*:^:2) 4").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Word::Verb(String::from("*:"), VerbImpl::StarCo),
        Word::Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 2),
        }),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 4),
        }),
    ];
    // TODO Should the result be an atom 256 here? rather than an array of shape 1?
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_elem(IxDyn(&[1]), 256)
        })
    );

    let words = vec![
        Word::Verb(String::from("*:"), VerbImpl::StarCo),
        Word::Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
    ];
    assert_eq!(
        jr::eval(words).unwrap(),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![16, 81, 256, 6561]).unwrap(),
        }),
    );
}

#[test]
fn test_collect_int_nouns() {
    let a = vec![
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        }),
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
    ];
    //let result = collect_int_nouns(a).unwrap();
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Word::Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![0, 1, 2, 3]).unwrap(),
        }),
    );
}

#[test]
fn test_collect_extint_nouns() {
    let a = vec![
        Word::Noun(ExtIntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        }),
        Word::Noun(ExtIntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Word::Noun(ExtIntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![0, 1, 2, 3]).unwrap(),
        }),
    );
}

#[test]
fn test_collect_char_nouns() {
    let a = vec![
        Word::Noun(CharArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec!['a', 'b']).unwrap(),
        }),
        Word::Noun(CharArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec!['c', 'd']).unwrap(),
        }),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Word::Noun(CharArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec!['a', 'b', 'c', 'd']).unwrap(),
        }),
    );
}

#[test]
fn test_TEMP_range() {
    // TODO DELETE
    assert_eq!((0..5), std::ops::Range { start: 0, end: 5 });
    assert_eq!(
        (0..5).collect::<Vec<i64>>(),
        std::ops::Range { start: 0, end: 5 }.collect::<Vec<i64>>()
    );
}

#[test]
fn test_TEMP_compare() {
    // TODO DELETE
    let a = Array::from_shape_vec(IxDyn(&[2]), vec![1, 2]).unwrap();
    let b = Array::from_shape_vec(IxDyn(&[3]), vec![2, 3, 4]).unwrap();
    assert!(a.shape() <= b.shape(),);
}
