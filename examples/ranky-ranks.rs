use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};
use log::debug;

use jr::cells::generate_cells;
use jr::{JArray, Rank, Word};

fn main() -> Result<()> {
    env_logger::init();
    assert_eq!("69".to_string(), run_j("69")?);

    for (x, op, y, (xr, yr)) in [
        ("1 2 3 4", "+", "10 20 30 40", (0, 0)),
        ("10", "+", "1 2 3 4", (0, 0)),
        ("1 2", "+", "2 3 $ 10 20 30 70 80 90", (0, 0)),
        ("2 3", "$", "10 20 30 40 50 60", (1, u32::MAX)),
        ("(3 1 $ 2 3 4)", "$", "0 1 2 3", (1, u32::MAX)),
        ("3", "$", "(2 2 $ 5 6 7 8)", (1, u32::MAX)),
    ] {
        let expr = format!("{x} {op} {y}");
        println!("e: {expr}    NB. {op} is {xr} {yr}");
        println!("j: {}", run_j(&expr)?.replace('\n', "\n   "));
        let res = scan_eval(&expr);
        println!("r: {}", format!("{:?}", res).replace('\n', "\n   "));

        let (xc, yc, common, spare) =
            generate_cells(arr(x)?, arr(y)?, (Rank::new(xr)?, Rank::new(yr)?))?;
        println!("g: {xc:?} {yc:?} {common:?} {spare:?}");
        println!();
    }
    Ok(())
}

fn run_j(expr: &str) -> Result<String> {
    let mut p = Command::new("jconsole.sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("executing jconsole.sh from path")?;
    p.stdin
        .as_mut()
        .expect("requested")
        .write_all(expr.as_bytes())
        .context("writing to j")?;
    let out = p.wait_with_output().context("waiting for j to run")?;
    let s = String::from_utf8(out.stdout).context("reading from j")?;
    Ok(s.trim().to_string())
}

fn arr(sentence: &str) -> Result<JArray> {
    let word = scan_eval(sentence)?;
    Ok(match word {
        Word::Noun(arr) => arr,
        _ => bail!("unexpected non-noun: {word:?}"),
    })
}

fn scan_eval(sentence: &str) -> Result<Word> {
    let tokens = jr::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    jr::eval(tokens, &mut HashMap::new()).with_context(|| anyhow!("evaluating {:?}", sentence))
}
