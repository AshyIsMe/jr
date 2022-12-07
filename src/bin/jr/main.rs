use std::collections::VecDeque;

use anyhow::{anyhow, Context, Result};
use cfg_if::cfg_if;
use jr::{Ctx, JError, Word};
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

    match scan_eval(buffer, ctx) {
        //Ok(output) => println!("{:?}", output),
        Ok(output) => println!("{}", output),
        Err(e) if matches!(JError::extract(&e), Some(JError::StackSuspension)) => {
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

fn scan_eval(sentence: &str, ctx: &mut Ctx) -> Result<Word> {
    if ctx.input_wanted() {
        if sentence != ")" {
            ctx.input_push(sentence)?;
            return Err(JError::StackSuspension).context("scan_eval");
        }
        ctx.input_done()?;
        return jr::eval(Vec::new(), ctx);
    }
    let tokens = jr::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    jr::eval(tokens, ctx).with_context(|| anyhow!("evaluating {:?}", sentence))
}
