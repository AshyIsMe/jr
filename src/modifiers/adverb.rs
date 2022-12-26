use std::fmt;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::arrays::JArrayCow;
use crate::cells::flatten_partial;
use crate::modifiers::c_atop;
use crate::number::promote_to_array;
use crate::verbs::v_self_classify;
use crate::{flatten, Arrayable, Ctx, JError, Word};

pub type AdverbFn = fn(&mut Ctx, Option<&Word>, &Word, &Word) -> Result<Word>;

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

pub fn a_not_implemented(_ctx: &mut Ctx, _x: Option<&Word>, _u: &Word, _y: &Word) -> Result<Word> {
    Err(JError::NonceError).context("blanket adverb implementation")
}

pub fn a_tilde(ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match x {
        None => match u {
            Word::Verb(_, u) => u.exec(ctx, Some(y), y).map(Word::Noun),
            _ => Err(JError::DomainError)
                .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u)),
        },
        Some(x) => match u {
            Word::Verb(_, u) => u.exec(ctx, Some(y), x).map(Word::Noun),
            _ => Err(JError::DomainError)
                .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u)),
        },
    }
}

pub fn a_slash(ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match x {
        None => match u {
            Word::Verb(_, u) => match y {
                Word::Noun(_) => y
                    .to_cells()?
                    .into_iter()
                    .map(Ok)
                    .rev()
                    // Reverse to force right to left execution.
                    // Required for (;/i.5) to work correctly.
                    // Yes we flipped y and x args in the lambda below:
                    .reduce(|y, x| u.exec(ctx, Some(&x?), &y?).map(Word::Noun))
                    .ok_or(JError::DomainError)?,
                _ => Err(JError::custom("noun expected")),
            },
            _ => Err(JError::DomainError).with_context(|| anyhow!("{:?}", u)),
        },
        Some(_x) => Err(JError::custom("dyadic / not implemented yet")),
    }
}

pub fn a_slash_dot(ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, y) {
        (Some(Word::Noun(x)), Word::Noun(y)) if x.shape().len() == 1 && y.shape().len() == 1 => {
            let classification = v_self_classify(x).context("classify")?;
            c_atop(
                ctx,
                Some(&Word::Noun(classification)),
                u,
                &Word::static_verb("#"),
                &Word::Noun(y.clone()),
            )
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} /. {y:?}")),
    }
}

/// (0 _)
pub fn a_backslash(ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, u, y) {
        (None, Word::Verb(_, u), Word::Noun(y)) => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 1..=y.len() {
                let chunk = &y[..i];
                piece.push(
                    u.exec(ctx, None, &Word::Noun(flatten_partial(chunk)?))
                        .context("backslash (u)")?,
                );
            }
            flatten(&piece.into_array()).map(Word::Noun)
        }
        (Some(Word::Noun(x)), Word::Verb(_, u), Word::Noun(y)) => {
            let x = x.approx_i64_one().context("backslash's x")?;
            let mut piece = Vec::new();
            let mut f = |chunk: &[JArrayCow]| -> Result<()> {
                piece.push(u.exec(ctx, None, &Word::Noun(flatten_partial(chunk)?))?);
                Ok(())
            };

            let size = usize::try_from(x.abs())?;
            if x < 0 {
                for chunk in y.outer_iter().collect_vec().chunks(size) {
                    f(chunk)?;
                }
            } else {
                for chunk in y.outer_iter().collect_vec().windows(size) {
                    f(chunk)?;
                }
            }

            flatten(&piece.into_array()).map(Word::Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} \\ {y:?}")),
    }
}

/// (_ 0 _)
pub fn a_suffix_outfix(ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, u, y) {
        (None, Word::Verb(_, u), Word::Noun(y)) => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 0..y.len() {
                piece.push(u.exec(ctx, None, &Word::Noun(flatten_partial(&y[i..])?))?);
            }
            flatten(&piece.into_array()).map(Word::Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} \\ {y:?}")),
    }
}

/// (_ _ _)
pub fn a_curlyrt(_ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    use Word::Noun;
    match (x, u, y) {
        (Some(Noun(x)), Noun(u), Noun(y))
            if x.shape().len() <= 1 && u.shape().len() <= 1 && y.shape().len() == 1 =>
        {
            let u = u.approx_usize_list()?;
            let x = x.clone().into_elems();
            let mut y = y.clone().into_elems();

            for u in u {
                *y.get_mut(u)
                    .ok_or(JError::IndexError)
                    .context("index out of bounds")? = x[u % x.len()].clone();
            }

            promote_to_array(y).map(Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} }} {y:?}")),
    }
}
