use std::fmt;
use std::iter;
use std::sync::Arc;

use anyhow::{anyhow, ensure, Context, Error, Result};
use itertools::Itertools;
use ndarray::prelude::*;

use crate::arrays::BoxArray;
use crate::cells::{apply_cells, fill_promote_reshape, monad_cells};
use crate::eval::{create_def, resolve_controls, VerbNoun};
use crate::foreign::foreign;
use crate::scan::str_to_primitive;
use crate::verbs::{append_nd, exec_dyad, exec_monad, BivalentOwned, PartialImpl, Rank, VerbImpl};
use crate::{arr0d, generate_cells, Ctx};
use crate::{HasEmpty, JArray, JError, Word};

#[derive(Clone)]
pub struct SimpleConjunction {
    pub name: &'static str,
    pub f: fn(&mut Ctx, &VerbNoun, &VerbNoun) -> Result<BivalentOwned>,
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

#[derive(Clone)]
pub struct OwnedAdverb {
    pub f: Arc<dyn Fn(&mut Ctx, &Word) -> Result<Word>>,
}

impl PartialEq for OwnedAdverb {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl fmt::Debug for OwnedAdverb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OwnedAdverb")
    }
}

#[derive(Clone)]
pub struct OwnedConjunction {
    pub f: Arc<dyn Fn(&mut Ctx, Option<&Word>, &Word) -> Result<Word>>,
}

impl PartialEq for OwnedConjunction {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl fmt::Debug for OwnedConjunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OwnedConjunction")
    }
}

pub fn c_not_implemented(_ctx: &mut Ctx, u: &VerbNoun, v: &VerbNoun) -> Result<BivalentOwned> {
    let u = u.clone();
    let v = v.clone();
    let biv = BivalentOwned::from_bivalent(move |_ctx, _x, _y| {
        Err(JError::NonceError)
            .context("blanket conjunction implementation")
            .with_context(|| anyhow!("m/u: {u:?}"))
            .with_context(|| anyhow!("n/v: {v:?}"))
    });
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn c_hatco(ctx: &mut Ctx, u: &VerbNoun, v: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;

    // TODO: inverse, converge and Dynamic Power (verb argument)
    // https://code.jsoftware.com/wiki/Vocabulary/hatco
    let u = u.clone();
    let v = v.clone();
    let make_n = |ja: &JArray| -> Result<ArrayD<i64>> {
        // TODO: this should support _infinite
        Ok(ja
            .to_i64()
            .ok_or(JError::DomainError)
            .context("hatco's noun should be integers")?
            .into_owned())
    };

    let biv = match (u, v) {
        (Verb(u), Noun(ja)) => {
            match ja {
                JArray::BoxArray(b)
                    if b.shape().is_empty() && b.iter().next().expect("atom").is_empty() =>
                {
                    let u = u.clone();
                    BivalentOwned::from_bivalent(move |ctx, x, y| {
                        let mut values = Vec::with_capacity(16);
                        values.push(y.clone());
                        let mut prev = u.exec(ctx, x, y)?;
                        loop {
                            values.push(prev.clone());
                            let cand = u.exec(ctx, x, &prev)?;
                            if cand == prev {
                                break;
                            }
                            prev = cand;
                        }
                        Ok(JArray::from_fill_promote(values)?)
                    })
                }
                JArray::BoxArray(b) if b.is_empty() || b.shape().is_empty() => {
                    return Err(JError::NonceError).context("hatco on boxed atoms");
                }
                JArray::BoxArray(b) => {
                    // x u^:(v0`v1`v2)y <==> (x v0 y)u^:(x v1 y) (x v2 y)
                    let b = b
                        .iter()
                        .map(|item| untie(ctx, item))
                        .collect::<Result<Vec<_>>>()?;
                    let (v0, v1, v2) = match &b[..] {
                        [Word::Verb(v1), Word::Verb(v2)] => (None, v1, v2),
                        [Word::Verb(v0), Word::Verb(v1), Word::Verb(v2)] => (Some(v0), v1, v2),
                        _ => {
                            return Err(JError::DomainError).with_context(|| {
                                anyhow!("hatco's gerund is the wrong shape: {:#?}", b)
                            })
                        }
                    };

                    let v0 = v0.cloned();
                    let v1 = v1.clone();
                    let v2 = v2.clone();

                    BivalentOwned::from_bivalent(move |ctx, x, y| {
                        let u = u.to_verb(ctx.eval())?;
                        let x = match (x, &v0) {
                            (Some(x), Some(v0)) => Some(v0.exec(ctx, Some(x), y)?),
                            _ => None,
                        };
                        let n = v1.exec(ctx, x.as_ref(), y)?;
                        let n = make_n(&n)?;
                        let y = v2.exec(ctx, x.as_ref(), y)?;
                        do_hatco(ctx, x.as_ref(), &u, &n, &y)
                    })
                }
                _ => {
                    let n = make_n(&ja)?;
                    let u = u.to_verb(ctx.eval())?;
                    BivalentOwned::from_bivalent(move |ctx, x, y| do_hatco(ctx, x, &u, &n, y))
                }
            }
        }
        (VerbNoun::Verb(u), VerbNoun::Verb(v)) => BivalentOwned::from_bivalent(move |ctx, x, y| {
            let u = u.to_verb(ctx.eval())?;
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
    fill_promote_reshape((
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

pub fn c_quote(_ctx: &mut Ctx, u: &VerbNoun, v: &VerbNoun) -> Result<BivalentOwned> {
    let VerbNoun::Noun(n) = v else {
        return Err(JError::DomainError).context("rank conjugation's arg must be a noun");
    };
    let n = n
        .approx()
        .ok_or(JError::DomainError)
        .context("rank expects numeric arguments")?;

    let biv = match u {
        VerbNoun::Verb(u) => {
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
            BivalentOwned::from_bivalent(move |ctx, x, y| {
                let u = u.to_verb(ctx.eval())?;
                match x {
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
                }
            })
        }
        VerbNoun::Noun(u) => {
            if n != arr0d(f32::INFINITY) {
                return Err(JError::NonceError).context("only infinite ranks");
            }
            let u = u.clone();
            BivalentOwned::from_bivalent(move |_ctx, _x, _y| Ok(u.clone()))
        }
    };

    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn c_tie(_ctx: &mut Ctx, u: &Word, v: &Word) -> Result<Word> {
    append_nd(&tie_top(u)?, &tie_top(v)?).map(Word::Noun)
}

// TODO: not quite a copy-paste of Word::boxed_ar
fn tie_top(u: &Word) -> Result<JArray> {
    Ok(match u {
        Word::Noun(u) => u.clone(),
        u @ Word::Verb(_) => JArray::from_list([u.boxed_ar()?]),
        Word::Name(s) => JArray::from_list([JArray::from_string(s)]),
        _ => return Err(JError::DomainError).context("can only gerund nouns and verbs"),
    })
}

pub fn c_agenda(ctx: &mut Ctx, u: &Word, v: &Word) -> Result<Word> {
    use Word::*;

    let Noun(JArray::BoxArray(u)) = u else { return Err(JError::DomainError).context("agenda's box array"); };

    if u.shape().len() > 1 {
        return Err(JError::NonceError).context("@. only implemented for lists");
    }

    match v {
        Noun(v) => do_agenda(ctx, u, v.approx_usize_one().context("agenda's v")?),
        Verb(v) => {
            let u = u.clone();
            let v = v.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                let v = v.exec(ctx, x, y)?;
                match do_agenda(ctx, &u, v.approx_usize_one()?)? {
                    Verb(a) => a.exec(ctx, x, y),
                    _ => Err(JError::DomainError).context("untied a non-verb, which is banned"),
                }
            });
            Ok(Verb(VerbImpl::Partial(PartialImpl {
                imp: BivalentOwned {
                    biv,
                    // supposedly depends on the rank of v
                    ranks: Rank::inf_inf_inf(),
                },
                def: None,
            })))
        }
        _ => Err(JError::DomainError).context("agenda's index type"),
    }
}

fn do_agenda(ctx: &mut Ctx, u: &BoxArray, v: usize) -> Result<Word> {
    let verb = u
        .iter()
        .nth(v)
        .ok_or(JError::IndexError)
        .context("gerund out of bounds")?;

    untie(ctx, verb)
}

fn untie(ctx: &mut Ctx, verb: &JArray) -> Result<Word> {
    Ok(match verb {
        JArray::BoxArray(b) => {
            let op = match &b[0] {
                JArray::CharArray(arr) if arr.shape().len() <= 1 => arr.iter().collect::<String>(),
                _ => {
                    return Err(JError::NonceError)
                        .with_context(|| anyhow!("unrecognised 'op' type in box array: {b:?}"));
                }
            };

            // https://code.jsoftware.com/wiki/Vocabulary/Foreigns#m5
            match op.as_ref() {
                "0" => {
                    if b.shape() != [2] {
                        return Err(JError::DomainError)
                            .with_context(|| anyhow!("noun but non-noun shaped box: {b:?}"));
                    }
                    Word::Noun(b[1].clone())
                }
                "2" => {
                    return Err(JError::NonceError)
                        .with_context(|| anyhow!("unimplemented un-tie op: executed adverb"));
                }
                "3" => {
                    if b.shape() != [2] {
                        return Err(JError::DomainError)
                            .with_context(|| anyhow!("fork but non-fork shaped box: {b:?}"));
                    }
                    let r = &b[1];
                    let r = match r {
                        JArray::BoxArray(b) if b.shape() == [3] => b,
                        _ => {
                            return Err(JError::DomainError)
                                .with_context(|| anyhow!("fork but non-fork shaped r: {r:?}"));
                        }
                    };
                    Word::Verb(VerbImpl::Fork {
                        f: Box::new(untie(ctx, &r[0])?),
                        g: Box::new(untie(ctx, &r[1])?),
                        h: Box::new(untie(ctx, &r[2])?),
                    })
                }
                "4" => {
                    return Err(JError::NonceError)
                        .with_context(|| anyhow!("unimplemented un-tie op: modifier train"));
                }
                _ => match str_to_primitive(&op)? {
                    Some(Word::Conjunction(c)) => {
                        if b.shape() != [2] {
                            return Err(JError::DomainError).with_context(|| {
                                anyhow!("conjunction but non-conjunction shaped box: {b:?}")
                            });
                        }

                        let JArray::BoxArray(r) = &b[1] else {
                            return Err(JError::DomainError).with_context(|| {
                                anyhow!("conjunction argument should be a box, not {:?}", &b[1])
                            });
                        };

                        if r.shape() != [2] {
                            return Err(JError::DomainError).with_context(|| {
                                anyhow!("conjunction but non-conjunction shaped argument: {b:?}")
                            });
                        }

                        let u = untie(ctx, &r[0])?;
                        let v = untie(ctx, &r[1])?;
                        let (farcical, conj) = c.form_conjunction(ctx, &u, &v)?;
                        ensure!(!farcical);
                        conj
                    }
                    other => {
                        return Err(JError::NonceError).with_context(|| {
                            anyhow!("can't un-tie primitive {other:?} for {op:?}")
                        });
                    }
                },
            }
        }
        JArray::CharArray(verb) if verb.shape().len() <= 1 => {
            let name = verb.iter().collect::<String>();
            str_to_primitive(&name)?
                .ok_or(JError::NonceError)
                .with_context(|| anyhow!("unable to un-tie char list {name:?}"))?
        }
        _ => return Err(JError::NonceError).with_context(|| anyhow!("unable to un-tie {verb:?}")),
    })
}

// https://code.jsoftware.com/wiki/Vocabulary/at#/media/File:Funcomp.png
pub fn c_atop(_ctx: &mut Ctx, u: &VerbNoun, v: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    match (u, v) {
        (Verb(u), Verb(v)) => {
            let u = u.clone();
            let v = v.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                let u = u.to_verb(ctx.eval())?;
                let v = v.to_verb(ctx.eval())?;
                do_atop(ctx, x, &u, &v, y)
            });
            Ok(BivalentOwned {
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

    fill_promote_reshape(r).context("expanding result of c_atop")
}

// https://code.jsoftware.com/wiki/Vocabulary/at#/media/File:Funcomp.png
pub fn c_at(_ctx: &mut Ctx, u: &VerbNoun, v: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    match (u, v) {
        (Verb(u), Verb(v)) => {
            let u = u.clone();
            let v = v.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                let r = v.exec(ctx, x, y).context("right half of c_at")?;
                u.exec(ctx, None, &r).context("left half of c_at")
            });
            Ok(BivalentOwned {
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
    let n = match n {
        Word::Noun(n) => n,
        Word::Verb(u) => return Ok((false, c_cor_u(u, m)?)),
        _ => return Err(JError::DomainError).context("cor's n must be verb or noun"),
    };
    let n = n.approx_i64_one()?;
    match m {
        Word::Noun(arr) if arr.approx_i64_one().ok() == Some(0) => {
            Ok((true, Word::Noun(JArray::from(arr0d(n)))))
        }
        Word::Noun(CharArray(jcode)) if jcode.shape().len() <= 1 => {
            let n = match n {
                0 => return Ok((false, Word::Noun(CharArray(jcode.clone())))),
                1 => 'a',
                2 => 'c',
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

pub fn c_cor_u(u: &VerbImpl, v: &Word) -> Result<Word> {
    let Word::Verb(v) = v else {
        return Err(JError::DomainError).context("u:v's v must be a verb");
    };

    let u = u.clone();
    let v = v.clone();

    Ok(Word::Verb(VerbImpl::Partial(PartialImpl {
        imp: BivalentOwned {
            biv: BivalentOwned::from_bivalent(move |ctx, x, y| match x {
                None => u.exec(ctx, None, y),
                Some(x) => v.exec(ctx, Some(x), y),
            }),
            // TODO: ranks should be from u and v, allegedly
            ranks: Rank::inf_inf_inf(),
        },
        def: None,
    })))
}

pub fn c_assign_adverse(_ctx: &mut Ctx, n: &VerbNoun, m: &VerbNoun) -> Result<BivalentOwned> {
    match (n, m) {
        (VerbNoun::Verb(n), VerbNoun::Verb(m)) => {
            let n = n.clone();
            let m = m.clone();
            let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
                n.exec(ctx, x, y).or_else(|_| m.exec(ctx, x, y))
            });
            Ok(BivalentOwned {
                biv,
                ranks: Rank::inf_inf_inf(),
            })
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("\nn: {n:?}\nm: {m:?}")),
    }
}

pub fn c_cut(_ctx: &mut Ctx, n: &VerbNoun, m: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
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

pub fn c_foreign(_ctx: &mut Ctx, l: &VerbNoun, r: &VerbNoun) -> Result<BivalentOwned> {
    match (l, r) {
        (VerbNoun::Noun(l), VerbNoun::Noun(r)) => {
            let l = l.approx_i64_one().context("foreign's left")?;
            let r = r.approx_i64_one().context("foreign's right")?;
            Ok(foreign(l, r)?)
        }
        _ => Err(JError::NonceError).context("unsupported foreign syntax"),
    }
}

pub fn c_bondo(_ctx: &mut Ctx, n: &VerbNoun, m: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;

    // TODO: some of these are presumably obviously monads or dyads
    let biv = match (n.clone(), m.clone()) {
        (Verb(n), Noun(m)) => BivalentOwned::from_bivalent(move |ctx, _x, y| {
            n.exec(ctx, Some(&m), &y.clone()).context("monad bondo VN")
        }),
        (Noun(n), Verb(m)) => BivalentOwned::from_bivalent(move |ctx, _x, y| {
            m.exec(ctx, Some(&n), y).context("monad bondo NV")
        }),
        (Verb(u), Verb(v)) => BivalentOwned::from_bivalent(move |ctx, x, y| {
            let u = u.to_verb(ctx.eval())?;
            let v = v.to_verb(ctx.eval())?;

            match x {
                None => do_atop(ctx, None, &u, &v, &y),
                Some(x) => {
                    let l = v.exec(ctx, None, &x).context("left bondo NVV")?;
                    let r = v.exec(ctx, None, &y).context("right bondo NVV")?;
                    u.exec(ctx, Some(&l), &r).context("central bondo NVV")
                }
            }
        }),
        _ => return Err(JError::NonceError).with_context(|| anyhow!("bondo n:{n:?} m:{m:?}")),
    };
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

pub fn c_under(_ctx: &mut Ctx, u: &VerbNoun, v: &VerbNoun) -> Result<BivalentOwned> {
    use VerbNoun::*;
    let (u, v) = match (u, v) {
        (Verb(u), Verb(v)) => (u, v),
        _ => return Err(JError::NonceError).with_context(|| anyhow!("under dual u:{u:?} v:{v:?}")),
    };
    let u = u.clone();
    let v = v.clone();
    let biv = BivalentOwned::from_bivalent(move |ctx, x, y| {
        let u = u.to_verb(ctx.eval())?;
        let v = v.to_verb(ctx.eval())?;
        let vi = v
            .obverse()
            .ok_or(JError::NonceError)
            .context("lacking obverse")?;
        match x {
            None => do_under_monad(ctx, &u, &v, &vi, y),
            Some(x) => do_under_dyad(ctx, x, u, v, vi, y),
        }
    });
    Ok(BivalentOwned {
        biv,
        ranks: Rank::inf_inf_inf(),
    })
}

fn do_under_monad(
    ctx: &mut Ctx,
    u: &VerbImpl,
    v: &VerbImpl,
    vi: &VerbImpl,
    y: &JArray,
) -> Result<JArray, Error> {
    let (cells, frame) = monad_cells(y, Rank::zero())?;
    let mut parts = Vec::new();
    for y in cells {
        let v = v.exec(ctx, None, &y).context("under dual v")?;
        let u = u.exec(ctx, None, &v).context("under dual u")?;
        let vi = vi.exec(ctx, None, &u).context("under dual vi")?;
        parts.push(vi);
    }
    JArray::from_fill_promote(parts)?
        .reshape(frame)
        .map(|cow| cow.to_owned())
}

fn do_under_dyad(
    ctx: &mut Ctx,
    x: &JArray,
    u: VerbImpl,
    v: VerbImpl,
    vi: VerbImpl,
    y: &JArray,
) -> Result<JArray, Error> {
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
    JArray::from_fill_promote(parts)?.reshape(frame)
}
