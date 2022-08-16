use std::io::{self, Write};

#[derive(Debug)]
enum Token {
    LP,
    RP,
    Verb(String),
    LiteralNumberArray(String),
    LiteralString(String),
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

    let mut ns: usize = usize::MAX; //number array start
    let mut ne: usize = usize::MAX; //number array end
    let mut num_end: bool;

    //TODO recursive descent instead of a dumb loop
    for (i, c) in sentence.chars().enumerate() {
        word_end = false;
        new_token = None;
        num_end = false;
        match c {
            '(' => {
                word_end = true;
                num_end = true;
                new_token = Some(Token::LP)
            }
            ')' => {
                word_end = true;
                num_end = true;
                new_token = Some(Token::RP)
            }
            '\n' => {
                word_end = true;
                num_end = true;
            }
            ' ' | '\t' => {
                word_end = true;
                //num_end = true
            }
            '0'..='9' => match ns {
                usize::MAX => {
                    ns = i; //new number array started
                    ne = i;
                }
                _ => ne = i //number continued
            },
            '.' => {
                match ns {
                    usize::MAX => {
                        //not in a number
                        match ws {
                            usize::MAX => {
                                ws = i; //new word started
                                we = i;
                            }
                            _ => we = i, //word continued
                        }
                    }
                    _ => ne = i, //number continued
                }
            }
            _ => {
                num_end = true;
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
        } else if num_end && (ns < usize::MAX) {
            tokens.push(Token::LiteralNumberArray(String::from(&sentence[ns..=ne])));
            ns = usize::MAX;
            ne = usize::MAX;
        }
        match new_token {
            Some(t) => tokens.push(t),
            None => (),
        }
    }
    Ok(tokens)
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
