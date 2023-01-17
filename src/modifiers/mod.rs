//! Implementations for Adverbs and Conjunctions:
//! https://code.jsoftware.com/wiki/Vocabulary/Modifiers

mod adverb;
mod conj;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JArray, JError, Word};

use crate::verbs::{PartialDef, PartialImpl, VerbImpl};
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
                        def: Box::new(PartialDef::Conjunction(u, self.clone(), v)),
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
                ModifierImpl::Adverb(c) => {
                    let u = u
                        .when_verb_noun()
                        .context("adverbs only take verb-likes or nouns")
                        .with_context(|| anyhow!("u: {u:?}"))?;
                    Word::Verb(VerbImpl::Partial(PartialImpl {
                        imp: (c.f)(ctx, &u).with_context(|| anyhow!("u: {u:?}"))?,
                        def: Box::new(PartialDef::Adverb(self.clone(), u.clone())),
                    }))
                }
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

    pub fn name(&self) -> String {
        use ModifierImpl::*;
        match self {
            Adverb(a) => a.name.to_string(),
            Conjunction(c) => c.name.to_string(),
            WordyConjunction(w) => w.name.to_string(),
            OwnedConjunction(_c) => format!("unrepresentable conjunction"),
            Cor => ":".to_string(),
            OwnedAdverb(_a) => format!("unrepresentable adverb"),
            DerivedAdverb { .. } => format!("unrepresentable adverb"),
            MmHook { .. } => format!("unrepresentable mmhook"),
        }
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
