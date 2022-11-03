use std::collections::{HashMap, VecDeque};

use anyhow::{anyhow, Context, Result};
use cfg_if::cfg_if;
use jr::Word;
use log::{debug, warn};

#[cfg(feature = "tui")]
mod tui;

fn main() -> Result<()> {
    env_logger::init();

    println!("jr {}", env!("CARGO_PKG_VERSION"));

    cfg_if! {
    if #[cfg(feature = "tui")] {
        tui::drive()?
    } else {
        plain_drive()?
    }
    }

    Ok(())
}

#[cfg(not(feature = "tui"))]
fn plain_drive() -> Result<()> {
    use std::io::{self, Write};

    let mut names = HashMap::new();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        stdout.write_all(b"   ")?; //prompt
        stdout.flush()?;
        stdin.read_line(&mut buffer)?;

        if eval(&buffer, &mut names)? {
            break;
        }
        buffer.truncate(0);
    }

    Ok(())
}

fn eval(buffer: &str, names: &mut HashMap<String, Word>) -> Result<bool> {
    let buffer = buffer.trim();
    if "exit" == buffer || buffer.is_empty() {
        return Ok(true);
    }

    match scan_eval(&buffer, names) {
        //Ok(output) => println!("{:?}", output),
        Ok(output) => println!("{}", output),
        Err(e) => {
            warn!("{:?}", e);
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

    Ok(false)
}

fn scan_eval(sentence: &str, names: &mut HashMap<String, Word>) -> Result<Word> {
    let tokens = jr::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    jr::eval(tokens, names).with_context(|| anyhow!("evaluating {:?}", sentence))
}
