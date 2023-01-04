use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::arrays::JArrayCow;
use crate::cells::fill_promote_list_cow;
use crate::modifiers::do_atop;
use crate::number::promote_to_array;
use crate::verbs::{v_self_classify, DyadOwned, MonadOwned, PartialImpl, VerbImpl};
use crate::{primitive_verbs, Ctx, JArray, JError, Rank, Word};

pub type AdverbFn = fn(&mut Ctx, Option<&Word>, &Word, &Word) -> Result<Word>;
pub type AdverbFn2 = fn(&mut Ctx, &Word) -> Result<Word>;

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

#[derive(Clone)]
pub struct SimpleAdverb2 {
    pub name: &'static str,
    pub f: AdverbFn2,
}

impl PartialEq for SimpleAdverb2 {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl fmt::Debug for SimpleAdverb2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleAdverb2({:?})", self.name)
    }
}

pub fn a_not_implemented(_ctx: &mut Ctx, _u: &Word) -> Result<Word> {
    Err(JError::NonceError).context("blanket adverb implementation")
}

pub fn a_tilde(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(su, u) = u else { return Err(JError::DomainError)
        .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u)) };

    let mu = u.clone();

    // are we supposed to be, like, not generating these functions if they don't exist?
    let monad = Some(MonadOwned {
        // this "depends on the rank of u", but it seems to execute as if its infinite, what have I missed?
        rank: Rank::infinite(),
        // rank: mu
        //     .monad_rank()
        //     .ok_or(JError::NonceError)
        //     .context("can only ~ ranked verbs")?,
        f: Arc::new(move |ctx, y| {
            let y = Word::Noun(y.clone());
            mu.exec(ctx, Some(&y), &y).map(Word::Noun)
        }),
    });

    let du = u.clone();
    let dyad = Some(DyadOwned {
        rank: Rank::infinite_infinite(),
        // rank: du
        //     .dyad_rank()
        //     .ok_or(JError::NonceError)
        //     .context("can only ~ ranked verbs")?,
        f: Arc::new(move |ctx, x, y| {
            let x = Word::Noun(x.clone());
            let y = Word::Noun(y.clone());
            du.exec(ctx, Some(&y), &x).map(Word::Noun)
        }),
    });

    Ok(Word::Verb(
        format!("{su}~"),
        VerbImpl::Partial(PartialImpl {
            name: format!("{su}~"),
            monad,
            dyad,
        }),
    ))
}

pub fn a_slash(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(_, u) = u else { return Err(JError::DomainError).context("verb for /'s u"); };
    let u = u.clone();
    let (monad, dyad) = PartialImpl::from_legacy_inf(move |ctx, x, y| {
        if x.is_some() {
            return Err(JError::NonceError).context("dyadic / not implemented yet");
        }
        y.outer_iter()
            .collect_vec()
            .into_iter()
            .map(|x| Ok(x.to_owned()))
            .rev()
            // Reverse to force right to left execution.
            // Required for (;/i.5) to work correctly.
            // Yes we flipped y and x args in the lambda below:
            .reduce(|y, x| {
                let x = x?;
                let y = y?;
                u.exec(ctx, Some(&Word::Noun(x)), &Word::Noun(y))
            })
            .ok_or(JError::DomainError)?
            .map(Word::Noun)
    });
    Ok(Word::Verb(
        "/?".to_string(),
        VerbImpl::Partial(PartialImpl {
            name: "/?".to_string(),
            monad,
            dyad,
        }),
    ))
}

pub fn a_slash_dot(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(_, u  ) = u.clone() else { return Err(JError::DomainError).context("/.'s u must be a verb"); };

    let (monad, dyad) = PartialImpl::from_legacy_inf(move |ctx, x, y| match x {
        Some(x) if x.shape().len() <= 1 && y.shape().len() <= 1 => {
            let classification = v_self_classify(x).context("classify")?;
            do_atop(
                ctx,
                Some(&Word::Noun(classification)),
                &u,
                &primitive_verbs("#").expect("tally always exists"),
                &Word::Noun(y.clone()),
            )
            .map(Word::Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} /. {y:?}")),
    });
    Ok(Word::Verb(
        "/.?".to_string(),
        VerbImpl::Partial(PartialImpl {
            name: "/.?".to_string(),
            monad,
            dyad,
        }),
    ))
}

/// (0 _)
pub fn a_backslash(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(_, u) = u else { return Err(JError::DomainError).context("backslash's u must be a verb"); };
    let u = u.clone();
    let (monad, dyad) = PartialImpl::from_legacy_inf(move |ctx, x, y| match x {
        None => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 1..=y.len() {
                let chunk = &y[..i];
                piece.push(
                    u.exec(ctx, None, &Word::Noun(fill_promote_list_cow(chunk)?))
                        .context("backslash (u)")?,
                );
            }
            JArray::from_fill_promote(piece).map(Word::Noun)
        }
        Some(x) => {
            let x = x.approx_i64_one().context("backslash's x")?;
            let mut piece = Vec::new();
            let mut f = |chunk: &[JArrayCow]| -> Result<()> {
                piece.push(u.exec(ctx, None, &Word::Noun(fill_promote_list_cow(chunk)?))?);
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

            JArray::from_fill_promote(piece).map(Word::Noun)
        }
    });
    Ok(Word::Verb(
        "\\?".to_string(),
        VerbImpl::Partial(PartialImpl {
            name: "\\?".to_string(),
            monad,
            dyad,
        }),
    ))
}

/// (_ 0 _)
pub fn a_suffix_outfix(ctx: &mut Ctx, x: Option<&Word>, u: &Word, y: &Word) -> Result<Word> {
    match (x, u, y) {
        (None, Word::Verb(_, u), Word::Noun(y)) => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 0..y.len() {
                piece.push(u.exec(ctx, None, &Word::Noun(fill_promote_list_cow(&y[i..])?))?);
            }
            JArray::from_fill_promote(piece).map(Word::Noun)
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
