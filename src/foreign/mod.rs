mod conversion;
mod files;
mod global_param;
mod scripts;

use anyhow::{anyhow, Context, Result};
use log::warn;

use crate::{Ctx, HasEmpty, JArray, JError, Word};

use conversion::*;
use files::*;
use global_param::*;
use scripts::*;

/// https://www.jsoftware.com/help/dictionary/xmain.htm
pub fn foreign(ctx: &mut Ctx, l: usize, r: usize, x: Option<&Word>, y: &Word) -> Result<Word> {
    let unsupported = |name: &'static str| -> Result<Word> {
        Err(JError::NonceError).with_context(|| anyhow!("unsupported {name} foreign: {l}!:{r}"))
    };

    let stub = |name: &'static str| -> Result<Word> {
        warn!("stubbed out foreign {l}!:{r}: {name}");
        Ok(Word::Noun(JArray::empty()))
    };

    match (l, r) {
        (0, k) => f_load_script(ctx, k, y),
        (1, 1) => f_read_file(y).context("reading file"),
        (1, _) => unsupported("file"),
        (2, _) => unsupported("host"),
        (3, 3) => f_dump_hex(x, y),
        (3, _) => unsupported("conversion"),
        (4, _) => unsupported("name"),
        (5, _) => unsupported("representation"),
        (6, _) => unsupported("time"),
        (7, _) => unsupported("space"),
        (8, _) => unsupported("format"),
        (9, 12) => f_os_type(),
        (9, _) => unsupported("global param"),
        (13, _) => unsupported("debug"),
        (15, _) => unsupported("dll"),
        (18, 4) => stub("set locales"),
        (18, _) => unsupported("locales"),
        (128, _) => unsupported("misc"),
        _ => unsupported("major"),
    }
}
