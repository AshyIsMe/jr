use anyhow::{Context, Result};

use crate::number::Num;
use crate::{Elem, JArray, JError, Word};

/// monad, rank 0, num ->  num
pub fn m0nn(y: &JArray, f: impl FnOnce(Num) -> Num) -> Result<Word> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'y'")?;

    Ok(Word::Noun(f(y).into()))
}

/// monad, rank 0, num -> result num
pub fn m0nrn(y: &JArray, f: impl FnOnce(Num) -> Result<Num>) -> Result<Word> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'y'")?;

    Ok(Word::Noun(f(y)?.into()))
}

/// monad, rank 0, num -> jarray
pub fn m0nj(y: &JArray, f: impl FnOnce(Num) -> JArray) -> Result<Word> {
    let y = y
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("expecting a single number for 'y'")?;

    Ok(Word::Noun(f(y)))
}

/// rank: (0, 0), input: any Num, output: Result<Num>
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

/// rank: (0, 0), input: any Element, output: Boolean
pub fn rank0eb(x: &JArray, y: &JArray, f: impl FnOnce(Elem, Elem) -> bool) -> Result<Word> {
    let x = x
        .single_elem()
        .ok_or(JError::DomainError)
        .context("expecting a single element for 'x'")?;

    let y = y
        .single_elem()
        .ok_or(JError::DomainError)
        .context("expecting a single element for 'y'")?;

    let v = f(x, y);
    Ok(Word::Noun(Num::bool(v).into()))
}
