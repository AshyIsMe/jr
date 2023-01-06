use std::fmt;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::{Ctx, JArray, JError};

use super::ranks::{DyadRank, Rank};

pub type MonadOwnedF = Arc<dyn Fn(&mut Ctx, &JArray) -> Result<JArray>>;

#[derive(Clone)]
pub struct MonadOwned {
    pub f: MonadOwnedF,
    pub rank: Rank,
}

pub type DyadOwnedF = Arc<dyn Fn(&mut Ctx, &JArray, &JArray) -> Result<JArray>>;
pub type BivalentOwnedF = Arc<dyn Fn(&mut Ctx, Option<&JArray>, &JArray) -> Result<JArray>>;

#[derive(Clone)]
pub struct DyadOwned {
    pub f: DyadOwnedF,
    pub rank: DyadRank,
}

#[derive(Clone)]
pub struct PartialImpl {
    pub name: String,
    pub monad: Option<MonadOwned>,
    pub dyad: Option<DyadOwned>,
    pub ranks: (Rank, DyadRank),
    pub biv: Option<BivalentOwnedF>,
}

impl fmt::Debug for PartialImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PartialImpl({})", self.name)
    }
}

impl PartialEq for PartialImpl {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl PartialImpl {
    pub fn from_legacy_inf(
        f: impl Fn(&mut Ctx, Option<&JArray>, &JArray) -> Result<JArray> + 'static + Clone,
    ) -> Option<BivalentOwnedF> {
        Some(Arc::new(move |ctx, x, y| f(ctx, x, y)))
    }

    pub fn from_monad(
        f: impl Fn(&mut Ctx, &JArray) -> Result<JArray> + 'static + Clone,
    ) -> Option<BivalentOwnedF> {
        Some(Arc::new(move |ctx, x, y| {
            ensure_monad(x)?;
            f(ctx, y)
        }))
    }
}

fn ensure_monad(x: Option<&JArray>) -> Result<()> {
    match x {
        Some(_) => Err(JError::DomainError).context("dyadic invocation of monad-only verb"),
        None => Ok(()),
    }
}
