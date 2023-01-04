//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, bail, Context, Result};
use std::ops::Deref;

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
            ModifierImpl::DerivedAdverb { l, r } => match (l.deref(), r.deref()) {
                // TODO: hot garbage, working around adverbs not being forming, I think?
                // TODO: expecting DerivedAdverbs to go with forming adverbs
                (Word::Conjunction(_cn, c@ ModifierImpl::Conjunction(_)), r) => {
                    let (farcical, verb) = c.form(ctx, u, r)?;
                    assert!(!farcical);
                    match verb {
                        Word::Verb(_, v) => v.exec(ctx, x, y).map(Word::Noun),
                        _ => bail!(
                            "TODO: forming DerivedAdverb\nl: {l:?}\nr: {r:?}\nx: {x:?}\nu: {u:?}\nv: {v:?}\ny: {y:?}"
                        ),
                    }
                }
                _ => bail!(
                    "TODO: DerivedAdverb\nl: {l:?}\nr: {r:?}\nx: {x:?}\nu: {u:?}\nv: {v:?}\ny: {y:?}"
                ),
            },
            ModifierImpl::Cor |
            ModifierImpl::Conjunction(_) => bail!("shouldn't be calling these"),
        }
    }

    pub fn form(&self, ctx: &mut Ctx, u: &Word, v: &Word) -> Result<(bool, Word)> {
        Ok(match self {
            ModifierImpl::Cor => c_cor(ctx, u, v)?,
            ModifierImpl::Conjunction(c) => (false, (c.f)(ctx, u, v)?),
            _ => return Err(JError::SyntaxError).context("non-conjunction in conjunction context"),
        })
    }
}
