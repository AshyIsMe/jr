//! https://www.jsoftware.com/help/dictionary/dx004.htm

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::ctx::Names;
use crate::foreign::files::{arg_to_string, arg_to_string_list};
use crate::{arr0d, Ctx, JArray, JError, Word};

// 4!:0
pub fn f_name_status(ctx: &Ctx, y: &JArray) -> Result<Word> {
    let JArray::BoxArray(y) = y else { return Err(JError::DomainError).context("boxed name please"); };
    let name = arg_to_string(y)?;
    let result = match ctx.eval().locales.lookup(&name) {
        Ok(Some(w)) => name_code(w)
            .ok_or(JError::NonceError)
            .with_context(|| anyhow!("unknown word in name {name:?}"))?,
        Ok(None) => -1,
        Err(_) => -2,
    };

    Ok(Word::Noun(JArray::IntArray(arr0d(result))))
}

// 4!:1
pub fn f_name_namelist(ctx: &Ctx, x: Option<&JArray>, y: &JArray) -> Result<Word> {
    let y = y.approx_i64_list().context("name list's y")?;
    let x = match x {
        Some(x) => Some(
            x.when_string()
                .ok_or(JError::DomainError)
                .context("single string for x")?,
        ),
        None => None,
    };

    if y.contains(&6) {
        return Err(JError::NonceError).context("unable to list locales");
    }
    // check for valid y values
    for i in y.iter() {
        if ![0, 1, 2, 3, 6].contains(i) {
            return Err(JError::DomainError).context("invalid y value");
        }
    }

    let locales = &ctx.eval().locales;
    let mut names: Vec<String> = locales
        .anon
        .last()
        .expect("there's always an anonymous locale")
        .0
        .iter()
        .chain(
            locales
                .inner
                .get(locales.current.as_str())
                .unwrap_or(&Names::default())
                .0
                .iter(),
        )
        .filter(|(k, _v)| x.as_ref().map(|x| k.starts_with(x)).unwrap_or(true))
        .filter(|(_k, v)| {
            name_code(v)
                .map(|code| y.contains(&code))
                .unwrap_or_default()
        })
        .map(|(k, _v)| k.to_string())
        .collect();
    names.sort();
    Ok(Word::Noun(JArray::from_list(
        names
            .into_iter()
            .map(|s| JArray::from_string(&s))
            .collect_vec(),
    )))
}

// 4!:55
pub fn f_name_erase(ctx: &mut Ctx, y: &JArray) -> Result<Word> {
    let JArray::BoxArray(y) = y else { return Err(JError::DomainError).context("boxed name please"); };
    let mut ret = Vec::new();
    for name in arg_to_string_list(y)? {
        ret.push(ctx.eval_mut().locales.erase(&name).is_ok() as u8);
    }
    Ok(Word::Noun(if ret.len() == 1 {
        JArray::BoolArray(arr0d(ret[0]))
    } else {
        JArray::from_list(ret)
    }))
}

fn name_code(w: &Word) -> Option<i64> {
    Some(match w {
        Word::Noun(_) => 0i64,
        Word::Adverb(_, _) => 1,
        Word::Conjunction(_, _) => 2,
        Word::Verb(_, _) => 3,
        _ => return None,
    })
}
