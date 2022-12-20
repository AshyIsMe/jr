//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, bail, Context, Result};

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
            ModifierImpl::DerivedAdverb { l, r } => bail!("TODO: DerivedAdverb l: {l:?} r: {r:?}"),
        }
    }

    pub fn farcical(&self, m: &JArray, n: &JArray) -> Result<bool> {
        match self {
            ModifierImpl::Conjunction(c) => (c.farcical)(m, n),
            _ => Ok(false),
        }
    }
}
