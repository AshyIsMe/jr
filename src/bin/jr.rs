use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};

use anyhow::{anyhow, Context, Result};
use jr::Word;
use log::debug;

fn main() -> io::Result<()> {
    env_logger::init();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("jr {}", env!("CARGO_PKG_VERSION"));
    let mut names = HashMap::new();

    loop {
        // repl
        stdout.write_all(b"   ")?; //prompt
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        match buffer.trim() {
            "exit" => break,
            _sentence => {
                match scan_eval(&buffer, &mut names) {
                    //Ok(output) => println!("{:?}", output),
                    Ok(output) => println!("{}", output),
                    Err(e) => {
                        let mut stack: VecDeque<_> = e.chain().rev().collect();

                        println!(
                            "error: {}",
                            stack
                                .pop_front()
                                .expect("chain contains at least the error")
                        );

                        for error in stack {
                            println!("cause: {}", error);
                        }
                    }
                }
                buffer.truncate(0);
            }
        }
    }

    Ok(())
}

fn scan_eval(sentence: &str, names: &mut HashMap<String, Word>) -> Result<Word> {
    let tokens = jr::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    jr::eval(tokens, names).with_context(|| anyhow!("evaluating {:?}", sentence))
}
