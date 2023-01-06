use std::fmt;
use std::sync::Arc;

use anyhow::Result;

use crate::{Ctx, JArray};

use super::ranks::{DyadRank, Rank};

pub type MonadOwnedF = Arc<dyn Fn(&mut Ctx, &JArray) -> Result<JArray>>;

#[derive(Clone)]
pub struct MonadOwned {
    pub f: MonadOwnedF,
    pub rank: Rank,
}

pub type DyadOwnedF = Arc<dyn Fn(&mut Ctx, &JArray, &JArray) -> Result<JArray>>;

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
    ) -> (Option<MonadOwned>, Option<DyadOwned>) {
        let j = f.clone();
        (
            Self::mi(Arc::new(move |ctx, y| f(ctx, None, y))),
            Self::di(Arc::new(move |ctx, x, y| j(ctx, Some(x), y))),
        )
    }

    pub fn mi(f: MonadOwnedF) -> Option<MonadOwned> {
        Some(MonadOwned {
            f,
            rank: Rank::infinite(),
        })
    }

    pub fn m0(f: MonadOwnedF) -> Option<MonadOwned> {
        Some(MonadOwned {
            f,
            rank: Rank::zero(),
        })
    }

    pub fn di(f: DyadOwnedF) -> Option<DyadOwned> {
        Some(DyadOwned {
            f,
            rank: Rank::infinite_infinite(),
        })
    }
}
