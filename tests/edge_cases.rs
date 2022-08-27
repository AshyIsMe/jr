use jr::Word;
use ndarray::prelude::*;

// TODO support unicode properly
//#[test]
//fn test_scan_nunez() {
//let _ =jr::scan("й");
//}

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
        [Word::IntArray {
            v: ArrayD::from_shape_vec(IxDyn(&[3]), vec![1, 2, -3]).unwrap()
        }]
    );
}

#[test]
fn test_scan_string() {
    let words = jr::scan("'abc'").unwrap();
    println!("{:?}", words);
    assert_eq!(
        words,
        [jr::chararray!["abc"]]
    );
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
            Word::Verb(String::from("+")),
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
            jr::chararray!["abc"],
            Word::Verb(String::from(",")),
            jr::chararray!["def"],
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
            Word::Verb(String::from("+")),
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
            Word::Noun(String::from("a.")),
            Word::Verb(String::from("I.")),
            jr::chararray!["A"],
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
            Word::Noun(String::from("a.")),
            Word::Verb(String::from("I.")),
            jr::chararray!["A"],
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
        Word::IntArray {
            v: Array::from_elem(IxDyn(&[1]), 4)
        }
    );

    let words = jr::scan("1 2 3 + 4 5 6").unwrap();
    println!("{:?}", words);
    let result = jr::eval(words).unwrap();
    assert_eq!(
        result,
        Word::IntArray {
            v: Array::from_shape_vec(IxDyn(&[3]), vec![5, 7, 9]).unwrap()
        }
    );
}
