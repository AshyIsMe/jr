//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JError, Word};

use crate::verbs::{PartialImpl, VerbImpl};
pub use adverb::*;
pub use conj::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierImpl {
    Adverb(SimpleAdverb),
    Conjunction(SimpleConjunction),
    WordyConjunction(WordyConjunction),
    Cor,
    // this is a partially applied conjunction
    DerivedAdverb { c: Box<ModifierImpl>, u: Box<Word> },
}

impl ModifierImpl {
    pub fn form_conjunction(&self, ctx: &mut Ctx, u: &Word, v: &Word) -> Result<(bool, Word)> {
        Ok(match self {
            ModifierImpl::Cor => c_cor(ctx, u, v)?,
            ModifierImpl::WordyConjunction(c) => (false, (c.f)(ctx, u, v)?),
            ModifierImpl::Conjunction(c) => {
                let partial = (c.f)(ctx, u, v)?;
                (
                    false,
                    Word::Verb(VerbImpl::Partial(PartialImpl {
                        imp: partial,
                        def: Some(vec![Word::Conjunction(self.clone()), u.clone(), v.clone()]),
                    })),
                )
            }
            _ => return Err(JError::SyntaxError).context("non-conjunction in conjunction context"),
        })
    }

    pub fn form_adverb(&self, ctx: &mut Ctx, u: &Word) -> Result<Word> {
        Ok(match self {
            ModifierImpl::Adverb(c) => Word::Verb(VerbImpl::Partial(PartialImpl {
                imp: (c.f)(ctx, u)?,
                def: Some(vec![Word::Adverb(self.clone()), u.clone()]),
            })),
            ModifierImpl::DerivedAdverb { c, u: v } => {
                let (farcical, word) = c.form_conjunction(ctx, u, v)?;
                assert!(!farcical);
                word
            }
            _ => {
                return Err(JError::SyntaxError)
                    .with_context(|| anyhow!("non-adverb in adverb context: {self:?}"))
            }
        })
    }
}
