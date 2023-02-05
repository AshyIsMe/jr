use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use ndarray::IxDyn;

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

impl VerbNoun {
    pub fn name(&self) -> String {
        use VerbNoun::*;
        match self {
            // TODO: copy-paste of Word::name()
            Verb(MaybeVerb::Verb(v)) => v.name(),
            Verb(MaybeVerb::Name(v)) => quote_string(v),
            Noun(arr) => quote_arr(arr),
        }
    }

    pub fn boxed_ar(&self) -> Result<JArray> {
        match self {
            VerbNoun::Verb(MaybeVerb::Verb(v)) => v.boxed_ar(),
            // TODO: don't LOOPBACK to Word
            VerbNoun::Verb(MaybeVerb::Name(s)) => Word::Name(s.clone()).boxed_ar(),
            // TODO: don't LOOPBACK to Word
            VerbNoun::Noun(n) => Word::Noun(n.clone()).boxed_ar(),
        }
    }
}

fn quote_string(s: impl AsRef<str>) -> String {
    let s = s.as_ref();
    let s = s.replace('\'', "''");
    format!("'{s}'")
}

pub fn quote_arr(arr: &JArray) -> String {
    let _shape = if arr.shape().is_empty() {
        String::new()
    } else {
        format!(
            "{} $ ",
            arr.shape().iter().map(|v| format!("{v}")).join(" ")
        )
    };

    let arr = arr
        .reshape(IxDyn(&[arr.tally()]))
        .expect("reshape to same size is infallible");

    // format!("{shape}{}", format!("{arr}").trim())
    // AA 20230305 Why did we want the shape included here?
    format!("{arr}").trim().to_string()
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
