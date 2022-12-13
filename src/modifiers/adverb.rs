use std::fmt;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::arrays::JArrayCow;
use crate::modifiers::c_atop;
use crate::verbs::v_self_classify;
use crate::{flatten, Arrayable, JArray, JError, Word};

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
            let classification = v_self_classify(x).context("classify")?;
            c_atop(
                Some(&Word::Noun(classification)),
                u,
                &Word::static_verb("#"),
                &Word::Noun(y.clone()),
            )
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} /. {y:?}")),
    }
}

fn flatten_partial(chunk: &[JArrayCow]) -> Result<JArray> {
    flatten(
        &chunk
            .iter()
            .map(|arr| JArray::from(arr.clone()))
            .collect_vec()
            .into_array()?,
    )
}

/// (0 _)
pub fn a_backslash(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, u, y) {
        (Some(Word::Noun(x)), Word::Verb(_, u), Word::Noun(y)) => {
            let x = x
                .single_math_num()
                .ok_or(JError::DomainError)
                .context("infix needs a number")?
                .value_i64()
                .ok_or(JError::DomainError)
                .context("infix needs an int")?;
            let mut piece = Vec::new();
            let mut f = |chunk: &[JArrayCow]| -> Result<()> {
                piece.push(u.exec(None, &Word::Noun(flatten_partial(chunk)?))?);
                Ok(())
            };

            let size = usize::try_from(x.abs())?;
            if x < 0 {
                for chunk in y.outer_iter().chunks(size) {
                    f(chunk)?;
                }
            } else {
                for chunk in y.outer_iter().windows(size) {
                    f(chunk)?;
                }
            }

            flatten(&piece.into_array()?).map(Word::Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} \\ {y:?}")),
    }
}

/// (_ 0 _)
pub fn a_suffix_outfix(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, u, y) {
        (None, Word::Verb(_, u), Word::Noun(y)) => {
            let y = y.outer_iter();
            let mut piece = Vec::new();
            for i in 0..y.len() {
                piece.push(u.exec(None, &Word::Noun(flatten_partial(&y[i..])?))?);
            }
            flatten(&piece.into_array()?).map(Word::Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} \\ {y:?}")),
    }
}
