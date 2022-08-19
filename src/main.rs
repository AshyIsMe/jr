use std::io::{self, Write};

#[derive(Debug,PartialEq)]
enum Token {
    LP,
    RP,
    Verb(String),
    LitNumArray(String),
    LitString(String),
}

#[derive(Debug)]
struct ParseError {
    message: String,
}

fn scan(sentence: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut ws: usize = usize::MAX; //word start
    let mut we: usize = usize::MAX; //word end
    let mut word_end: bool;
    let mut new_token: Option<Token>;

    let mut skip: usize = 0;

    //TODO recursive descent instead of a dumb loop
    for (i, c) in sentence.chars().enumerate() {
        word_end = false;
        new_token = None;
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match c {
            '(' => {
                word_end = true;
                new_token = Some(Token::LP)
            }
            ')' => {
                word_end = true;
                new_token = Some(Token::RP)
            }
            ' ' | '\t' | '\n' => {
                word_end = true;
            }
            '0'..='9' => {
                let (l, t) = scan_litnumarray(&sentence[i..])?;
                tokens.push(t);
                skip = l;
                continue;
            }
            '\'' => {
                let (l, t) = scan_litstring(&sentence[i..])?;
                tokens.push(t);
                skip = l;
                continue;
            }
            _ => {
                match ws {
                    usize::MAX => {
                        ws = i; //new word started
                        we = i;
                    }
                    _ => we = i, //word continued
                }
            }
        }
        if word_end && (ws < usize::MAX) {
            tokens.push(Token::Verb(String::from(&sentence[ws..=we])));
            ws = usize::MAX;
            we = usize::MAX;
        }
        match new_token {
            Some(t) => tokens.push(t),
            None => (),
        }
    }
    Ok(tokens)
}

fn scan_litnumarray(sentence: &str) -> Result<(usize, Token), ParseError> {
    let mut l: usize = usize::MAX;
    for (i, c) in sentence.chars().enumerate() {
        l = i;
        match c {
            '0'..='9' | '.' | 'e' | 'j' | 'r' | ' ' | '\t' => {
                () //still valid keep iterating
            }
            _ => {
                break;
            }
        }
    }
    //Err(ParseError {message: String::from("Empty number literal")})
    Ok((l, Token::LitNumArray(String::from(&sentence[0..l]))))
}

fn scan_litstring(sentence: &str) -> Result<(usize, Token), ParseError> {
    let mut l: usize = usize::MAX;
    let mut leading_quote: bool = false;
    for (i, c) in sentence.chars().enumerate().skip(1) {
        l = i;
        match c {
            '\'' => match leading_quote {
                true =>
                // double quote in string, literal quote char
                {
                    leading_quote = false
                }
                false => leading_quote = true,
            },
            '\n' => {
                return Err(ParseError {
                    message: String::from("open quote"),
                })
            }
            _ => match leading_quote {
                true => {
                    //string closed previous char
                    l -= 1;
                    break;
                }
                false => {
                    () //still valid keep iterating
                }
            },
        }
    }
    Ok((
        l,
        Token::LitString(String::from(&sentence[1..l]).replace("''", "'")),
    ))
}

fn scan_word(sentence: &str) -> Result<(usize, Token), ParseError> {
    // user defined adverbs/verbs/nouns
    let mut l: usize = usize::MAX;
    for (i, c) in sentence.chars().enumerate() {
        l = i;
        if "()`.:; \t\n".contains(c) {
            break;
        }
    }
    //Err(ParseError {message: String::from("Empty number literal")})
    Ok((l, Token::Verb(String::from(&sentence[0..l]))))
}

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("jr {}", option_env!("CARGO_PKG_VERSION").unwrap());

    loop {
        // repl
        stdout.write(b"   ")?; //prompt
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        match buffer.trim() {
            "exit" => break,
            _sentence => {
                let tokens = scan(&buffer);
                println!("tokens: {:?}", tokens);
                buffer = String::from("");
            }
        }
    }

    Ok(())
}


#[test]
fn scan_num() {
    let tokens = scan("1 2 3\n").unwrap();
    println!("{:?}", tokens);
    assert_eq!(tokens, [Token::LitNumArray(String::from("1 2 3"))]);
}
