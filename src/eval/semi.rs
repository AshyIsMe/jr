use anyhow::{anyhow, Context, Result};

use crate::ctx::Eval;
use crate::verbs::VerbImpl;
use crate::{Ctx, JArray, JError, Word};

#[derive(Clone, Debug, PartialEq)]
pub enum VerbNoun {
    Verb(MaybeVerb),
    Noun(JArray),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaybeVerb {
    Verb(VerbImpl),
    Name(String),
}

impl Word {
    pub fn when_verb_noun(&self) -> Option<VerbNoun> {
        match self {
            Word::Noun(m) => Some(VerbNoun::Noun(m.clone())),
            Word::Verb(v) => Some(VerbNoun::Verb(MaybeVerb::Verb(v.clone()))),
            Word::Name(s) => Some(VerbNoun::Verb(MaybeVerb::Name(s.clone()))),
            _ => None,
        }
    }

    pub fn when_verb(&self) -> Option<MaybeVerb> {
        match self {
            Word::Verb(v) => Some(MaybeVerb::Verb(v.clone())),
            Word::Name(s) => Some(MaybeVerb::Name(s.clone())),
            _ => None,
        }
    }
}

impl Into<Word> for VerbNoun {
    fn into(self) -> Word {
        match self {
            VerbNoun::Verb(v) => v.into(),
            VerbNoun::Noun(s) => Word::Noun(s),
        }
    }
}

impl Into<Word> for MaybeVerb {
    fn into(self) -> Word {
        match self {
            MaybeVerb::Verb(v) => Word::Verb(v),
            MaybeVerb::Name(s) => Word::Name(s),
        }
    }
}

impl MaybeVerb {
    // clones() in all cases, but without it, you can't really eval the result
    // (due to ctx being borrowed immutable, and eval needing it mutable), so clone here for easier users?

    // similarly, into_verb() isn't very useful as normally this value is captured,
    // and you can't move out of the capture
    pub fn to_verb(&self, eval: &Eval) -> Result<VerbImpl> {
        Ok(match self {
            MaybeVerb::Verb(v) => v.clone(),
            MaybeVerb::Name(s) => match eval.locales.lookup(&s)? {
                Some(Word::Verb(v)) => v.clone(),
                Some(_) => {
                    return Err(JError::DomainError).with_context(|| {
                        anyhow!("name resolved to not a verb after binding as verb: {s:?}")
                    })
                }
                None => {
                    return Err(JError::DomainError)
                        .with_context(|| anyhow!("name wasn't defined: {s:?}"))
                }
            },
        })
    }

    // TODO: probably shouldn't use this, and should inline it everywhere; to_verb isn't free
    pub fn exec(&self, ctx: &mut Ctx, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
        let v = self.to_verb(ctx.eval())?;
        v.exec(ctx, x, y)
    }
}
