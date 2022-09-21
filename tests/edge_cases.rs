use jr::verbs::reshape;
use jr::JArray::*;
use jr::{JError, ModifierImpl, VerbImpl, Word};
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
fn test_scan_string() -> Result<(), JError> {
    let words = jr::scan("'abc'").unwrap();
    assert_eq!(words, [jr::char_array("abc")?]);
    Ok(())
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
            Word::Verb(String::from("+"), VerbImpl::Plus),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn only_whitespace() {
    jr::scan("\r").unwrap();
}

#[test]
fn test_scan_string_verb_string() -> Result<(), JError> {
    let words = jr::scan("'abc','def'").unwrap();
    assert_eq!(
        words,
        [
            jr::char_array("abc")?,
            Word::Verb(String::from(","), VerbImpl::NotImplemented),
            jr::char_array("def")?,
        ]
    );
    Ok(())
}

#[test]
fn test_scan_name_verb_name_not_spaced() {
    let words = jr::scan("foo+bar\n").unwrap();
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::Verb(String::from("+"), VerbImpl::Plus),
            Word::Name(String::from("bar")),
        ]
    );
}

#[test]
fn test_scan_primitives() -> Result<(), JError> {
    let words = jr::scan("a. I. 'A' \n").unwrap();
    assert_eq!(
        words,
        [
            jr::char_array("a.")?,
            Word::Verb(String::from("I."), VerbImpl::NotImplemented),
            jr::char_array("A")?,
        ]
    );
    Ok(())
}

#[test]
fn test_scan_primitives_not_spaced() -> Result<(), JError> {
    let words = jr::scan("a.I.'A' \n")?;
    assert_eq!(
        words,
        [
            jr::char_array("a.")?,
            Word::Verb(String::from("I."), VerbImpl::NotImplemented),
            jr::char_array("A")?,
        ]
    );
    Ok(())
}
