use anyhow::{Context, Result};

use crate::{JArray, JError, Num, Word};

pub fn rank0(x: &JArray, y: &JArray, f: impl FnOnce(Num, Num) -> Result<Num>) -> Result<Word> {
    let x = x
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'x'")?;

    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'y'")?;

    Ok(Word::Noun(f(x, y)?.into()))
}
