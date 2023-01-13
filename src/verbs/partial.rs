use std::fmt;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::eval::VerbNoun;
use crate::modifiers::ModifierImpl;
use crate::{Ctx, JArray, JError, Word};

use super::ranks::{DyadRank, Rank};

pub type BivalentOwnedF = Arc<dyn Fn(&mut Ctx, Option<&JArray>, &JArray) -> Result<JArray>>;

#[derive(Clone)]
pub struct PartialImpl {
    pub imp: BivalentOwned,
    pub def: Box<PartialDef>,
}

#[derive(Clone, Debug)]
pub enum PartialDef {
    Adverb(ModifierImpl, VerbNoun),
    Conjunction(VerbNoun, ModifierImpl, VerbNoun),
    Cor(i64, Vec<Word>),
    Unimplemented(&'static str),
}

#[derive(Clone)]
pub struct BivalentOwned {
    pub biv: BivalentOwnedF,
    pub ranks: (Rank, DyadRank),
}

impl PartialImpl {
    pub fn name(&self) -> String {
        match self.def.as_ref() {
            _ => format!("[TODO: no name for partial {:?}]", self.def),
        }
    }
}

impl fmt::Debug for PartialImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PartialImpl({:?})", self.def)
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
