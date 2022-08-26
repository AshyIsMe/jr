use jr::{scan, Word};
use ndarray::prelude::*;

// TODO support unicode properly
//#[test]
//fn test_scan_nunez() {
//let _ = scan("й");
//}

#[test]
fn invalid_prime() {
    // TODO: error matcher / diagnostics
    assert!(jr::scan("'").is_err());
}

#[test]
fn test_scan_num() {
    let Words = scan("1 2 _3\n").unwrap();
    println!("{:?}", Words);
    //assert_eq!(Words, [Word::LitNumArray(String::from("1 2 _3"))]);
    assert_eq!(
        Words,
        [Word::IntArray {
            v: ArrayD::from_shape_vec(IxDyn(&[3]), vec![1, 2, -3]).unwrap()
        }]
    );
}

#[test]
fn test_scan_string() {
    let Words = scan("'abc'").unwrap();
    println!("{:?}", Words);
    assert_eq!(Words, [Word::LitString(String::from("abc"))]);
}

#[test]
fn test_scan_name() {
    let Words = scan("abc\n").unwrap();
    println!("{:?}", Words);
    assert_eq!(Words, [Word::Name(String::from("abc"))]);
}

#[test]
fn test_scan_name_verb_name() {
    let Words = scan("foo + bar\n").unwrap();
    println!("{:?}", Words);
    assert_eq!(
        Words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+")),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn only_whitespace() {
    scan("\r").unwrap();
}

#[test]
fn test_scan_string_verb_string() {
    let Words = scan("'abc','def'").unwrap();
    println!("{:?}", Words);
    assert_eq!(
        Words,
        [
            Word::LitString(String::from("abc")),
            Word::Verb(String::from(",")),
            Word::LitString(String::from("def")),
        ]
    );
}

#[test]
fn test_scan_name_verb_name_not_spaced() {
    let Words = scan("foo+bar\n").unwrap();
    println!("{:?}", Words);
    assert_eq!(
        Words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+")),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn test_scan_primitives() {
    let Words = scan("a. I. 'A' \n").unwrap();
    println!("{:?}", Words);
    assert_eq!(
        Words,
        [
            Word::Noun(String::from("a.")),
            Word::Verb(String::from("I.")),
            Word::LitString(String::from("A")),
        ]
    );
}

#[test]
fn test_scan_primitives_not_spaced() {
    let Words = scan("a.I.'A' \n").unwrap();
    println!("{:?}", Words);
    assert_eq!(
        Words,
        [
            Word::Noun(String::from("a.")),
            Word::Verb(String::from("I.")),
            Word::LitString(String::from("A")),
        ]
    );
}
