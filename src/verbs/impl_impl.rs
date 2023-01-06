use std::ops::Deref;

use anyhow::{anyhow, Context, Result};

use super::ranks::Rank;
use crate::cells::{apply_cells, fill_promote_reshape, generate_cells, monad_apply, monad_cells};
use crate::verbs::partial::PartialImpl;
use crate::verbs::primitive::PrimitiveImpl;
use crate::verbs::DyadRank;
use crate::{primitive_verbs, Ctx, JArray, JError, Num, Word};

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
    fill_promote_reshape(&r)
}

pub fn exec_dyad_inner(
    f: impl FnMut(&JArray, &JArray) -> Result<JArray>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<VerbResult> {
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
    if Rank::infinite_infinite() == rank {
        return (f)(x, y).context("infinite dyad shortcut");
    }

    let r = exec_dyad_inner(f, rank, x, y)?;
    fill_promote_reshape(&r)
}

impl VerbImpl {
    pub fn exec(&self, ctx: &mut Ctx, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
        fill_promote_reshape(&self.partial_exec(ctx, x, y)?)
    }

    pub fn partial_exec(
        &self,
        ctx: &mut Ctx,
        x: Option<&JArray>,
        y: &JArray,
    ) -> Result<VerbResult> {
        use Word::*;
        match self {
            VerbImpl::Primitive(imp) => match x {
                None => exec_monad_inner(imp.monad.f, imp.monad.rank, y)
                    .with_context(|| anyhow!("y: {y:?}"))
                    .with_context(|| anyhow!("monadic {:?}", imp.name)),
                Some(x) => {
                    let dyad = imp
                        .dyad
                        .ok_or(JError::DomainError)
                        .with_context(|| anyhow!("there is no dyadic {:?}", imp.name))?;
                    exec_dyad_inner(dyad.f, dyad.rank, x, y)
                        .with_context(|| anyhow!("x: {x:?}"))
                        .with_context(|| anyhow!("y: {y:?}"))
                        .with_context(|| anyhow!("dyadic {:?}", imp.name))
                }
            },
            VerbImpl::Partial(imp) => {
                let biv = imp
                    .biv
                    .as_ref()
                    .ok_or(JError::SyntaxError)
                    .context("always available")?;
                match x {
                    None => exec_monad_inner(|y| biv(ctx, None, y), imp.ranks.0, y)
                        .with_context(|| anyhow!("y: {y:?}"))
                        .with_context(|| anyhow!("monadic partial {:?}", imp.name)),
                    Some(x) => exec_dyad_inner(|x, y| biv(ctx, Some(x), y), imp.ranks.1, x, y)
                        .with_context(|| anyhow!("x: {x:?}"))
                        .with_context(|| anyhow!("y: {y:?}"))
                        .with_context(|| anyhow!("dyadic partial {:?}", imp.name)),
                }
            }
            VerbImpl::Fork { f, g, h } => match (f.deref(), g.deref(), h.deref()) {
                (Verb(f), Verb(g), Verb(h)) => {
                    log::debug!("Fork {:?} {:?} {:?}", f, g, h);
                    log::debug!("{:?} {:?} {:?}:\n{:?}", x, f, y, f.exec(ctx, x, y));
                    log::debug!("{:?} {:?} {:?}:\n{:?}", x, h, y, h.exec(ctx, x, y));
                    let f = match f {
                        VerbImpl::Cap => None,
                        _ => Some(f.exec(ctx, x, y).context("fork impl (f)")?),
                    };
                    // TODO: it's very unclear to me that this should be a recursive call,
                    // TODO: and not exec() with some mapping like elsewhere
                    let ny = h.exec(ctx, x, y).context("fork impl (h)")?;
                    g.partial_exec(ctx, f.as_ref(), &ny)
                        .context("fork impl (g)")
                }
                (Noun(m), Verb(g), Verb(h)) => {
                    // TODO: it's very unclear to me that this should be a recursive call,
                    // TODO: and not exec() with some mapping like elsewhere
                    let ny = h.exec(ctx, x, y)?;
                    g.partial_exec(ctx, Some(m), &ny)
                }
                _ => panic!("invalid Fork {:?}", self),
            },
            VerbImpl::Hook { l, r } => match (l.deref(), r.deref()) {
                (Verb(u), Verb(v)) => {
                    let ny = v.exec(ctx, None, y)?;
                    match x {
                        // TODO: it's very unclear to me that this should be a recursive call,
                        // TODO: and not exec() with some mapping like elsewhere
                        None => u.partial_exec(ctx, Some(y), &ny),
                        Some(x) => u.partial_exec(ctx, Some(x), &ny),
                    }
                }
                _ => panic!("invalid Hook {:?}", self),
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
            Self::Partial(p) => p.monad.as_ref().map(|p| p.rank),
            _ => None,
        }
    }
    /// The dyad rank, if this is a dyad.
    // TODO: presumably this is implementable for derived verbs
    pub fn dyad_rank(&self) -> Option<DyadRank> {
        match self {
            Self::Primitive(p) => p.dyad.map(|d| d.rank),
            Self::Partial(p) => p.dyad.as_ref().map(|d| d.rank),
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
        None
    }
}
