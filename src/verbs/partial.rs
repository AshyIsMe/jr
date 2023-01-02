use std::fmt;
use std::sync::Arc;

use anyhow::Result;

use crate::{Ctx, JArray, Word};

use super::ranks::{DyadRank, Rank};

#[derive(Clone)]
pub struct MonadOwned {
    pub f: Arc<dyn Fn(&mut Ctx, &JArray) -> Result<Word>>,
    pub rank: Rank,
}

#[derive(Clone)]
pub struct DyadOwned {
    pub f: Arc<dyn Fn(&mut Ctx, &JArray, &JArray) -> Result<Word>>,
    pub rank: DyadRank,
}

#[derive(Clone)]
pub struct PartialImpl {
    pub name: String,
    pub monad: Option<MonadOwned>,
    pub dyad: Option<DyadOwned>,
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
