use anyhow::{anyhow, Context, Result};

use crate::JError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rank(u8);
pub type DyadRank = (Rank, Rank);

impl Rank {
    pub fn new(val: u8) -> Self {
        Self::new_checked(val).expect("unchecked rank creation")
    }

    pub fn new_checked(val: u8) -> Result<Self> {
        if val >= 64 {
            Err(JError::LimitError).with_context(|| anyhow!("{val} is too many ranks"))
        } else {
            Ok(Rank(val))
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
        Self::new_checked(rounded as u8)
    }

    pub const fn infinite() -> Self {
        Rank(u8::MAX)
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
