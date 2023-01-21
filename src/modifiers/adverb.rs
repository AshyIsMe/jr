use std::fmt;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::cells::fill_promote_reshape;
use crate::eval::VerbNoun;
use crate::modifiers::do_atop;
use crate::verbs::{v_self_classify, BivalentOwned, VerbImpl};
use crate::{primitive_verbs, Ctx, JArray, JError, Rank};

pub type AdverbFn = fn(&mut Ctx, &VerbNoun) -> Result<BivalentOwned>;

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

pub fn a_not_implemented(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    let u = u.clone();
    let biv = BivalentOwned::from_bivalent(move |_ctx, _x, _y| {
        Err(JError::NonceError)
            .context("blanket adverb implementation")
            .with_context(|| anyhow!("m/u: {u:?}"))
    });
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn a_tilde(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    let Verb(u) = u else {
        return Err(JError::DomainError)
            .with_context(|| anyhow!("expected to ~ a verb, not {:?}", u))?;
    };

    let u = u.clone();
    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
        let u = u.to_verb(ctx.eval())?;
        match x {
            None => u.exec(ctx, Some(y), y),
            Some(x) => u.exec(ctx, Some(y), x),
        }
    });

    Ok(BivalentOwned {
        biv,
        // this "depends on the rank of u", but it seems to execute as if its infinite, what have I missed?
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn a_slash(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    let Verb(u) = u else { return Err(JError::DomainError).context("verb for /'s u"); };
    let u = u.clone();
    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
        let u = u.to_verb(ctx.eval())?;
        if let Some(x) = x {
            return a_table(ctx, &u, x, y);
        }

        if let Some(arr) = u.fast_between(ctx, y)? {
            return Ok(arr);
        }

        y.outer_iter()
            .rev()
            .map(Ok)
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
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

fn a_table(ctx: &mut Ctx, u: &VerbImpl, x: &JArray, y: &JArray) -> Result<JArray> {
    let mut items = Vec::new();
    for x in x.outer_iter() {
        for y in y.outer_iter() {
            items.push(u.exec(ctx, Some(&x.to_owned()), &y.to_owned())?);
        }
    }

    fill_promote_reshape((vec![x.len_of_0(), y.len_of_0()], items))
}

pub fn a_slash_dot(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    let Verb(u  ) = u.clone() else { return Err(JError::DomainError).context("/.'s u must be a verb"); };

    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| match x {
        Some(x) if x.shape().len() <= 1 && y.shape().len() <= 1 => {
            let u = u.to_verb(ctx.eval())?;
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
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

/// (0 _)
pub fn a_backslash(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    let Verb(u) = u else { return Err(JError::DomainError).context("backslash's u must be a verb"); };
    let u = u.clone();
    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| match x {
        None => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 1..=y.len() {
                let chunk = &y[..i];
                piece.push(
                    u.exec(
                        ctx,
                        None,
                        &JArray::from_fill_promote(chunk.iter().cloned())?,
                    )
                    .context("backslash (u)")?,
                );
            }
            JArray::from_fill_promote(piece)
        }
        Some(x) => {
            let x = x.approx_i64_one().context("backslash's x")?;
            let mut piece = Vec::new();
            let mut f = |chunk: &[JArray]| -> Result<()> {
                piece.push(u.exec(
                    ctx,
                    None,
                    &JArray::from_fill_promote(chunk.iter().cloned())?,
                )?);
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
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

/// (_ 0 _)
pub fn a_suffix_outfix(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    let Verb(u) = u else { return Err(JError::DomainError).context("suffix outfix's u must be a verb"); };

    let u = u.clone();
    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| match x {
        None => {
            let y = y.outer_iter().collect_vec();
            let mut piece = Vec::new();
            for i in 0..y.len() {
                piece.push(u.exec(
                    ctx,
                    None,
                    &JArray::from_fill_promote(y[i..].iter().cloned())?,
                )?);
            }
            JArray::from_fill_promote(piece)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {u:?} \\. {y:?}")),
    });

    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

/// (_ _ _)
pub fn a_curlyrt(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    match u {
        Noun(noun) => build_curlrt(noun),
        Verb(u) => {
            let u = u.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                let u = u.exec(ctx, None, y)?;
                (build_curlrt(&u)?.biv)(ctx, x, y)
            });
            Ok(BivalentOwned {
                biv,
                ranks: Rank::inf_inf_inf(),
            })
        }
    }
}

fn build_curlrt(u: &JArray) -> Result<BivalentOwned> {
    if u.shape().len() > 1 {
        return Err(JError::NonceError).context("u must be a list");
    }

    if u.is_empty() {
        return Err(JError::LengthError).context("u can't be empty");
    }

    let u = u.approx_usize_list()?;
    let biv = BivalentOwned::from_bivalent(move |_ctx, x, y| match x {
        Some(x) => {
            if x.is_empty() {
                return Err(JError::LengthError).context("x cannot be empty");
            }
            let x = x.outer_iter().collect_vec();
            let mut y = y.outer_iter().collect_vec();

            for (i, u) in u.iter().enumerate() {
                *y.get_mut(*u)
                    .ok_or(JError::IndexError)
                    .context("index out of bounds")? = x[i % x.len()].clone();
            }

            JArray::from_fill_promote(y)
        }
        None => Err(JError::NonceError).context("monadic }"),
    });

    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn a_bdot(_ctx: &mut Ctx, u: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    match u {
        Noun(m) => {
            let m = m.approx_i64_one().context("b.'s mode")?;
            if m < -16 || m > 34 {
                return Err(JError::DomainError).context("impossible b. mode");
            }
            Ok(BivalentOwned {
                biv: BivalentOwned::from_bivalent(move |_ctx, _x, _y| {
                    Err(JError::NonceError).with_context(|| anyhow!("b.'s mode {m}"))
                }),
                ranks: (Rank::zero(), Rank::zero_zero()),
            })
        }
        Verb(_) => Err(JError::NonceError).with_context(|| anyhow!("b. verb info for {u:?}")),
    }
}
