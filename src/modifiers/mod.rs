//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JError, Word};

pub use adverb::*;
pub use conj::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierImpl {
    Adverb(SimpleAdverb),
    Conjunction(SimpleConjunction),
    Cor,
    DerivedAdverb { l: Box<Word>, r: Box<Word> },
}

impl ModifierImpl {
    pub fn form(&self, ctx: &mut Ctx, u: &Word, v: &Word) -> Result<(bool, Word)> {
        Ok(match self {
            ModifierImpl::Cor => c_cor(ctx, u, v)?,
            ModifierImpl::Conjunction(c) => (false, (c.f)(ctx, u, v)?),
            _ => return Err(JError::SyntaxError).context("non-conjunction in conjunction context"),
        })
    }

    pub fn form_adverb(&self, ctx: &mut Ctx, u: &Word) -> Result<Option<Word>> {
        Ok(match self {
            ModifierImpl::Adverb(c) => Some((c.f)(ctx, u)?),
            ModifierImpl::DerivedAdverb { .. } => None,
            _ => {
                return Err(JError::SyntaxError)
                    .with_context(|| anyhow!("non-adverb in adverb context: {self:?}"))
            }
        })
    }
}
