use anyhow::Result;
use jr::JArray::*;
use jr::Word;
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
fn test_scan_num() -> Result<()> {
    let words = jr::scan("1 2 _3\n")?;
    assert_eq!(
        words,
        [Word::Noun(IntArray(ArrayD::from_shape_vec(
            IxDyn(&[3]),
            vec![1, 2, -3]
        )?))]
    );
    Ok(())
}

#[test]
fn test_scan_string() -> Result<()> {
    let words = jr::scan("'abc'")?;
    assert_eq!(words, [jr::char_array("abc")?]);
    Ok(())
}

#[test]
fn test_scan_name() -> Result<()> {
    let words = jr::scan("abc\n")?;
    assert_eq!(words, [Word::Name(String::from("abc"))]);
    Ok(())
}

#[test]
fn test_scan_name_verb_name() -> Result<()> {
    let words = jr::scan("foo + bar\n")?;
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
    jr::scan("\r")?;
    Ok(())
}

#[test]
fn test_scan_string_verb_string() -> Result<()> {
    let words = jr::scan("'abc','def'")?;
    assert_eq!(
        words,
        [
            jr::char_array("abc")?,
            Word::static_verb(","),
            jr::char_array("def")?,
        ]
    );
    Ok(())
}

#[test]
fn test_scan_name_verb_name_not_spaced() -> Result<()> {
    let words = jr::scan("foo+bar\n")?;
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
    let words = jr::scan("a. I. 'A' \n")?;
    assert_eq!(
        words,
        [
            jr::char_array("a.")?,
            Word::static_verb("I."),
            jr::char_array("A")?,
        ]
    );
    Ok(())
}

#[test]
fn test_scan_primitives_not_spaced() -> Result<()> {
    let words = jr::scan("a.I.'A' \n")?;
    assert_eq!(
        words,
        [
            jr::char_array("a.")?,
            Word::static_verb("I."),
            jr::char_array("A")?,
        ]
    );
    Ok(())
}
