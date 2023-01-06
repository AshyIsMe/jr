use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JArray, JError};

use super::ranks::{DyadRank, Rank};

pub type BivalentOwnedF = Arc<dyn Fn(&mut Ctx, Option<&JArray>, &JArray) -> Result<JArray>>;

#[derive(Clone)]
pub struct PartialImpl {
    pub imp: BivalentOwned,
}

#[derive(Clone)]
pub struct BivalentOwned {
    pub name: String,
    pub biv: BivalentOwnedF,
    pub ranks: (Rank, DyadRank),
}

impl PartialImpl {
    pub fn name(&self) -> Result<String> {
        Err(JError::NonceError).with_context(|| anyhow!("no name for partial {self:?}"))
    }
}

impl fmt::Debug for PartialImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PartialImpl({:?})", self.name())
    }
}

impl PartialEq for PartialImpl {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl BivalentOwned {
    pub fn from_bivalent(
        f: impl Fn(&mut Ctx, Option<&JArray>, &JArray) -> Result<JArray> + 'static + Clone,
    ) -> BivalentOwnedF {
        Arc::new(move |ctx, x, y| f(ctx, x, y))
    }

    pub fn from_monad(
        f: impl Fn(&mut Ctx, &JArray) -> Result<JArray> + 'static + Clone,
    ) -> BivalentOwnedF {
        Arc::new(move |ctx, x, y| {
            ensure_monad(x)?;
            f(ctx, y)
        })
    }
}

fn ensure_monad(x: Option<&JArray>) -> Result<()> {
    match x {
        Some(_) => Err(JError::DomainError).context("dyadic invocation of monad-only verb"),
        None => Ok(()),
    }
}
