use std::fmt;
use std::iter;

use anyhow::{anyhow, ensure, Context, Result};
use itertools::Itertools;
use ndarray::prelude::*;

use crate::cells::{apply_cells, fill_promote_reshape, monad_cells};
use crate::eval::{create_def, resolve_controls};
use crate::foreign::foreign;
use crate::verbs::{exec_dyad, exec_monad, BivalentOwned, Rank, VerbImpl};
use crate::{arr0d, generate_cells, primitive_verbs, Ctx};
use crate::{HasEmpty, JArray, JError, Word};

#[derive(Clone)]
pub struct SimpleConjunction {
    pub name: &'static str,
    pub f: fn(&mut Ctx, &Word, &Word) -> Result<BivalentOwned>,
}

impl PartialEq for SimpleConjunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl fmt::Debug for SimpleConjunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleConjunction({:?})", self.name)
    }
}

#[derive(Clone)]
pub struct WordyConjunction {
    pub name: &'static str,
    pub f: fn(&mut Ctx, &Word, &Word) -> Result<Word>,
}

impl PartialEq for WordyConjunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl fmt::Debug for WordyConjunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WordyConjunction({:?})", self.name)
    }
}

pub fn c_not_implemented(_ctx: &mut Ctx, _u: &Word, _v: &Word) -> Result<BivalentOwned> {
    Err(JError::NonceError).context("blanket conjunction implementation")
}

pub fn c_hatco(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<BivalentOwned> {
    // TODO: inverse, converge and Dynamic Power (verb argument)
    // https://code.jsoftware.com/wiki/Vocabulary/hatco
    let u = u.clone();
    let v = v.clone();
    let biv = match (u, v) {
        (Word::Verb(u), Word::Noun(ja)) => {
            // TODO: this should support _infinite
            let n = ja
                .to_i64()
                .ok_or(JError::DomainError)
                .context("hatco's noun should be integers")?
                .into_owned();
            BivalentOwned::from_bivalent(move |ctx, x, y| do_hatco(ctx, x, &u, &n, y))
        }
        (Word::Verb(u), Word::Verb(v)) => BivalentOwned::from_bivalent(move |ctx, x, y| {
            let n = v.exec(ctx, x, y)?;
            // TODO: this should support _infinite
            let n = n
                .to_i64()
                .ok_or(JError::DomainError)
                .context("hatco's (derived) noun should be integers")?
                .into_owned();

            do_hatco(ctx, x, &u, &n, y)
        }),
        (u, v) => return Err(JError::DomainError).with_context(|| anyhow!("{u:?} {v:?}")),
    };

    Ok(BivalentOwned {
        name: "hatco".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

fn do_hatco(
    ctx: &mut Ctx,
    x: Option<&JArray>,
    u: &VerbImpl,
    n: &ArrayD<i64>,
    y: &JArray,
) -> Result<JArray> {
    fill_promote_reshape(&(
        n.shape().to_vec(),
        n.iter()
            .map(|i| -> Result<_> {
                let mut t = y.clone();
                for _ in 0..*i {
                    t = u.exec(ctx, x, &t)?;
                }
                Ok(t)
            })
            .collect::<Result<Vec<JArray>>>()?,
    ))
}

pub fn c_quote(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<BivalentOwned> {
    let biv = match (u, v) {
        (Word::Verb(u), Word::Noun(n)) => {
            let n = n
                .approx()
                .ok_or(JError::DomainError)
                .context("rank expects integer arguments")?;

            let ranks = match (n.shape().len(), n.len()) {
                (0, 1) => {
                    let only = n.iter().next().copied().expect("checked the length");
                    [only, only, only]
                }
                (1, 1) => [n[0], n[0], n[0]],
                (1, 2) => [n[1], n[0], n[1]],
                (1, 3) => [n[0], n[1], n[2]],
                _ => {
                    return Err(JError::LengthError).with_context(|| {
                        anyhow!("rank operator requires a list of 1-3 elements, not: {n:?}")
                    })
                }
            };

            let ranks = (
                Rank::from_approx(ranks[0])?,
                Rank::from_approx(ranks[1])?,
                Rank::from_approx(ranks[2])?,
            );

            let u = u.clone();
            BivalentOwned::from_bivalent(move |ctx, x, y| match x {
                None => exec_monad(
                    |y| {
                        u.exec(ctx, None, y)
                            .context("running monadic u inside re-rank")
                    },
                    ranks.0,
                    y,
                )
                .context("monadic rank drifting"),
                Some(x) => exec_dyad(
                    |x, y| {
                        u.exec(ctx, Some(x), y)
                            .context("running dyadic u inside re-rank")
                    },
                    (ranks.1, ranks.2),
                    x,
                    y,
                )
                .context("dyadic rank drifting"),
            })
        }
        (Word::Noun(u), Word::Noun(n)) => {
            let n = n
                .approx()
                .ok_or(JError::DomainError)
                .context("rank expects integer arguments")?;
            if n != arr0d(f32::INFINITY) {
                return Err(JError::NonceError).context("only infinite ranks");
            }
            let u = u.clone();
            BivalentOwned::from_bivalent(move |_ctx, _x, _y| Ok(u.clone()))
        }
        _ => {
            return Err(JError::NonceError)
                .with_context(|| "rank conjunction - other options? {x:?}, {u:?}, {v:?}, {y:?}")
        }
    };

    Ok(BivalentOwned {
        name: "\"".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn c_tie(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<Word> {
    Ok(Word::Noun(JArray::from_list(vec![
        u.boxed_ar()?,
        v.boxed_ar()?,
    ])))
}

pub fn c_agenda(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<Word> {
    use Word::*;

    let Noun(v) = v else { return Err(JError::NounResultWasRequired).context("agenda's index type"); };
    let v = v.approx_usize_one().context("agenda's v")?;

    let Noun(JArray::BoxArray(u)) = u else { return Err(JError::DomainError).context("agenda's box array"); };

    if u.shape().len() > 1 {
        return Err(JError::NonceError).context("@. only implemented for lists");
    }

    let verb = u
        .iter()
        .nth(v)
        .ok_or(JError::IndexError)
        .context("gerund out of bounds")?;
    let JArray::CharArray(verb) = verb else { return Err(JError::DomainError).context("gerunds are strings"); };
    if verb.len() > 1 {
        return Err(JError::DomainError).context("gerunds are single strings");
    }

    let name = verb.iter().collect::<String>();
    let verb = primitive_verbs(&name)
        .ok_or(JError::NonceError)
        .context("unable to match *primitive* verb")?;

    Ok(Verb(verb))
}

// https://code.jsoftware.com/wiki/Vocabulary/at#/media/File:Funcomp.png
pub fn c_atop(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<BivalentOwned> {
    match (u, v) {
        (Word::Verb(u), Word::Verb(v)) => {
            let u = u.clone();
            let v = v.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| do_atop(ctx, x, &u, &v, y));
            Ok(BivalentOwned {
                name: "atop".to_string(),
                biv,
                ranks: Rank::inf_inf_inf(),
            })
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("expected to verb @ verb, not {u:?} @ {v:?}")),
    }
}

pub fn do_atop(
    ctx: &mut Ctx,
    x: Option<&JArray>,
    u: &VerbImpl,
    v: &VerbImpl,
    y: &JArray,
) -> Result<JArray> {
    let mut r = v.partial_exec(ctx, x, y).context("right half of c_atop")?;
    // surely this private field access indicates a design problem of some kind
    r.1 =
        r.1.into_iter()
            .map(|a| u.exec(ctx, None, &a))
            .collect::<Result<Vec<_>>>()
            .context("left half of c_at")?;

    fill_promote_reshape(&r).context("expanding result of c_atop")
}

// https://code.jsoftware.com/wiki/Vocabulary/at#/media/File:Funcomp.png
pub fn c_at(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<BivalentOwned> {
    match (u, v) {
        (Word::Verb(u), Word::Verb(v)) => {
            let u = u.clone();
            let v = v.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                let r = v.partial_exec(ctx, x, y).context("right half of c_at")?;
                let r = fill_promote_reshape(&r).context("expanding result of c_atop")?;
                u.exec(ctx, None, &r).context("left half of c_at")
            });
            Ok(BivalentOwned {
                name: "at".to_string(),
                biv,
                ranks: Rank::inf_inf_inf(),
            })
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("expected to verb @: verb, not {u:?} @: {v:?}")),
    }
}

pub fn c_cor(_ctx: &mut Ctx, n: &Word, m: &Word) -> Result<(bool, Word)> {
    use crate::arrays::JArray::*;
    let Word::Noun(n) = n else { return Err(JError::DomainError).context("cor's n") };
    let n = n.approx_i64_one()?;
    match m {
        Word::Noun(arr) if arr.approx_i64_one().ok() == Some(0) => {
            Ok((true, Word::Noun(JArray::from(arr0d(n)))))
        }
        Word::Noun(CharArray(jcode)) if jcode.shape().len() <= 1 => {
            let n = match n {
                0 => return Ok((false, Word::Noun(CharArray(jcode.clone())))),
                3 => 'm',
                4 => 'd',
                _ => return Err(JError::NonceError).context("unsupported cor mode"),
            };
            let jcode = jcode.iter().collect::<String>();
            let mut words = crate::scan(&jcode)?;
            if !resolve_controls(&mut words)? {
                return Err(JError::SyntaxError).context("unable to resolve controls in cor");
            }

            Ok((false, create_def(n, words)?))
        }
        _ => Err(JError::DomainError).with_context(|| anyhow!("{n:?} {m:?}")),
    }
}

pub fn c_assign_adverse(_ctx: &mut Ctx, n: &Word, m: &Word) -> Result<BivalentOwned> {
    match (n, m) {
        (Word::Verb(n), Word::Verb(m)) => {
            let n = n.clone();
            let m = m.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                n.exec(ctx, x, y).or_else(|_| m.exec(ctx, x, y))
            });
            Ok(BivalentOwned {
                name: format!("?::?"),
                biv,
                ranks: Rank::inf_inf_inf(),
            })
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("\nn: {n:?}\nm: {m:?}")),
    }
}

pub fn c_cut(_ctx: &mut Ctx, n: &Word, m: &Word) -> Result<BivalentOwned> {
    use Word::*;
    let Noun(m) = m else { return Err(JError::DomainError).context("cut's mode arg"); };
    let Verb(v) = n.clone() else { return Err(JError::DomainError).context("cut's verb arg"); };
    let m = m.approx_i64_one().context("cut's m")?;

    let is_end = match m {
        2 | -2 => true,
        1 | -1 => false,
        _ => return Err(JError::DomainError).context("invalid mode for dyadic cut"),
    };

    let is_inclusive = match m {
        -2 | -1 => false,
        2 | 1 => true,
        _ => return Err(JError::DomainError).context("invalid mode for dyadic cut"),
    };

    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
        let parts = y.outer_iter().collect_vec();
        ensure!(!parts.is_empty());

        let frets: Vec<usize> = match x {
            None => {
                let key = if is_end {
                    &parts[parts.len() - 1]
                } else {
                    &parts[0]
                };
                let mut frets = parts.iter().positions(|part| part == key).collect_vec();
                if !is_end {
                    frets.push(parts.len());
                }
                frets
            }
            Some(JArray::BoolArray(x)) if x.shape().len() == 1 => {
                x.iter().positions(|part| *part == 1).collect()
            }

            _ => return Err(JError::NonceError).with_context(|| anyhow!("{x:?} ? ;. ? {y:?}")),
        };

        let out = cut_frets(&frets, is_inclusive, is_end)
            .into_iter()
            .map(|(s, e)| &parts[s..e])
            .map(|sub| {
                let sub = if sub.is_empty() {
                    JArray::empty()
                } else {
                    JArray::from_fill_promote(sub.iter().map(|v| v.to_owned()))
                        .context("flattening intermediate")?
                };
                v.exec(ctx, None, &sub).context("evaluating intermediate")
            })
            .collect::<Result<Vec<_>>>()?;

        JArray::from_fill_promote(out)
    });

    Ok(BivalentOwned {
        name: "?;.?".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

fn cut_frets(
    frets: &[usize],
    is_inclusive: bool,
    is_end: bool,
) -> Box<dyn Iterator<Item = (usize, usize)> + '_> {
    if is_end {
        let it = iter::once(0)
            .chain(frets.iter().map(|x| *x + 1))
            .tuple_windows();
        if is_inclusive {
            Box::new(it)
        } else {
            Box::new(it.map(|(s, e)| (s, e - 1)))
        }
    } else {
        let it = frets.iter().copied().tuple_windows();
        if is_inclusive {
            Box::new(it)
        } else {
            Box::new(it.map(|(s, e)| (s + 1, e)))
        }
    }
}

#[cfg(test)]
mod test_cut {
    use super::cut_frets;
    use itertools::Itertools;

    #[test]
    fn test_cut_inc_end() {
        assert_eq!(
            cut_frets(&[0, 3, 4, 6], true, true).collect_vec(),
            vec![(0, 1), (1, 4), (4, 5), (5, 7)]
        );

        assert_eq!(
            cut_frets(&[3, 4, 6], true, true).collect_vec(),
            vec![(0, 4), (4, 5), (5, 7)]
        );
    }

    #[test]
    fn test_cut_exc_end() {
        assert_eq!(
            cut_frets(&[0, 3, 4, 6], false, true).collect_vec(),
            vec![(0, 0), (1, 3), (4, 4), (5, 6)]
        );

        assert_eq!(
            cut_frets(&[3, 4, 6], false, true).collect_vec(),
            vec![(0, 3), (4, 4), (5, 6)]
        );
    }

    #[test]
    fn test_cut_inc_start() {
        assert_eq!(
            cut_frets(&[0, 3, 4, 6, 7], true, false).collect_vec(),
            vec![(0, 3), (3, 4), (4, 6), (6, 7)]
        );

        assert_eq!(
            cut_frets(&[0, 3, 4, 6], true, false).collect_vec(),
            vec![(0, 3), (3, 4), (4, 6)]
        );
    }
}

pub fn c_foreign(_ctx: &mut Ctx, l: &Word, r: &Word) -> Result<BivalentOwned> {
    match (l, r) {
        (Word::Noun(l), Word::Noun(r)) => {
            let l = l.approx_i64_one().context("foreign's left")?;
            let r = r.approx_i64_one().context("foreign's right")?;
            Ok(foreign(l, r)?)
        }
        _ => Err(JError::NonceError).context("unsupported foreign syntax"),
    }
}

pub fn c_bondo(_ctx: &mut Ctx, n: &Word, m: &Word) -> Result<BivalentOwned> {
    // TODO: some of these are presumably obviously monads or dyads
    let biv = match (n.clone(), m.clone()) {
        (Word::Verb(n), Word::Noun(m)) => BivalentOwned::from_bivalent(move |ctx, _x, y| {
            n.exec(ctx, Some(&m), &y.clone()).context("monad bondo VN")
        }),
        (Word::Noun(n), Word::Verb(m)) => BivalentOwned::from_bivalent(move |ctx, _x, y| {
            m.exec(ctx, Some(&n), y).context("monad bondo NV")
        }),
        (Word::Verb(u), Word::Verb(v)) => BivalentOwned::from_bivalent(move |ctx, x, y| match x {
            None => do_atop(ctx, None, &u, &v, &y),
            Some(x) => {
                let l = v.exec(ctx, None, &x).context("left bondo NVV")?;
                let r = v.exec(ctx, None, &y).context("right bondo NVV")?;
                u.exec(ctx, Some(&l), &r).context("central bondo NVV")
            }
        }),
        _ => return Err(JError::NonceError).with_context(|| anyhow!("bondo n:{n:?} m:{m:?}")),
    };
    Ok(BivalentOwned {
        name: "bondo".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn c_under(_ctx: &mut Ctx, n: &Word, m: &Word) -> Result<BivalentOwned> {
    let (u, v) = match (n, m) {
        (Word::Verb(n), Word::Verb(m)) => (n, m),
        _ => return Err(JError::NonceError).with_context(|| anyhow!("under dual n:{n:?} m:{m:?}")),
    };
    let vi = v
        .obverse()
        .ok_or(JError::NonceError)
        .context("lacking obverse")?;
    let u = u.clone();
    let v = v.clone();
    let vi = vi.clone();
    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| match x {
        None => {
            let (cells, frame) = monad_cells(y, Rank::zero())?;
            let mut parts = Vec::new();
            for y in cells {
                let v = v.exec(ctx, None, &y).context("under dual v")?;
                let u = u.exec(ctx, None, &v).context("under dual u")?;
                let vi = vi.exec(ctx, None, &u).context("under dual vi")?;
                parts.push(vi);
            }
            JArray::from_fill_promote(parts)?
                .to_shape(frame)
                .map(|cow| cow.to_owned())
        }
        Some(x) => {
            let vr = v
                .dyad_rank()
                .ok_or(JError::NonceError)
                .context("missing rank for dyad")?;
            let (frame, cells) = generate_cells(x.clone(), y.clone(), vr)?;
            let parts = apply_cells(
                &cells,
                |x, y| {
                    let l = v.exec(ctx, None, x).context("under dual l")?;
                    let r = v.exec(ctx, None, y).context("under dual r")?;
                    let u = u.exec(ctx, Some(&l), &r).context("under dual u")?;
                    vi.exec(ctx, None, &u).context("under dual vi")
                },
                vr,
            )?;
            JArray::from_fill_promote(parts)?
                .to_shape(frame)
                .map(|cow| cow.to_owned())
        }
    });
    Ok(BivalentOwned {
        name: "under".to_string(),
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}
