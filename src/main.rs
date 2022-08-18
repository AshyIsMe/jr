use std::io::{self, Write};

#[derive(Debug)]
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
            '\n' => {
                word_end = true;
            }
            ' ' | '\t' => {
                word_end = true;
            }
            '0'..='9' => match scanLitNumArray(&sentence[i..]) {
                Ok((l, t)) => {
                    tokens.push(t);
                    skip = l-1;
                    continue;
                }
                Err(e) => return Err(e)
            },
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

fn scanLitNumArray(sentence: &str) -> Result<(usize, Token), ParseError> {
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

fn main() -> io::Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("jr {}", option_env!("CARGO_PKG_VERSION").unwrap());

    loop {
        // repl
        stdout.write(b"   ")?; //prompt
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
