//! https://www.jsoftware.com/help/dictionary/dx000.htm

use anyhow::{anyhow, Context, Result};
use std::fs;

use crate::{feed, Ctx, EvalOutput, HasEmpty, JArray, JError, Word};

pub fn f_load_script(ctx: &mut Ctx, k: usize, y: &Word) -> Result<Word> {
    if k != 0o000 {
        return Err(JError::NonceError).context("only support [file, error, silent] load mode");
    }
    let Word::Noun(JArray::BoxArray(y)) = y else { return Err(JError::NonceError).context("only support <'filepath' loading"); };
    if y.len() != 1 {
        return Err(JError::NonceError).context("only one path please");
    }

    let path = y.iter().next().expect("just checked");
    let JArray::CharArray(path) = path else { return Err(JError::NonceError).context("string required") };
    if path.shape().len() != 1 {
        return Err(JError::NonceError).context("single string required");
    }

    let path = path.iter().collect::<String>();
    let path = fs::canonicalize(&path).with_context(|| anyhow!("canonicalising {path}"))?;

    let mut last = EvalOutput::Regular(Word::Nothing);
    for line in fs::read_to_string(&path)
        .with_context(|| anyhow!("reading {path:?}"))?
        .split('\n')
    {
        last = feed(line, ctx)?;
    }
    match last {
        EvalOutput::Regular(_) => Ok(Word::Noun(JArray::empty())),
        other => Err(anyhow!("file unexpectedly finished inside a {other:?}")),
    }
}
