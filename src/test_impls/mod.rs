mod jsoft_binary;
mod jsoft_runs;

use std::collections::HashMap;
use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context, Result};
use log::debug;

use crate::Word;

pub use jsoft_runs::{Run, RunList};

pub fn run_j(expr: impl AsRef<str>) -> Result<String> {
    let expr = expr.as_ref();
    run_j_inner(expr).with_context(|| anyhow!("running j on: {expr:?}"))
}

fn run_j_inner(expr: &str) -> Result<String> {
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

pub fn scan_eval(sentence: &str) -> Result<Word> {
    let tokens = crate::scan(sentence)?;
    debug!("tokens: {:?}", tokens);
    crate::eval(tokens, &mut HashMap::new()).with_context(|| anyhow!("evaluating {:?}", sentence))
}
