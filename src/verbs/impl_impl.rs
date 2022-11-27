use std::fmt;
use std::ops::Deref;

use anyhow::{anyhow, bail, Context, Result};
use log::debug;

use super::ranks::Rank;
use crate::cells::{apply_cells, flatten, generate_cells, monad_apply, monad_cells};
use crate::{JArray, JError, Word};

#[derive(Copy, Clone)]
pub struct Monad {
    // TODO: NOT public
    pub f: fn(&JArray) -> Result<Word>,
    pub rank: Rank,
}

pub type DyadF = fn(&JArray, &JArray) -> Result<Word>;
pub type DyadRank = (Rank, Rank);

#[derive(Copy, Clone)]
pub struct Dyad {
    pub f: DyadF,
    pub rank: DyadRank,
}

#[derive(Copy, Clone)]
pub struct PrimitiveImpl {
    // TODO: NOT public
    pub name: &'static str,
    // TODO: NOT public
    pub monad: Monad,
    // TODO: NOT public
    pub dyad: Option<Dyad>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Primitive(PrimitiveImpl),

    //Adverb or Conjunction modified Verb eg. +/ or u^:n etc.
    //Modifiers take a left and right argument refered to as either
    //u and v if verbs or m and n if nouns (or combinations of either).
    DerivedVerb {
        l: Box<Word>,
        r: Box<Word>,
        m: Box<Word>,
    },
    Fork {
        f: Box<Word>,
        g: Box<Word>,
        h: Box<Word>,
    },
    Hook {
        l: Box<Word>,
        r: Box<Word>,
    },
}

pub fn exec_monad(f: impl Fn(&JArray) -> Result<Word>, rank: Rank, y: &JArray) -> Result<Word> {
    if rank.is_infinite() {
        return f(y).context("infinite monad shortcut");
    }

    let (cells, common_frame) = monad_cells(y, rank)?;

    let results = monad_apply(&cells, |y| {
        Ok(match f(y)? {
            Word::Noun(arr) => arr,
            other => bail!("not handling non-noun outputs {other:?}"),
        })
    })?;

    let results = flatten(&common_frame, &[], &[results])?;

    Ok(Word::Noun(results))
}

pub fn exec_dyad(
    f: impl Fn(&JArray, &JArray) -> Result<Word>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<Word> {
    if Rank::infinite_infinite() == rank {
        return (f)(x, y).context("infinite dyad shortcut");
    }
    let (cells, common_frame, surplus_frame) =
        generate_cells(x.clone(), y.clone(), rank).context("generating cells")?;

    let application_result = apply_cells(&cells, f, rank).context("applying function to cells")?;
    debug!("application_result: {:?}", application_result);

    let flat = flatten(&common_frame, &surplus_frame, &application_result)?;

    Ok(Word::Noun(flat))
}

impl VerbImpl {
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word> {
        self.exec_ranked(x, y, None)
    }
    pub fn exec_ranked(
        &self,
        x: Option<&Word>,
        y: &Word,
        rank: Option<(Rank, Rank, Rank)>,
    ) -> Result<Word> {
        use Word::*;
        match self {
            VerbImpl::Primitive(imp) => match (x, y) {
                (None, Noun(y)) => exec_monad(imp.monad.f, imp.monad.rank, y)
                    .with_context(|| anyhow!("monadic {:?}", imp.name)),
                (Some(Noun(x)), Noun(y)) => {
                    let dyad = imp
                        .dyad
                        .ok_or(JError::DomainError)
                        .with_context(|| anyhow!("there is no dyadic {:?}", imp.name))?;
                    exec_dyad(dyad.f, rank.map(|r| (r.1, r.2)).unwrap_or(dyad.rank), x, y)
                        .with_context(|| anyhow!("dyadic {:?}", imp.name))
                }
                other => Err(JError::DomainError)
                    .with_context(|| anyhow!("primitive on non-nouns: {other:#?}")),
            },
            VerbImpl::DerivedVerb { l, r, m } => match (l.deref(), r.deref(), m.deref()) {
                (u @ Verb(_, _), Nothing, Adverb(_, a)) => a.exec(x, u, &Nothing, y),
                (m @ Noun(_), Nothing, Adverb(_, a)) => a.exec(x, m, &Nothing, y),
                (l, r, Conjunction(_, c))
                    if matches!(l, Noun(_) | Verb(_, _)) && matches!(r, Noun(_) | Verb(_, _)) =>
                {
                    c.exec(x, l, r, y)
                }
                _ => panic!("invalid DerivedVerb {:?}", self),
            },
            VerbImpl::Fork { f, g, h } => match (f.deref(), g.deref(), h.deref()) {
                (Verb(_, f), Verb(_, g), Verb(_, h)) => {
                    log::warn!("Fork {:?} {:?} {:?}", f, g, h);
                    log::warn!("{:?} {:?} {:?}:\n{:?}", x, f, y, f.exec(x, y));
                    log::warn!("{:?} {:?} {:?}:\n{:?}", x, h, y, h.exec(x, y));
                    g.exec(Some(&f.exec(x, y)?), &h.exec(x, y)?)
                }
                (Noun(m), Verb(_, g), Verb(_, h)) => g.exec(Some(&Noun(m.clone())), &h.exec(x, y)?),
                _ => panic!("invalid Fork {:?}", self),
            },
            VerbImpl::Hook { l, r } => match (l.deref(), r.deref()) {
                (Verb(_, u), Verb(_, v)) => match x {
                    None => u.exec(Some(y), &v.exec(None, y)?),
                    Some(x) => u.exec(Some(x), &v.exec(None, y)?),
                },
                _ => panic!("invalid Hook {:?}", self),
            },
        }
    }
}

impl PrimitiveImpl {
    pub fn monad(name: &'static str, f: fn(&JArray) -> Result<Word>) -> Self {
        Self {
            name,
            monad: Monad {
                f,
                rank: Rank::infinite(),
            },
            dyad: None,
        }
    }

    pub const fn new(
        name: &'static str,
        monad: fn(&JArray) -> Result<Word>,
        dyad: fn(&JArray, &JArray) -> Result<Word>,
        ranks: (Rank, Rank, Rank),
    ) -> Self {
        Self {
            name,
            monad: Monad {
                f: monad,
                rank: ranks.0,
            },
            dyad: Some(Dyad {
                f: dyad,
                rank: (ranks.1, ranks.2),
            }),
        }
    }
}

impl fmt::Debug for PrimitiveImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PrimitiveImpl({})", self.name)
    }
}

impl PartialEq for PrimitiveImpl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
