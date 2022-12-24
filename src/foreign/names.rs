//! https://www.jsoftware.com/help/dictionary/dx004.htm

use anyhow::{anyhow, Context, Result};

use crate::foreign::files::arg_to_string;
use crate::{arr0d, Ctx, JArray, JError, Word};

// 4!:0
pub fn f_name_status(ctx: &Ctx, y: &Word) -> Result<Word> {
    let Word::Noun(JArray::BoxArray(y)) = y else { return Err(JError::DomainError).context("boxed name please"); };
    let name = arg_to_string(y)?;
    let result = match ctx.eval().locales.lookup(&name) {
        Ok(Some(Word::Noun(_))) => 0i64,
        Ok(Some(Word::Adverb(_, _))) => 1,
        Ok(Some(Word::Conjunction(_, _))) => 2,
        Ok(Some(Word::Verb(_, _))) => 3,
        Ok(Some(other)) => {
            return Err(JError::NonceError)
                .with_context(|| anyhow!("unknown word in name {name:?}: {other:?}"))
        }
        Ok(None) => -1,
        Err(_) => -2,
    };

    Ok(Word::Noun(JArray::IntArray(arr0d(result))))
}
