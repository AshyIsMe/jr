use anyhow::Result;
use jr::primitive_nouns;
use jr::JArray::*;
use jr::Word;
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
#[ignore]
fn test_scan_prime_prime_nunez() {
    let _ = jr::scan("'й'e");
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
fn test_scan_atoms() -> Result<()> {
    let words = jr::scan("1\n")?;
    assert_eq!(
        words,
        [Word::Noun(BoolArray(ArrayD::from_elem(IxDyn(&[]), 1)))]
    );
    let words = jr::scan("42\n")?;
    assert_eq!(
        words,
        [Word::Noun(IntArray(ArrayD::from_elem(IxDyn(&[]), 42)))]
    );
    let words = jr::scan("3.14\n")?;
    assert_eq!(
        words,
        [Word::Noun(FloatArray(ArrayD::from_elem(IxDyn(&[]), 3.14)))]
    );
    let words = jr::scan("'a'\n")?;
    assert_eq!(
        words,
        [Word::Noun(CharArray(ArrayD::from_elem(IxDyn(&[]), 'a')))]
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
            primitive_nouns("a.").unwrap(),
            Word::static_verb("I."),
            Word::Noun(CharArray(ArrayD::from_elem(IxDyn(&[]), 'A')))
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
            primitive_nouns("a.").unwrap(),
            Word::static_verb("I."),
            Word::Noun(CharArray(ArrayD::from_elem(IxDyn(&[]), 'A')))
        ]
    );
    Ok(())
}
