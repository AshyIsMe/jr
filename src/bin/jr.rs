use jr::{JError, Word};
use log::debug;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    env_logger::init();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("jr {}", env!("CARGO_PKG_VERSION"));

    loop {
        // repl
        stdout.write_all(b"   ")?; //prompt
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        match buffer.trim() {
            "exit" => break,
            _sentence => {
                match scan_eval(&buffer) {
                    Ok(output) => println!("{:?}", output),
                    Err(e) => println!("error: {}", e),
                }
                buffer.truncate(0);
            }
        }
    }

    Ok(())
}

fn scan_eval(sentence: &str) -> Result<Word, JError> {
    let tokens = jr::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    jr::eval(tokens)
}
