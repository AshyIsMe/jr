use std::io::{self, Write};

enum Token {
    LP,
    RP,
    Verb(String),
}

fn scan(sentence: &str) -> Option<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    for c in sentence.chars() {
        match c {
            '(' => tokens.push(Token::LP),
            ')' => tokens.push(Token::RP),
            ' ' => (),
            _ => (),
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
                scan(&buffer);
                println!("que? '{}'", buffer.trim());
                buffer = String::from("");
            }
        }
    }

    Ok(())
}
