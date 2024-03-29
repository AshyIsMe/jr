use std::ops::Deref;

use anyhow::{anyhow, bail, Context, Result};

use super::ranks::Rank;
use crate::cells::{apply_cells, fill_promote_reshape, generate_cells, monad_apply, monad_cells};
use crate::number::float_is_int;
use crate::verbs::primitive::PrimitiveImpl;
use crate::verbs::{DyadRank, PartialDef, PartialImpl};
use crate::{arr0ad, primitive_verbs, Ctx, JArray, JError, Num, Word};

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Primitive(PrimitiveImpl),

    Partial(PartialImpl),

    Fork {
        f: Box<Word>,
        g: Box<Word>,
        h: Box<Word>,
    },
    Hook {
        l: Box<Word>,
        r: Box<Word>,
    },
    Cap,

    Number(f64),
}

pub type VerbResult = (Vec<usize>, Vec<JArray>);

fn exec_monad_inner(
    f: impl FnMut(&JArray) -> Result<JArray>,
    rank: Rank,
    y: &JArray,
) -> Result<VerbResult> {
    let (cells, frames) = monad_cells(y, rank)?;

    let application_results = monad_apply(&cells, f)?;

    Ok((frames, application_results))
}

pub fn exec_monad(
    mut f: impl FnMut(&JArray) -> Result<JArray>,
    rank: Rank,
    y: &JArray,
) -> Result<JArray> {
    if rank.is_infinite() {
        return f(y).context("infinite monad shortcut");
    }

    let r = exec_monad_inner(f, rank, y)?;
    fill_promote_reshape(r)
}

pub fn exec_dyad_inner(
    mut f: impl FnMut(&JArray, &JArray) -> Result<JArray>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<VerbResult> {
    if x.shape().is_empty() && y.shape().is_empty() {
        return Ok((Vec::new(), vec![f(x, y)?]));
    }

    let (frames, cells) = generate_cells(x.clone(), y.clone(), rank).context("generating cells")?;

    let application_result = apply_cells(&cells, f, rank).context("applying function to cells")?;

    Ok((frames, application_result))
}

pub fn exec_dyad(
    mut f: impl FnMut(&JArray, &JArray) -> Result<JArray>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<JArray> {
    if rank.0.is_infinite() && rank.1.is_infinite() {
        return (f)(x, y).context("infinite dyad shortcut");
    }

    let r = exec_dyad_inner(f, rank, x, y)?;
    fill_promote_reshape(r)
}

impl VerbImpl {
    pub fn exec(&self, ctx: &mut Ctx, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
        fill_promote_reshape(self.partial_exec(ctx, x, y)?)
    }

    pub fn partial_exec(
        &self,
        ctx: &mut Ctx,
        x: Option<&JArray>,
        y: &JArray,
    ) -> Result<VerbResult> {
        use Word::*;
        match self {
            VerbImpl::Primitive(imp) => partial_exec_primitive(imp, x, y),
            VerbImpl::Partial(p) => {
                let biv = &p.imp.biv;
                match x {
                    None => exec_monad_inner(|y| biv(ctx, None, y), p.imp.ranks.0, y)
                        .with_context(|| anyhow!("y: {y:?}"))
                        .with_context(|| anyhow!("monadic partial {:?}", p.name())),
                    Some(x) => exec_dyad_inner(|x, y| biv(ctx, Some(x), y), p.imp.ranks.1, x, y)
                        .with_context(|| anyhow!("x: {x:?}"))
                        .with_context(|| anyhow!("y: {y:?}"))
                        .with_context(|| anyhow!("dyadic partial: {}", p.name())),
                }
            }
            VerbImpl::Fork { f, g, h } => match (f.deref(), g.deref(), h.deref()) {
                (Verb(f), Verb(g), Verb(h)) => {
                    log::debug!("Fork {:?} {:?} {:?}", f, g, h);
                    log::debug!("{:?} {:?} {:?}:\n{:?}", x, f, y, f.exec(ctx, x, y));
                    log::debug!("{:?} {:?} {:?}:\n{:?}", x, h, y, h.exec(ctx, x, y));
                    match (f, h) {
                        (VerbImpl::Primitive(f), VerbImpl::Primitive(h)) => {
                            // f and h are primitives so execute in parallel (no global assignments to worry about)
                            crossbeam::scope(|s| {
                                let thread_l =
                                    s.spawn(|_| exec_primitive(f, x, y).context("fork impl (f)"));
                                let thread_r =
                                    s.spawn(|_| exec_primitive(h, x, y).context("fork impl (h)"));

                                let nx = thread_l.join().expect("thread_l panic")?;
                                let ny = thread_r.join().expect("thread_r panic")?;
                                g.partial_exec(ctx, Some(&nx), &ny).context("fork impl (g)")
                            })
                            .unwrap()
                        }
                        _ => {
                            let f = match f {
                                VerbImpl::Cap => None,
                                _ => Some(f.exec(ctx, x, y).context("fork impl (f)").unwrap()),
                            };
                            let ny = h.exec(ctx, x, y).context("fork impl (h)").unwrap();

                            g.partial_exec(ctx, f.as_ref(), &ny)
                                .context("fork impl (g)")
                        }
                    }
                }
                (Noun(m), Verb(g), Verb(h)) => {
                    // TODO: it's very unclear to me that this should be a recursive call,
                    // TODO: and not exec() with some mapping like elsewhere
                    let ny = h.exec(ctx, x, y)?;
                    g.partial_exec(ctx, Some(m), &ny)
                }
                _ => panic!("invalid Fork {:?}", self),
            },
            VerbImpl::Hook { l, r } => match (l.when_verb(), r.when_verb()) {
                (Some(u), Some(v)) => {
                    let u = u.to_verb(ctx.eval())?;
                    let v = v.to_verb(ctx.eval())?;
                    let ny = v.exec(ctx, None, y)?;
                    match x {
                        // TODO: it's very unclear to me that this should be a recursive call,
                        // TODO: and not exec() with some mapping like elsewhere
                        None => u.partial_exec(ctx, Some(y), &ny),
                        Some(x) => u.partial_exec(ctx, Some(x), &ny),
                    }
                }
                _ => bail!("supposedly unreachable: invalid Hook {:?}", self),
            },
            VerbImpl::Cap => Err(JError::DomainError)
                .with_context(|| anyhow!("cap cannot be executed: {x:?} {y:?}")),
            VerbImpl::Number(i) => Ok((Vec::new(), vec![JArray::from(Num::Float(*i).demote())])),
        }
    }

    // TODO: presumably this is implementable for derived verbs
    pub fn monad_rank(&self) -> Option<Rank> {
        match self {
            Self::Primitive(p) => Some(p.monad.rank),
            Self::Partial(PartialImpl { imp, .. }) => Some(imp.ranks.0),
            _ => None,
        }
    }
    /// The dyad rank, if this is a dyad.
    // TODO: presumably this is implementable for derived verbs
    pub fn dyad_rank(&self) -> Option<DyadRank> {
        match self {
            Self::Primitive(p) => Some(p.dyad.rank),
            Self::Partial(PartialImpl { imp, .. }) => Some(imp.ranks.1),
            _ => None,
        }
    }

    pub fn obverse(&self) -> Option<VerbImpl> {
        match self {
            VerbImpl::Primitive(imp) => imp.inverse.and_then(primitive_verbs),
            _ => None,
        }
    }

    pub fn token(&self) -> Option<&str> {
        Some(match self {
            VerbImpl::Primitive(imp) => imp.name,
            _ => return None,
        })
    }

    pub fn name(&self) -> String {
        use VerbImpl::*;
        match self {
            Primitive(p) => p.name.to_string(),
            Partial(p) => p.name(),
            Fork { f, g, h } => format!("({} {} {})", f.name(), g.name(), h.name()),
            Hook { l, r } => format!("({} {})", l.name(), r.name()),
            Cap => "[:".to_string(),
            Number(i) => {
                if i < &0.0 {
                    format!("(_{}:)", i.abs())
                } else {
                    format!("({i}:)")
                }
            }
        }
    }

    pub fn boxed_ar(&self) -> Result<JArray> {
        // https://code.jsoftware.com/wiki/Vocabulary/Foreigns#m5
        use VerbImpl::*;
        Ok(match self {
            Primitive(imp) => JArray::from_string(imp.name),
            // TODO: invalid for inf, negatives
            Number(i) => JArray::from_string(match float_is_int(*i) {
                Some(i) if i >= 0 => format!("{i}:"),
                Some(i) => format!("_{i}:"),
                None => return Err(JError::NonceError).context("super lazy about infinity"),
            }),
            Partial(PartialImpl { def, .. }) => match &**def {
                PartialDef::Adverb(a, u) => {
                    JArray::from_list(vec![a.boxed_ar()?, JArray::from_list(vec![u.boxed_ar()?])])
                }
                PartialDef::Conjunction(u, c, v) => JArray::from_list(vec![
                    c.boxed_ar()?,
                    JArray::from_list(vec![u.boxed_ar()?, v.boxed_ar()?]),
                ]),
                PartialDef::Cor(n, def) => JArray::from_list([
                    JArray::from_string(":"),
                    JArray::from_list([
                        Word::Noun(JArray::IntArray(arr0ad(*n))).boxed_ar()?,
                        Word::Noun(JArray::from_string(stringify(def)).rank_extend(2))
                            .boxed_ar()?,
                    ]),
                ]),
            },
            Hook { l, r } => JArray::from_list([
                JArray::from_string("2"),
                JArray::from_list([l.boxed_ar()?, r.boxed_ar()?]),
            ]),
            Fork { f, g, h } => JArray::from_list([
                JArray::from_string("3"),
                JArray::from_list([f.boxed_ar()?, g.boxed_ar()?, h.boxed_ar()?]),
            ]),
            _ => {
                return Err(JError::NonceError)
                    .with_context(|| anyhow!("can't VerbImpl::boxed_ar {self:?}"))
            }
        })
    }
}

pub fn stringify(def: &[Word]) -> String {
    let mut ret = String::with_capacity(4 * def.len());
    for word in def {
        ret.push_str(&word.name());
        ret.push_str(" ");
    }
    ret
}

fn exec_primitive(imp: &PrimitiveImpl, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
    fill_promote_reshape(partial_exec_primitive(imp, x, y)?)
}

fn partial_exec_primitive(
    imp: &PrimitiveImpl,
    x: Option<&JArray>,
    y: &JArray,
) -> Result<VerbResult> {
    match x {
        None => exec_monad_inner(imp.monad.f, imp.monad.rank, y)
            .with_context(|| anyhow!("y: {y:?}"))
            .with_context(|| anyhow!("monadic {:?}", imp.name)),
        Some(x) => exec_dyad_inner(imp.dyad.f, imp.dyad.rank, x, y)
            .with_context(|| anyhow!("x: {x:?}"))
            .with_context(|| anyhow!("y: {y:?}"))
            .with_context(|| anyhow!("dyadic {:?}", imp.name)),
    }
}
