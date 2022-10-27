use anyhow::{anyhow, Context, Result};

use crate::JError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rank(u8);

impl Rank {
    pub fn new(val: u32) -> Result<Self> {
        if val == u32::MAX {
            Ok(Rank(u8::MAX))
        } else if val >= 64 {
            return Err(JError::LimitError).with_context(|| anyhow!("{val} is too many ranks"));
        } else {
            Ok(Rank(val as u8))
        }
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

    pub const fn is_infinite(&self) -> bool {
        self.0 == u8::MAX
    }

    pub const fn is_one(&self) -> bool {
        self.0 == 1
    }

    pub fn usize(&self) -> Option<usize> {
        if self.is_infinite() {
            return None;
        }
        Some(usize::from(self.0))
    }
}
