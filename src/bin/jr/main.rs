use std::collections::VecDeque;

use anyhow::{Context, Result};
use cfg_if::cfg_if;
use jr::{feed, Ctx, EvalOutput, JError};
use log::warn;

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

#[derive(Eq, PartialEq)]
enum EvalState {
    Regular,
    Done,
    MoreInput,
}

fn eval(buffer: &str, ctx: &mut Ctx) -> Result<EvalState> {
    let buffer = buffer.trim();
    if "exit" == buffer || buffer.is_empty() {
        return Ok(EvalState::Done);
    }

    match feed(buffer, ctx) {
        //Ok(output) => println!("{:?}", output),
        Ok(EvalOutput::Regular(output)) => println!("{}", output),
        Ok(EvalOutput::Return(_)) => {
            return Err(JError::SyntaxError).context("return in interactive context")
        }
        Ok(EvalOutput::Suspension) | Ok(EvalOutput::InDefinition) => {
            return Ok(EvalState::MoreInput)
        }
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

    Ok(EvalState::Regular)
}
