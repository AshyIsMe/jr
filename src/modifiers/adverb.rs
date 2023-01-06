use std::fmt;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::arrays::JArrayCow;
use crate::cells::fill_promote_list_cow;
use crate::modifiers::do_atop;
use crate::number::promote_to_array;
use crate::verbs::{v_self_classify, PartialImpl, VerbImpl};
use crate::{primitive_verbs, Ctx, JArray, JError, Rank, Word};

pub type AdverbFn = fn(&mut Ctx, &Word) -> Result<Word>;

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
        write!(f, "SimpleAdverb2({:?})", self.name)
    }
}

pub fn a_not_implemented(_ctx: &mut Ctx, _u: &Word) -> Result<Word> {
    Err(JError::NonceError).context("blanket adverb implementation")
}

pub fn a_tilde(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(u) = u else { return Err(JError::DomainError)
        .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u)) };

    let u = u.clone();
    let biv = PartialImpl::from_legacy_inf(move |ctx, x, y| match x {
        None => u.exec(ctx, Some(y), y),
        Some(x) => u.exec(ctx, Some(y), x),
    });

    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        name: format!("?~"),
        biv,
        // this "depends on the rank of u", but it seems to execute as if its infinite, what have I missed?
        ranks: Rank::inf_inf_inf(),
    })))
}

pub fn a_slash(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(u) = u else { return Err(JError::DomainError).context("verb for /'s u"); };
    let u = u.clone();
    let biv = PartialImpl::from_legacy_inf(move |ctx, x, y| {
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
                u.exec(ctx, Some(&x), &y)
            })
            .ok_or(JError::DomainError)?
    });
    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        name: "/?".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })))
}

pub fn a_slash_dot(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(u  ) = u.clone() else { return Err(JError::DomainError).context("/.'s u must be a verb"); };

    let biv = PartialImpl::from_legacy_inf(move |ctx, x, y| match x {
        Some(x) if x.shape().len() <= 1 && y.shape().len() <= 1 => {
            let classification = v_self_classify(x).context("classify")?;
            do_atop(
                ctx,
                Some(&classification),
                &u,
                &primitive_verbs("#").expect("tally always exists"),
                y,
            )
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} /. {y:?}")),
    });
    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        name: "/.?".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })))
}

/// (0 _)
pub fn a_backslash(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(u) = u else { return Err(JError::DomainError).context("backslash's u must be a verb"); };
    let u = u.clone();
    let biv = PartialImpl::from_legacy_inf(move |ctx, x, y| match x {
        None => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 1..=y.len() {
                let chunk = &y[..i];
                piece.push(
                    u.exec(ctx, None, &fill_promote_list_cow(chunk)?)
                        .context("backslash (u)")?,
                );
            }
            JArray::from_fill_promote(piece)
        }
        Some(x) => {
            let x = x.approx_i64_one().context("backslash's x")?;
            let mut piece = Vec::new();
            let mut f = |chunk: &[JArrayCow]| -> Result<()> {
                piece.push(u.exec(ctx, None, &fill_promote_list_cow(chunk)?)?);
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

            JArray::from_fill_promote(piece)
        }
    });
    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        name: "\\?".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })))
}

/// (_ 0 _)
pub fn a_suffix_outfix(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Verb(u) = u else { return Err(JError::DomainError).context("suffix outfix's u must be a verb"); };

    let u = u.clone();
    let biv = PartialImpl::from_legacy_inf(move |ctx, x, y| match x {
        None => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 0..y.len() {
                piece.push(u.exec(ctx, None, &fill_promote_list_cow(&y[i..])?)?);
            }
            JArray::from_fill_promote(piece)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} \\. {y:?}")),
    });

    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        name: "\\.?".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })))
}

/// (_ _ _)
pub fn a_curlyrt(_ctx: &mut Ctx, u: &Word) -> Result<Word> {
    let Word::Noun(u) = u else { return Err(JError::DomainError).context("}'s u must be a noun"); };
    if u.shape().len() > 1 {
        return Err(JError::NonceError).context("u must be a list");
    }
    let u = u.approx_usize_list()?;
    let biv = PartialImpl::from_legacy_inf(move |_ctx, x, y| match x {
        Some(x) if x.shape().len() <= 1 && y.shape().len() == 1 => {
            let x = x.clone().into_elems();
            let mut y = y.clone().into_elems();

            for u in &u {
                *y.get_mut(*u)
                    .ok_or(JError::IndexError)
                    .context("index out of bounds")? = x[u % x.len()].clone();
            }

            promote_to_array(y)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} }} {y:?}")),
    });

    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        name: "?}".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })))
}
