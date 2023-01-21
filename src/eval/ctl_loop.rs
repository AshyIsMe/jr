use crate::eval::controls::must_be_noun;
use crate::eval::ctl_if::split_once;
use crate::eval::{eval_lines, BlockEvalResult};
use crate::{eval, Ctx, HasEmpty, JArray, JError, Word};
use anyhow::{Context, Result};

pub fn control_for(ctx: &mut Ctx, style: Option<&str>, def: &[Word]) -> Result<BlockEvalResult> {
    if style.is_some() {
        return Err(JError::NonceError).context("for_ijk. not supported");
    }
    let (cond, block) = split_once(def, |w| matches!(w, Word::Do))
        .ok_or(JError::SyntaxError)
        .context("no do. in for.")?;

    let arr = eval(cond.to_vec(), ctx).and_then(must_be_noun)?;

    let mut last = Word::Noun(JArray::empty());
    for _item in arr.outer_iter() {
        match eval_lines(block, ctx)? {
            BlockEvalResult::Regular(n) => {
                last = n;
            }
            BlockEvalResult::Return(v) => return Ok(BlockEvalResult::Return(v)),
        }
    }

    Ok(BlockEvalResult::Return(last))
}
