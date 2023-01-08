use crate::eval::ctl_if::split_once;
use crate::eval::eval_lines;
use crate::{Ctx, JError, Word};
use anyhow::{Context, Result};

pub fn control_try(ctx: &mut Ctx, def: &[Word]) -> Result<()> {
    let (block, handle) = split_once(def, |w| {
        matches!(w, Word::Catch | Word::CatchT | Word::CatchD)
    })
    .ok_or(JError::SyntaxError)
    .context("no catch?. in try.")?;

    match eval_lines(block, ctx) {
        Ok(_) => Ok(()),
        Err(_) => eval_lines(handle, ctx).map(|_| ()),
    }
}
