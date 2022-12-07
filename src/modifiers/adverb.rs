use std::fmt;

use anyhow::{anyhow, Context, Result};

use crate::modifiers::c_at;
use crate::verbs::v_self_classify;
use crate::{JError, Word};

pub type AdverbFn = fn(Option<&Word>, &Word, &Word) -> Result<Word>;

#[derive(Clone)]
pub struct SimpleAdverb {
    pub name: &'static str,
    pub f: AdverbFn,
}

impl PartialEq for SimpleAdverb {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl fmt::Debug for SimpleAdverb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleAdverb({:?})", self.name)
    }
}

pub fn a_not_implemented(_x: Option<&Word>, _u: &Word, _y: &Word) -> Result<Word> {
    Err(JError::NonceError).context("blanket adverb implementation")
}

pub fn a_tilde(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match x {
        None => match u {
            Word::Verb(_, u) => u.exec(Some(y), y).map(Word::Noun),
            _ => Err(JError::DomainError)
                .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u)),
        },
        Some(x) => match u {
            Word::Verb(_, u) => u.exec(Some(y), x).map(Word::Noun),
            _ => Err(JError::DomainError)
                .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u)),
        },
    }
}

pub fn a_slash(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match x {
        None => match u {
            Word::Verb(_, u) => match y {
                Word::Noun(_) => y
                    .to_cells()?
                    .into_iter()
                    .map(Ok)
                    .reduce(|x, y| u.exec(Some(&x?), &y?).map(Word::Noun))
                    .ok_or(JError::DomainError)?,
                _ => Err(JError::custom("noun expected")),
            },
            _ => Err(JError::DomainError).with_context(|| anyhow!("{:?}", u)),
        },
        Some(_x) => Err(JError::custom("dyadic / not implemented yet")),
    }
}

pub fn a_slash_dot(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, y) {
        (Some(Word::Noun(x)), Word::Noun(y)) if x.shape().len() == 1 && y.shape().len() == 1 => {
            let classification = v_self_classify(y).context("classify")?;
            c_at(
                Some(&Word::Noun(classification)),
                u,
                &Word::static_verb("#"),
                &Word::Noun(y.clone()),
            )
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} /. {y:?}")),
    }
}
