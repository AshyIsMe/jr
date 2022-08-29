use jr::JArray::*;
use jr::{VerbImpl, Word};
use ndarray::prelude::*;

#[test]
fn test_scan_nunez() {
    let _ = jr::scan("й");
}

#[test]
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
    println!("{:?}", words);
    //assert_eq!(words, [Word::LitNumArray(String::from("1 2 _3"))]);
    assert_eq!(
        words,
        [Word::Noun(IntArray {
            v: ArrayD::from_shape_vec(IxDyn(&[3]), vec![1, 2, -3]).unwrap()
        })]
    );
}

#[test]
fn test_scan_string() {
    let words = jr::scan("'abc'").unwrap();
    println!("{:?}", words);
    assert_eq!(words, [jr::char_array("abc")]);
}

#[test]
fn test_scan_name() {
    let words = jr::scan("abc\n").unwrap();
    println!("{:?}", words);
    assert_eq!(words, [Word::Name(String::from("abc"))]);
}

#[test]
fn test_scan_name_verb_name() {
    let words = jr::scan("foo + bar\n").unwrap();
    println!("{:?}", words);
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+"), Some(VerbImpl::Plus)),
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
    println!("{:?}", words);
    assert_eq!(
        words,
        [
            jr::char_array("abc"),
            Word::Verb(String::from(","), Some(VerbImpl::NotImplemented)),
            jr::char_array("def"),
        ]
    );
}

#[test]
fn test_scan_name_verb_name_not_spaced() {
    let words = jr::scan("foo+bar\n").unwrap();
    println!("{:?}", words);
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+"), Some(VerbImpl::Plus)),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn test_scan_primitives() {
    let words = jr::scan("a. I. 'A' \n").unwrap();
    println!("{:?}", words);
    assert_eq!(
        words,
        [
            jr::char_array("a."),
            Word::Verb(String::from("I."), Some(VerbImpl::NotImplemented)),
            jr::char_array("A"),
        ]
    );
}

#[test]
fn test_scan_primitives_not_spaced() {
    let words = jr::scan("a.I.'A' \n").unwrap();
    println!("{:?}", words);
    assert_eq!(
        words,
        [
            jr::char_array("a."),
            Word::Verb(String::from("I."), Some(VerbImpl::NotImplemented)),
            jr::char_array("A"),
        ]
    );
}

#[test]
fn test_basic_addition() {
    let words = jr::scan("2 + 2").unwrap();
    println!("{:?}", words);
    let result = jr::eval(words).unwrap();
    assert_eq!(
        result,
        Word::Noun(IntArray {
            v: Array::from_elem(IxDyn(&[1]), 4)
        })
    );

    let words = jr::scan("1 2 3 + 4 5 6").unwrap();
    println!("{:?}", words);
    let result = jr::eval(words).unwrap();
    assert_eq!(
        result,
        Word::Noun(IntArray {
            v: Array::from_shape_vec(IxDyn(&[3]), vec![5, 7, 9]).unwrap()
        })
    );
}

#[test]
fn test_basic_times() {
    let words = jr::scan("2 * 2").unwrap();
    println!("{:?}", words);
    let result = jr::eval(words).unwrap();
    assert_eq!(
        result,
        Word::Noun(IntArray {
            v: Array::from_elem(IxDyn(&[1]), 4)
        })
    );

    let words = jr::scan("1 2 3 * 4 5 6").unwrap();
    println!("{:?}", words);
    let result = jr::eval(words).unwrap();
    assert_eq!(
        result,
        Word::Noun(IntArray {
            v: Array::from_shape_vec(IxDyn(&[3]), vec![4, 10, 18]).unwrap()
        })
    );
}
