use anyhow::{Context, Result};
use itertools::Itertools;
use log::debug;
use num_traits::Zero;

use crate::eval::{eval_lines, BlockEvalResult};
use crate::{eval, Ctx, Elem, HasEmpty, JArray, JError, Num, Word};

pub fn control_if(ctx: &mut Ctx, def: &[Word]) -> Result<BlockEvalResult> {
    debug!("control if.");
    let mut blocks = Vec::new();
    let mut start = 0;
    for point in def
        .iter()
        .positions(|w| matches!(w, Word::Else | Word::ElseIf))
    {
        blocks.push(&def[start..point]);
        start = point;
    }
    blocks.push(&def[start..]);
    let first = blocks.remove(0);

    if let Some(result) = eval_cond_and_block(ctx, first)? {
        return Ok(result);
    };

    // doesn't really care if you write elseif. {} else. {} elseif. {}, it'll just never run the last one
    for block in blocks {
        match &block[0] {
            Word::ElseIf => {
                if let Some(result) = eval_cond_and_block(ctx, &block[1..])? {
                    return Ok(result);
                }
            }
            Word::Else => {
                return eval_lines(&block[1..], ctx);
            }
            other => unreachable!("covered by above matches: {other:?}"),
        }
    }

    // it's dem damn Nothings again
    Ok(BlockEvalResult::Regular(Word::Noun(JArray::empty())))
}

/// runs the condition *and the block*, returning if it did so
fn eval_cond_and_block(ctx: &mut Ctx, def: &[Word]) -> Result<Option<BlockEvalResult>> {
    let (cond, follow) = split_once(def, |w| matches!(w, Word::Do))
        .ok_or(JError::SyntaxError)
        .context("no do. in if.")?;

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
        Ok(Some(
            eval_lines(follow, ctx).context("conditional if-statement block")?,
        ))
    } else {
        Ok(None)
    }
}

pub fn split_once(words: &[Word], f: impl Fn(&Word) -> bool) -> Option<(&[Word], &[Word])> {
    words
        .iter()
        .position(f)
        .map(|p| (&words[..p], &words[p + 1..]))
}
