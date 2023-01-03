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
    FormingConjunction(FormingConjunction),
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
                // TODO: hot garbage, working around adverbs not being forming, I think?
                // TODO: expecting DerivedAdverbs to go with forming adverbs
                (Word::Conjunction(_cn, c@ ModifierImpl::FormingConjunction(_)), r) => {
                    let verb = c.form(ctx, u, r)?.expect("forming conjunctions always form");
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
            ModifierImpl::FormingConjunction(_) => bail!("shouldn't be calling these"),
        }
    }

    pub fn form(&self, ctx: &mut Ctx, u: &Word, v: &Word) -> Result<Option<Word>> {
        Ok(Some(match self {
            ModifierImpl::FormingConjunction(c) => (c.f)(ctx, u, v)?,
            _ => return Ok(None),
        }))
    }

    pub fn farcical(&self, m: &JArray, n: &JArray) -> Result<bool> {
        match self {
            ModifierImpl::Conjunction(c) => (c.farcical)(m, n),
            _ => Ok(false),
        }
    }
}
