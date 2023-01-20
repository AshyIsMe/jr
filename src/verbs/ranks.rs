use anyhow::{anyhow, Context, Result};

use crate::JError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rank(u8);
pub type DyadRank = (Rank, Rank);

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

    pub fn from_approx(val: f32) -> Result<Self> {
        if val.is_infinite() && val.is_sign_positive() {
            return Ok(Rank::infinite());
        }
        if !val.is_finite() || val < -65. || val > 65. {
            return Err(JError::LimitError)
                .with_context(|| anyhow!("ranks must be infinite or -64 to 64, not {val:?}"));
        }

        let rounded = val.round();
        if (val - rounded).abs() > f32::EPSILON {
            return Err(JError::LimitError)
                .with_context(|| anyhow!("ranks must look like integers, not {val:?}"));
        }

        // already checked this is >=0, << 100
        Self::new(rounded as u8 as u32)
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
    pub const fn inf_inf_inf() -> (Self, (Self, Self)) {
        (Self::infinite(), Self::infinite_infinite())
    }

    pub const fn is_infinite(&self) -> bool {
        self.0 == u8::MAX
    }

    pub fn usize(&self) -> Option<usize> {
        if self.is_infinite() {
            return None;
        }
        Some(usize::from(self.0))
    }

    pub fn raw_u8(&self) -> u8 {
        self.0
    }
}
