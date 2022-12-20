mod jsoft_binary;
mod jsoft_runs;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, bail, Context, Result};

use crate::{Ctx, EvalOutput, JArray, Word};

pub use jsoft_runs::{Lookup, Run, RunList};

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
    let mut ctx = Ctx::empty();
    // always overwritten?
    let mut last = EvalOutput::Regular(Word::StartOfLine);
    for line in sentence.trim().split('\n') {
        last = crate::feed(line, &mut ctx).with_context(|| anyhow!("evaluating {:?}", line))?;
    }
    last.when_word()
}

pub fn read_ijs_lines(lines: &str) -> Vec<String> {
    lines
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("NB. "))
        .map(ToString::to_string)
        .collect()
}

pub fn read_ijs_dir(dir: impl AsRef<Path>) -> Result<Vec<String>> {
    let dir = dir.as_ref();
    fs::read_dir(dir)
        .with_context(|| anyhow!("listing {dir:?}"))?
        .map(|entry| {
            let path = entry?.path();
            fs::read_to_string(&path).with_context(|| anyhow!("reading {path:?}"))
        })
        .collect()
}

pub fn test_against(expr: impl AsRef<str>, lookup: &impl Lookup) -> Result<()> {
    let expr = expr.as_ref();
    assert_produces(&expr, lookup.get_cached(&expr)?).with_context(|| anyhow!("testing {:?}", expr))
}

pub fn assert_produces(expr: &str, (them, rendered): &(JArray, String)) -> Result<()> {
    let us = scan_eval(expr).context("running expression in smoke test")?;
    let Word::Noun(arr) = us else { bail!("unexpected non-array from eval: {us:?}") };

    if &arr == them {
        return Ok(());
    }

    Err(anyhow!("incorrect data, we got:\n{arr:?}\n\nThey expect:\n{them:?}\n\njsoft would render this like this:\n{rendered}"))
}
