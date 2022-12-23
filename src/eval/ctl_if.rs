use anyhow::{Context, Result};
use log::debug;
use num_traits::Zero;

use crate::eval::eval_lines;
use crate::{eval, Ctx, Elem, JError, Num, Word};

pub fn control_if(ctx: &mut Ctx, def: &[Word]) -> Result<()> {
    debug!("control if.");
    if def.iter().any(|w| matches!(w, Word::ElseIf)) {
        return Err(JError::NonceError).context("no elsif.");
    }
    let (cond, follow) = split_once(&def, |w| matches!(w, Word::Do))
        .ok_or(JError::SyntaxError)
        .context("no do. in if.")?;
    let (tru, fal) = match split_once(follow, |w| matches!(w, Word::Else)) {
        Some(x) => x,
        None => (follow, [].as_slice()),
    };
    let cond = match eval(cond.to_vec(), ctx).context("if statement conditional")? {
        Word::Noun(arr) => arr
            .into_elems()
            .into_iter()
            .next()
            .map(|e| e != Elem::Num(Num::zero()))
            .unwrap_or_default(),
        _ => return Err(JError::NounResultWasRequired).context("evaluating if conditional"),
    };
    if cond {
        let _ = eval_lines(tru, ctx).context("true block")?;
    } else if !fal.is_empty() {
        let _ = eval_lines(fal, ctx).context("false block")?;
    }

    Ok(())
}

fn split_once(words: &[Word], f: impl Fn(&Word) -> bool) -> Option<(&[Word], &[Word])> {
    words
        .iter()
        .position(f)
        .map(|p| (&words[..p], &words[p + 1..]))
}
