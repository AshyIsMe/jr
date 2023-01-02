use std::fmt;

use anyhow::Result;

use crate::JArray;

use super::ranks::{DyadRank, Rank};

#[derive(Copy, Clone)]
pub struct Monad {
    pub f: fn(&JArray) -> Result<JArray>,
    pub rank: Rank,
}

#[derive(Copy, Clone)]
pub struct Dyad {
    pub f: fn(&JArray, &JArray) -> Result<JArray>,
    pub rank: DyadRank,
}

#[derive(Copy, Clone)]
pub struct PrimitiveImpl {
    pub name: &'static str,
    pub monad: Monad,
    pub dyad: Option<Dyad>,
    pub inverse: Option<&'static str>,
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
            inverse: None,
        }
    }

    pub const fn new(
        name: &'static str,
        monad: fn(&JArray) -> Result<JArray>,
        dyad: fn(&JArray, &JArray) -> Result<JArray>,
        ranks: (Rank, Rank, Rank),
        inverse: Option<&'static str>,
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
            inverse,
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
