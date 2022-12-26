//! https://www.jsoftware.com/help/dictionary/dx004.htm

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use log::debug;

use crate::foreign::files::arg_to_string;
use crate::{arr0d, Arrayable, Ctx, JArray, JError, Word};

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

// 4!:1
pub fn f_name_namelist(ctx: &Ctx, x: Option<&Word>, y: &Word) -> Result<Word> {
    match y {
        Word::Noun(JArray::IntArray(_)) => match x {
            Some(_) => {
                todo!("f_name_namelist x startswith filter")
            }
            None => {
                // TODO: filter Noun/Verb/Adverb/Conjunction/Locale by y values 0 1 2 3 6
                // TODO: Only works for the anon names (inner function locals) currently:
                //    {{ 4!:1 [ 1 2 3 4 [ a =. 1 2 3 [ b =. 4 5 6}} ''
                // |[[a], [y], [b]]|
                debug!("ctx: {:?}", ctx);
                let names: Vec<JArray> = ctx
                    .eval()
                    .locales
                    .anon
                    .iter()
                    .flat_map(|n| n.0.keys())
                    .map(|s| JArray::CharArray(s.chars().collect_vec().into_array().unwrap()))
                    .collect();
                Word::noun(names)
            }
        },
        Word::Noun(JArray::BoolArray(_)) => todo!("BoolArray"),
        _ => {
            return Err(JError::DomainError).context("IntArray or BoolArray please");
        }
    }
}
