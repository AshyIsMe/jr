use crate::arrays::ArcArrayD;
use crate::{arr0ad, Word};
use crate::{char_array, primitive_nouns, JArray};
use anyhow::Result;
use itertools::Itertools;
use ndarray::prelude::*;

use super::scan;

#[test]
fn test_scan_nunez() {
    let _ = scan("й");
}

#[test]
fn test_scan_prime_nunez() {
    let _ = scan("'йй");
}

#[test]
fn test_scan_prime_prime_nunez() {
    let _ = scan("'й'e");
}

#[test]
fn test_scan_nunez_neg() {
    let _ = scan("й-");
}

#[test]
fn test_scan_wot() {
    let s = String::from_utf8(vec![0xc2, 0x85, 0]).unwrap();
    println!("s: {:?}", s);
    println!("chars: {:?}", s.chars().enumerate().collect_vec());
    let _ = scan(&s);
}

#[test]
fn invalid_prime() {
    // TODO: error matcher / diagnostics
    assert!(scan("'").is_err());
}

#[test]
fn test_scan_num() -> Result<()> {
    let words = scan("1 2 _3")?;
    assert_eq!(
        words,
        [Word::Noun(JArray::IntArray(ArcArrayD::from_shape_vec(
            IxDyn(&[3]),
            vec![1, 2, -3]
        )?))]
    );
    Ok(())
}

#[test]
fn test_scan_atoms() -> Result<()> {
    let words = scan("1")?;
    assert_eq!(
        words,
        [Word::Noun(JArray::BoolArray(ArcArrayD::from_elem(
            IxDyn(&[]),
            1
        )))]
    );
    let words = scan("42")?;
    assert_eq!(
        words,
        [Word::Noun(JArray::IntArray(ArcArrayD::from_elem(
            IxDyn(&[]),
            42
        )))]
    );
    let words = scan("3.14")?;
    assert_eq!(
        words,
        [Word::Noun(JArray::FloatArray(ArcArrayD::from_elem(
            IxDyn(&[]),
            3.14
        )))]
    );
    let words = scan("'a'")?;
    assert_eq!(
        words,
        [Word::Noun(JArray::CharArray(ArcArrayD::from_elem(
            IxDyn(&[]),
            'a'
        )))]
    );

    Ok(())
}

#[test]
fn test_scan_string() -> Result<()> {
    let words = scan("'abc'")?;
    assert_eq!(words, [char_array("abc")]);
    Ok(())
}

#[test]
fn test_scan_name() -> Result<()> {
    let words = scan("abc")?;
    assert_eq!(words, [Word::Name(String::from("abc"))]);
    Ok(())
}

#[test]
fn test_scan_name_verb_name() -> Result<()> {
    let words = scan("foo + bar")?;
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::static_verb("+"),
            Word::Name(String::from("bar")),
        ]
    );
    Ok(())
}

#[test]
fn only_whitespace() -> Result<()> {
    scan("\r")?;
    Ok(())
}

#[test]
fn test_scan_string_verb_string() -> Result<()> {
    let words = scan("'abc','def'")?;
    assert_eq!(
        words,
        [char_array("abc"), Word::static_verb(","), char_array("def"),]
    );
    Ok(())
}

#[test]
fn test_scan_name_verb_name_not_spaced() -> Result<()> {
    let words = scan("foo+bar")?;
    assert_eq!(
        words,
        [
            Word::Name(String::from("foo")),
            Word::static_verb("+"),
            Word::Name(String::from("bar")),
        ]
    );
    Ok(())
}

#[test]
fn test_scan_primitives() -> Result<()> {
    let words = scan("a. I. 'A' ")?;
    assert_eq!(
        words,
        [
            primitive_nouns("a.").unwrap(),
            Word::static_verb("I."),
            Word::Noun(JArray::CharArray(ArcArrayD::from_elem(IxDyn(&[]), 'A')))
        ]
    );
    Ok(())
}

#[test]
fn test_scan_primitives_not_spaced() -> Result<()> {
    let words = scan("a.I.'A' ")?;
    assert_eq!(
        words,
        [
            primitive_nouns("a.").unwrap(),
            Word::static_verb("I."),
            Word::Noun(JArray::CharArray(ArcArrayD::from_elem(IxDyn(&[]), 'A')))
        ]
    );
    Ok(())
}

#[test]
fn test_scan_newlines() -> Result<()> {
    let words = scan("a =: {{\nb =: 5\na\n}}")?;
    use Word::*;
    assert_eq!(
        words,
        [
            Name("a".to_string()),
            IsGlobal,
            DirectDefUnknown,
            NewLine,
            Name("b".to_string()),
            IsGlobal,
            Word::Noun(JArray::IntArray(arr0ad(5i64))),
            NewLine,
            Name("a".to_string()),
            NewLine,
            DirectDefEnd
        ]
    );
    Ok(())
}
