use std::fmt;
use std::ops::Deref;

use anyhow::{anyhow, bail, Context, Result};

use super::ranks::Rank;
use crate::arrays::BoxArray;
use crate::cells::{apply_cells, flatten, generate_cells, monad_apply, monad_cells};
use crate::{JArray, JError, Word};

#[derive(Copy, Clone)]
pub struct Monad {
    // TODO: NOT public
    pub f: fn(&JArray) -> Result<JArray>,
    pub rank: Rank,
}

pub type DyadF = fn(&JArray, &JArray) -> Result<JArray>;
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

    // TODO: I didn't even check what J does here
    Null,
}

impl fmt::Display for VerbImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use VerbImpl::*;
        match self {
            Primitive(imp) => write!(f, "{}", imp.name),
            // TODO: remember names of other verbs
            other => write!(f, "{:?}", other),
        }
    }
}

pub fn exec_monad_inner(
    f: impl Fn(&JArray) -> Result<JArray>,
    rank: Rank,
    y: &JArray,
) -> Result<BoxArray> {
    let (cells, common_frame) = monad_cells(y, rank)?;

    let results = monad_apply(&cells, f)?;

    Ok(BoxArray::from_shape_vec(common_frame, results).expect("monad_apply generated"))
}

pub fn exec_monad(f: impl Fn(&JArray) -> Result<JArray>, rank: Rank, y: &JArray) -> Result<JArray> {
    if rank.is_infinite() {
        return f(y).context("infinite monad shortcut");
    }

    let r = exec_monad_inner(f, rank, y)?;
    flatten(&r)
}

pub fn exec_dyad_inner(
    f: impl Fn(&JArray, &JArray) -> Result<JArray>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<BoxArray> {
    let (frames, cells) = generate_cells(x.clone(), y.clone(), rank).context("generating cells")?;

    let application_result = apply_cells(&cells, f, rank).context("applying function to cells")?;
    Ok(BoxArray::from_shape_vec(frames, application_result).expect("apply_cells generated shape"))
}

pub fn exec_dyad(
    f: impl Fn(&JArray, &JArray) -> Result<JArray>,
    rank: DyadRank,
    x: &JArray,
    y: &JArray,
) -> Result<JArray> {
    if Rank::infinite_infinite() == rank {
        return (f)(x, y).context("infinite dyad shortcut");
    }

    let r = exec_dyad_inner(f, rank, x, y)?;
    flatten(&r)
}

impl VerbImpl {
    pub fn exec(&self, x: Option<&Word>, y: &Word) -> Result<JArray> {
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
                    exec_dyad(dyad.f, dyad.rank, x, y)
                        .with_context(|| anyhow!("dyadic {:?}", imp.name))
                }
                other => Err(JError::DomainError)
                    .with_context(|| anyhow!("primitive on non-nouns: {other:#?}")),
            },
            VerbImpl::DerivedVerb { l, r, m } => match (l.deref(), r.deref(), m.deref()) {
                (u @ Verb(_, _), Nothing, Adverb(_, a)) => {
                    a.exec(x, u, &Nothing, y).and_then(must_be_noun)
                }
                (m @ Noun(_), Nothing, Adverb(_, a)) => {
                    a.exec(x, m, &Nothing, y).and_then(must_be_noun)
                }
                (l, r, Conjunction(_, c))
                    if matches!(l, Noun(_) | Verb(_, _)) && matches!(r, Noun(_) | Verb(_, _)) =>
                {
                    c.exec(x, l, r, y).and_then(must_be_noun)
                }
                _ => panic!("invalid DerivedVerb {:?}", self),
            },
            VerbImpl::Fork { f, g, h } => match (f.deref(), g.deref(), h.deref()) {
                (Verb(_, f), Verb(_, g), Verb(_, h)) => {
                    log::warn!("Fork {:?} {:?} {:?}", f, g, h);
                    log::warn!("{:?} {:?} {:?}:\n{:?}", x, f, y, f.exec(x, y));
                    log::warn!("{:?} {:?} {:?}:\n{:?}", x, h, y, h.exec(x, y));
                    g.exec(
                        Some(&f.exec(x, y).map(Word::Noun)?),
                        &h.exec(x, y).map(Word::Noun)?,
                    )
                }
                (Noun(m), Verb(_, g), Verb(_, h)) => {
                    g.exec(Some(&Noun(m.clone())), &h.exec(x, y).map(Word::Noun)?)
                }
                _ => panic!("invalid Fork {:?}", self),
            },
            VerbImpl::Hook { l, r } => match (l.deref(), r.deref()) {
                (Verb(_, u), Verb(_, v)) => match x {
                    None => u.exec(Some(y), &v.exec(None, y).map(Word::Noun)?),
                    Some(x) => u.exec(Some(x), &v.exec(None, y).map(Word::Noun)?),
                },
                _ => panic!("invalid Hook {:?}", self),
            },
            VerbImpl::Null => bail!("please don't try and execute nothing"),
        }
    }

    /// The dyad rank, if this is a dyad.
    // TODO: presumably this is implementable for derived verbs
    pub fn dyad_rank(&self) -> Option<DyadRank> {
        match self {
            Self::Primitive(p) => p.dyad.map(|d| d.rank),
            _ => None,
        }
    }
}

fn must_be_noun(v: Word) -> Result<JArray> {
    match v {
        Word::Noun(arr) => Ok(arr),
        _ => Err(JError::DomainError).context("unexpected non-noun in noun context"),
    }
}

impl PrimitiveImpl {
    pub fn monad(name: &'static str, f: fn(&JArray) -> Result<JArray>) -> Self {
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
        monad: fn(&JArray) -> Result<JArray>,
        dyad: fn(&JArray, &JArray) -> Result<JArray>,
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
