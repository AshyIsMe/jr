
use jr::{Token, scan};

#[test]
fn invalid_prime() {
    // TODO: error matcher / diagnostics
    assert!(jr::scan("'").is_err());
}

#[test]
fn test_scan_num() {
    let tokens = scan("1 2 _3\n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(tokens, [Token::LitNumArray(String::from("1 2 _3"))]);
}

#[test]
fn test_scan_string() {
    let tokens = scan("'abc'").unwrap();
    println!("{:?}", tokens);
    assert_eq!(tokens, [Token::LitString(String::from("abc"))]);
}

#[test]
fn test_scan_name() {
    let tokens = scan("abc\n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(tokens, [Token::Name(String::from("abc"))]);
}

#[test]
fn test_scan_name_verb_name() {
    let tokens = scan("foo + bar\n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(
        tokens,
        [
            Token::Name(String::from("foo")),
            Token::Primitive(String::from("+")),
            Token::Name(String::from("bar")),
        ]
    );
}

#[test]
fn only_whitespace() {
    scan("\r").unwrap();
}

#[test]
fn test_scan_string_verb_string() {
    let tokens = scan("'abc','def'").unwrap();
    println!("{:?}", tokens);
    assert_eq!(
        tokens,
        [
            Token::LitString(String::from("abc")),
            Token::Primitive(String::from(",")),
            Token::LitString(String::from("def")),
        ]
    );
}

#[test]
fn test_scan_name_verb_name_not_spaced() {
    let tokens = scan("foo+bar\n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(
        tokens,
        [
            Token::Name(String::from("foo")),
            Token::Primitive(String::from("+")),
            Token::Name(String::from("bar")),
        ]
    );
}

#[test]
fn test_scan_nunez() {
    let _ = scan("й");
}

#[test]
fn test_scan_primitives() {
    let tokens = scan("a. I. 'A' \n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(
        tokens,
        [
            Token::Primitive(String::from("a.")),
            Token::Primitive(String::from("I.")),
            Token::LitString(String::from("A")),
        ]
    );
}

#[test]
fn test_scan_primitives_not_spaced() {
    let tokens = scan("a.I.'A' \n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(
        tokens,
        [
            Token::Primitive(String::from("a.")),
            Token::Primitive(String::from("I.")),
            Token::LitString(String::from("A")),
        ]
    );
}
