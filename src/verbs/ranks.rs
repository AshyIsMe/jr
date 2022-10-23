use anyhow::{anyhow, Context, Result};

use crate::JError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rank(u8);

impl Rank {
    pub fn new(val: u8) -> Result<Self> {
        if val >= 64 {
            return Err(JError::LimitError).with_context(|| anyhow!("{val} is too many ranks"));
        }
        Ok(Rank(val))
    }

    pub const fn zero() -> Self {
        Rank(0)
    }

    pub const fn one() -> Self {
        Rank(1)
    }

    pub const fn zero_zero() -> (Self, Self) {
        (Self::zero(), Self::zero())
    }

    pub const fn infinite() -> Self {
        Rank(u8::MAX)
    }

    pub const fn infinite_infinite() -> (Self, Self) {
        (Self::infinite(), Self::infinite())
    }

    pub fn usize(&self) -> usize {
        usize::from(self.0)
    }
}
