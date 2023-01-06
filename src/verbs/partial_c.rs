use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};

use crate::{Ctx, JError, Word};

use super::ranks::{DyadRank, Rank};

pub type BivalentCOwnedF = Arc<dyn Fn(&mut Ctx, Option<&Word>, &Word) -> Result<Word>>;

#[derive(Clone)]
pub struct PartialCImpl {
    pub imp: BivalentCOwned,
    pub def: Option<Vec<Word>>,
}

#[derive(Clone)]
pub struct BivalentCOwned {
    pub biv: BivalentCOwnedF,
    pub ranks: (Rank, DyadRank),
}

impl PartialCImpl {
    pub fn name(&self) -> Result<String> {
        Err(JError::NonceError).with_context(|| anyhow!("no name for partial {self:?}"))
    }
}

impl fmt::Debug for PartialCImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PartialCImpl")
    }
}

impl PartialEq for PartialCImpl {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl BivalentCOwned {
    pub fn from_bivalent(
        f: impl Fn(&mut Ctx, Option<&Word>, &Word) -> Result<Word> + 'static + Clone,
    ) -> BivalentCOwnedF {
        Arc::new(move |ctx, x, y| f(ctx, x, y))
    }
}
