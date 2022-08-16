use std::io::{self, Write};

#[derive(Debug)]
enum Token {
    LP,
    RP,
    Verb(String),
}

fn scan(sentence: &str) -> Option<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut vs: usize = usize::MAX; //verbstart
    let mut ve: usize = usize::MAX; //verbend
    let mut wordend: bool = false;
    let mut newToken: Option<Token> = None;

    for (i, c) in sentence.chars().enumerate() {
        wordend = false;
        newToken = None;
        match c {
            '(' => {
                wordend = true;
                newToken = Some(Token::LP)
            }
            ')' => {
                wordend = true;
                newToken = Some(Token::RP)
            }
            ' ' => wordend = true,
            '\n' => wordend = true,
            _ => match vs {
                usize::MAX => {
                    vs = i; //new word started
                    ve = i
                }
                _ => ve = i, //word continued
            },
        }
        if wordend && (vs < usize::MAX) {
            tokens.push(Token::Verb(String::from(&sentence[vs..=ve])));
            vs = usize::MAX;
            ve = usize::MAX
        }
        match newToken {
            Some(t) => tokens.push(t),
            None => ()
        }
    }
    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
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
                //println!("que? '{}'", buffer.trim());
                println!("tokens: {:?}", tokens);
                buffer = String::from("");
            }
        }
    }

    Ok(())
}
