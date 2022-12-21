//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, bail, Context, Result};
use std::ops::Deref;

use crate::{Ctx, JArray, Word};

pub use adverb::*;
pub use conj::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierImpl {
    Adverb(SimpleAdverb),
    Conjunction(SimpleConjunction),
    DerivedAdverb { l: Box<Word>, r: Box<Word> },
}

impl ModifierImpl {
    pub fn exec(
        &self,
        ctx: &mut Ctx,
        x: Option<&Word>,
        u: &Word,
        v: &Word,
        y: &Word,
    ) -> Result<Word> {
        match self {
            ModifierImpl::Adverb(a) => {
                (a.f)(ctx, x, u, y).with_context(|| anyhow!("adverb: {:?}", a.name))
            }
            ModifierImpl::Conjunction(c) => {
                (c.f)(ctx, x, u, v, y).with_context(|| anyhow!("conjunction: {:?}", c.name))
            }
            ModifierImpl::DerivedAdverb { l, r } => match (l.deref(), r.deref()) {
                (Word::Conjunction(cn, c), r) => c
                    .exec(ctx, x, u, r, y)
                    .with_context(|| anyhow!("derived adverb conjunction {cn:?}")),
                _ => bail!(
                    "TODO: DerivedAdverb l: {l:?} r: {r:?} x: {x:?} u: {u:?} v: {v:?} y: {y:?}"
                ),
            },
        }
    }

    pub fn farcical(&self, m: &JArray, n: &JArray) -> Result<bool> {
        match self {
            ModifierImpl::Conjunction(c) => (c.farcical)(m, n),
            _ => Ok(false),
        }
    }
}
