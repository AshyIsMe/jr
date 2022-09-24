use jr::verbs::reshape;
use jr::JArray::*;
use jr::Word::*;
use jr::{collect_nouns, int_array, JError, ModifierImpl, VerbImpl, Word};
use ndarray::prelude::*;

#[test]
fn test_basic_addition() {
    let words = jr::scan("2 + 2").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[1]), 4)
        })
    );

    let words = jr::scan("1 2 3 + 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![5, 7, 9]).unwrap()
        })
    );

    let words = jr::scan("1 + 3.14").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(FloatArray {
            a: Array::from_elem(IxDyn(&[1]), 1.0 + 3.14)
        })
    );
}

#[test]
fn test_basic_times() {
    let words = jr::scan("2 * 2").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[1]), 4)
        })
    );

    let words = jr::scan("1 2 3 * 4 5 6").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![4, 10, 18]).unwrap()
        })
    );
}

#[test]
fn test_parse_basics() {
    let words = vec![
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[1]), vec![2]).unwrap(),
        }),
        Verb(String::from("+"), VerbImpl::Plus),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![1, 2, 3]).unwrap(),
        }),
    ];
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[3]), vec![3, 4, 5]).unwrap()
        })
    );
}

#[test]
fn test_insert_adverb() {
    let words = jr::scan("+/1 2 3").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 6)
        })
    );
}

#[test]
fn test_reshape() {
    let words = jr::scan("2 2 $ 1 2 3 4").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![1, 2, 3, 4]).unwrap()
        })
    );

    let words = jr::scan("4 $ 1").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[4]), 1)
        })
    );

    let words = jr::scan("1 2 3 $ 1 2").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[1, 2, 3]), vec![1, 2, 1, 2, 1, 2]).unwrap()
        })
    );

    let words = jr::scan("3 $ 2 2 $ 0 1 2 3").unwrap();
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
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
        Verb(String::from("*:"), VerbImpl::StarCo),
        Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Noun(BoolArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        }),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 4),
        }),
    ];
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![4, 16]).unwrap(),
        })
    );
}

#[test]
fn test_power_conjunction_noun_arg() {
    //let words = jr::scan("(*:^:2) 4").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(String::from("*:"), VerbImpl::StarCo),
        Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 2),
        }),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[]), 4),
        }),
    ];
    // TODO Should the result be an atom 256 here? rather than an array of shape 1?
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_elem(IxDyn(&[1]), 256)
        })
    );

    let words = vec![
        Verb(String::from("*:"), VerbImpl::StarCo),
        Conjunction(String::from("^:"), ModifierImpl::HatCo),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
    ];
    assert_eq!(
        jr::eval(words).unwrap(),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![16, 81, 256, 6561]).unwrap(),
        }),
    );
}

#[test]
fn test_collect_int_nouns() {
    let a = vec![
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        }),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
    ];
    //let result = collect_int_nouns(a).unwrap();
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![0, 1, 2, 3]).unwrap(),
        }),
    );
}

#[test]
fn test_collect_extint_nouns() {
    let a = vec![
        Noun(ExtIntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![0, 1]).unwrap(),
        }),
        Noun(ExtIntArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec![2, 3]).unwrap(),
        }),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(ExtIntArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec![0, 1, 2, 3]).unwrap(),
        }),
    );
}

#[test]
fn test_collect_char_nouns() {
    let a = vec![
        Noun(CharArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec!['a', 'b']).unwrap(),
        }),
        Noun(CharArray {
            a: Array::from_shape_vec(IxDyn(&[2]), vec!['c', 'd']).unwrap(),
        }),
    ];
    let result = collect_nouns(a).unwrap();
    println!("result: {:?}", result);
    assert_eq!(
        result,
        Noun(CharArray {
            a: Array::from_shape_vec(IxDyn(&[2, 2]), vec!['a', 'b', 'c', 'd']).unwrap(),
        }),
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
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[5]), vec![1, 2, 3, 4, 5]).unwrap(),
        }),
    ];
    assert_eq!(jr::eval(words).unwrap(), int_array(vec![3]).unwrap());
}

#[test]
fn test_fork_noun() {
    //let words = jr::scan("(15 % #) 1 2 3 4 5").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(
            String::from("+/%#"),
            VerbImpl::Fork {
                f: Box::new(int_array(vec![15]).unwrap()),
                g: Box::new(Verb(String::from("%"), VerbImpl::Percent)),
                h: Box::new(Verb(String::from("#"), VerbImpl::Number)),
            },
        ),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[5]), vec![1, 2, 3, 4, 5]).unwrap(),
        }),
    ];
    assert_eq!(jr::eval(words).unwrap(), int_array(vec![3]).unwrap());
}

#[test]
fn test_hook() {
    //let words = jr::scan("(i. #) 3 1 4 1 5 9").unwrap(); //TODO use this when parens are implemented
    let words = vec![
        Verb(
            String::from("i.#"),
            VerbImpl::Hook {
                r: Box::new(Verb(String::from("i."), VerbImpl::IDot)),
                l: Box::new(Verb(String::from("#"), VerbImpl::Number)),
            },
        ),
        Noun(IntArray {
            a: Array::from_shape_vec(IxDyn(&[5]), vec![1, 2, 3, 4, 5]).unwrap(),
        }),
    ];
    assert_eq!(jr::eval(words).unwrap(), int_array(vec![3]).unwrap());
}
