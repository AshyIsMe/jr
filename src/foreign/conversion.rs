use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{flatten, Arrayable, JArray, JError, Num, Word};

pub fn f_dump_hex(x: Option<&Word>, y: &Word) -> Result<Word> {
    if cfg!(not(target_pointer_width = "64")) {
        return Err(JError::NonceError).context("only support 64-bit (laziness)");
    }

    if cfg!(not(target_endian = "little")) {
        return Err(JError::NonceError).context("haha, very funny");
    }

    match x {
        Some(Word::Noun(x)) => match x.single_math_num() {
            Some(x) if x == Num::Int(3) || x == Num::Int(11) => (),
            _ => return Err(JError::NonceError).context("unsupported mode"),
        },
        None => (),
        _ => return Err(JError::DomainError).context("invalid mode"),
    }

    let y = match y {
        Word::Noun(arr) => arr,
        _ => return Err(JError::NounResultWasRequired).context("can only serialise data"),
    };

    let mut result = Vec::with_capacity(8);
    result.push(0xe3); // 64-bit, reversed

    match y {
        JArray::IntArray(arr) => {
            result.push(4);
            // note: not JArray.len()
            result.push(i64::try_from(arr.len())?);
            result.push(i64::try_from(arr.shape().len())?);
            for shape in arr.shape() {
                result.push(i64::try_from(*shape)?);
            }
            result.extend(arr.iter().copied());
        }

        _ => return Err(JError::NonceError).context("only int arrays (don't ask)"),
    }

    let result = result
        .into_iter()
        .map(|x| {
            format!("{:016x}", x.to_be())
                .chars()
                .collect_vec()
                .into_array()
                .expect("infalliable for vec")
                .into()
        })
        .collect_vec();

    flatten(&result.into_array()?).map(Word::Noun)
}

pub fn f_int_bytes(x: Option<&Word>, y: &Word) -> Result<Word> {
    let Some(Word::Noun(x)) = x else { return Err(JError::DomainError).context("invalid mode type"); };
    let Word::Noun(y) = y else { return Err(JError::DomainError).context("invalid data"); };
    match x.single_math_num() {
        Some(x) if x == Num::Int(2) => (),
        _ => return Err(JError::NonceError).context("unsupported mode"),
    }
    let Some(y) = y.single_math_num() else { return Err(JError::NonceError).context("only single numbers"); };
    let Some(y) = y.value_i64() else  { return Err(JError::NonceError).context("only integers"); };
    if y < 32 && y > 128 {
        return Err(JError::NonceError).context("ascii only");
    }

    Ok(Word::Noun(JArray::CharArray(
        // thanks, I hate it
        vec![y as u8 as char, '\0', '\0', '\0']
            .into_array()
            .expect("infallible"),
    )))
}
