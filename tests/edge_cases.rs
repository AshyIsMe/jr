use jr::verbs::reshape;
use jr::JArray::*;
use jr::{VerbImpl, Word};
use ndarray::prelude::*;

#[test]
#[ignore]
fn test_scan_nunez() {
    let _ = jr::scan("й");
}

#[test]
#[ignore]
fn test_scan_prime_nunez() {
    let _ = jr::scan("'йй");
}

#[test]
fn invalid_prime() {
    // TODO: error matcher / diagnostics
    assert!(jr::scan("'").is_err());
}

#[test]
fn test_scan_num() {
    let words = jr::scan("1 2 _3\n").unwrap();
    assert_eq!(
        words,
        [Word::Noun(IntArray {
            a: ArrayD::from_shape_vec(IxDyn(&[3]), vec![1, 2, -3]).unwrap()
        })]
    );
}

#[test]
fn test_scan_string() {
    let words = jr::scan("'abc'").unwrap();
    assert_eq!(words, [jr::char_array("abc")]);
}

#[test]
fn test_scan_name() {
    let words = jr::scan("abc\n").unwrap();
    assert_eq!(words, [Word::Name(String::from("abc"))]);
}

#[test]
fn test_scan_name_verb_name() {
    let words = jr::scan("foo + bar\n").unwrap();
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+"), Box::new(VerbImpl::Plus)),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn only_whitespace() {
    jr::scan("\r").unwrap();
}

#[test]
fn test_scan_string_verb_string() {
    let words = jr::scan("'abc','def'").unwrap();
    assert_eq!(
        words,
        [
            jr::char_array("abc"),
            Word::Verb(String::from(","), Box::new(VerbImpl::NotImplemented)),
            jr::char_array("def"),
        ]
    );
}

#[test]
fn test_scan_name_verb_name_not_spaced() {
    let words = jr::scan("foo+bar\n").unwrap();
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+"), Box::new(VerbImpl::Plus)),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn test_scan_primitives() {
    let words = jr::scan("a. I. 'A' \n").unwrap();
    assert_eq!(
        words,
        [
            jr::char_array("a."),
            Word::Verb(String::from("I."), Box::new(VerbImpl::NotImplemented)),
            jr::char_array("A"),
        ]
    );
}

#[test]
fn test_scan_primitives_not_spaced() {
    let words = jr::scan("a.I.'A' \n").unwrap();
    assert_eq!(
        words,
        [
            jr::char_array("a."),
            Word::Verb(String::from("I."), Box::new(VerbImpl::NotImplemented)),
            jr::char_array("A"),
        ]
    );
}

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
        Word::Verb(String::from("+"), Box::new(VerbImpl::Plus)),
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
    let r = reshape(vec![4], y).unwrap();
    assert_eq!(r, Array::from_elem(IxDyn(&[4]), 1));
}

#[test]
fn test_TEMP_range() {
    assert_eq!((0..5), std::ops::Range { start: 0, end: 5 });
    assert_eq!(
        (0..5).collect::<Vec<i64>>(),
        std::ops::Range { start: 0, end: 5 }.collect::<Vec<i64>>()
    );
}
