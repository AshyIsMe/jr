//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JArray, JError, Word};

use crate::verbs::{PartialImpl, VerbImpl};
pub use adverb::*;
pub use conj::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ModifierImpl {
    Adverb(SimpleAdverb),
    OwnedAdverb(OwnedAdverb),

    Conjunction(SimpleConjunction),
    WordyConjunction(WordyConjunction),
    OwnedConjunction(OwnedConjunction),
    Cor,
    // this is a partially applied conjunction; (c v) or (c n), which needs a u/m to become a verb
    DerivedAdverb {
        c: Box<ModifierImpl>,
        vn: Box<Word>,
    },
    // (a a), which needs a u/m to become a verb
    MmHook {
        l: Box<ModifierImpl>,
        r: Box<ModifierImpl>,
    },
}

impl ModifierImpl {
    pub fn form_conjunction(&self, ctx: &mut Ctx, u: &Word, v: &Word) -> Result<(bool, Word)> {
        Ok(match self {
            ModifierImpl::Cor => c_cor(ctx, u, v)
                .context("cor (:)")
                .with_context(|| anyhow!("u: {u:?}"))
                .with_context(|| anyhow!("v: {v:?}"))?,
            ModifierImpl::WordyConjunction(c) => (
                false,
                (c.f)(ctx, u, v)
                    .context(c.name)
                    .with_context(|| anyhow!("u: {u:?}"))
                    .with_context(|| anyhow!("v: {v:?}"))?,
            ),
            ModifierImpl::OwnedConjunction(c) => (false, (c.f)(ctx, Some(u), v)?),
            ModifierImpl::Conjunction(c) => {
                let u = u
                    .when_verb_noun()
                    .ok_or(JError::DomainError)
                    .context("conjunction's input must be a verb-like or a noun")
                    .with_context(|| anyhow!("u: {u:?}"))?;
                let v = v
                    .when_verb_noun()
                    .ok_or(JError::DomainError)
                    .context("conjunction's input must be a verb-like or a noun")
                    .with_context(|| anyhow!("u: {u:?}"))?;

                let partial = (c.f)(ctx, &u, &v)
                    .context(c.name)
                    .with_context(|| anyhow!("u: {u:?}"))
                    .with_context(|| anyhow!("v: {v:?}"))?;
                (
                    false,
                    Word::Verb(VerbImpl::Partial(PartialImpl {
                        imp: partial,
                        def: Some(vec![u.into(), Word::Conjunction(self.clone()), v.into()]),
                    })),
                )
            }
            _ => return Err(JError::SyntaxError).context("non-conjunction in conjunction context"),
        })
    }

    pub fn form_adverb(&self, ctx: &mut Ctx, u: &Word) -> Result<(bool, Word)> {
        Ok((
            false,
            match self {
                ModifierImpl::Adverb(c) => Word::Verb(VerbImpl::Partial(PartialImpl {
                    imp: (c.f)(ctx, u).with_context(|| anyhow!("u: {u:?}"))?,
                    def: Some(vec![Word::Adverb(self.clone()), u.clone()]),
                })),
                ModifierImpl::OwnedAdverb(a) => {
                    (a.f)(ctx, u).with_context(|| anyhow!("u: {u:?}"))?
                }
                ModifierImpl::DerivedAdverb { c, vn } => {
                    return c
                        .form_conjunction(ctx, u, vn)
                        .with_context(|| anyhow!("u: {u:?}"))
                        .with_context(|| anyhow!("v/n: {vn:?}"))
                }
                ModifierImpl::MmHook { l, r } => {
                    let (l_farcical, lu) = l.form_adverb(ctx, u)?;
                    if l_farcical {
                        return Err(JError::NonceError)
                            .context("farcical conjunction execution in adverb hook");
                    }
                    return r.form_adverb(ctx, &lu);
                }
                _ => {
                    return Err(JError::SyntaxError)
                        .with_context(|| anyhow!("non-adverb in adverb context: {self:?}"))
                }
            },
        ))
    }

    pub fn boxed_ar(&self) -> Result<JArray> {
        use ModifierImpl::*;
        Ok(match self {
            Adverb(a) => JArray::from_string(a.name),
            Conjunction(c) => JArray::from_string(c.name),
            Cor => JArray::from_string(":"),
            _ => {
                return Err(JError::NonceError)
                    .with_context(|| anyhow!("can't ModifierImpl::boxed_ar {self:?}"))
            }
        })
    }
}
