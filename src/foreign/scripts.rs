//! https://www.jsoftware.com/help/dictionary/dx000.htm

use std::fs;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use log::info;

use super::files::arg_to_fs_path;
use crate::{feed, Ctx, EvalOutput, HasEmpty, JArray, JError, Word};

pub fn f_load_script(ctx: &mut Ctx, k: usize, y: &Word) -> Result<Word> {
    let [src, err, display]: [char; 3] = format!("{k:03}")
        .chars()
        .collect_vec()
        .try_into()
        .expect("format");

    match display {
        '0' => (),
        '1' => return Err(JError::NonceError).context("display"),
        '2' => return Err(JError::NonceError).context("tautologies"),
        '3' => return Err(JError::NonceError).context("tautologies with result"),
        _ => return Err(JError::NonceError).context("unrecognised 'display' option"),
    };

    let break_on_error = match err {
        '0' => true,
        '1' => false,
        _ => return Err(JError::NonceError).context("unrecognised 'err' option"),
    };

    match src {
        '0' => (),
        '1' => return Err(JError::NonceError).context("from noun"),
        _ => return Err(JError::NonceError).context("unrecognised 'err' option"),
    }

    let path = arg_to_fs_path(y)?;

    let mut last = EvalOutput::Regular(Word::Nothing);
    for line in fs::read_to_string(&path)
        .with_context(|| anyhow!("reading {path:?}"))?
        .split('\n')
    {
        match feed(line, ctx) {
            Ok(word) => last = word,
            Err(e) if break_on_error => return Err(e),
            Err(e) => {
                info!("ignoring error during script load, as requested: {e:?}")
            }
        }
    }
    match last {
        EvalOutput::Regular(_) => Ok(Word::Noun(JArray::empty())),
        other => Err(anyhow!("file unexpectedly finished inside a {other:?}")),
    }
}
