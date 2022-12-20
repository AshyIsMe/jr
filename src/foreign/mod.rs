mod files;
mod scripts;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JError, Word};

use files::*;
use scripts::*;

/// https://www.jsoftware.com/help/dictionary/xmain.htm
pub fn foreign(ctx: &mut Ctx, l: usize, r: usize, _x: Option<&Word>, y: &Word) -> Result<Word> {
    match (l, r) {
        (0, k) => f_load_script(ctx, k, y),
        (1, 1) => f_read_file(y).context("reading file"),
        _ => Err(JError::NonceError).with_context(|| anyhow!("unsupported foreign: {l}!:{r}")),
    }
}
