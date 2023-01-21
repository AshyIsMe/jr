use anyhow::{anyhow, Context, Result};

use crate::JError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rank(u8);
pub type DyadRank = (Rank, Rank);

#[macro_export]
#[rustfmt::skip]
macro_rules! rank {
    (_          _           _          ) => ((Rank::infinite(), Rank::infinite(), Rank::infinite()));
    ($m:literal _           _          ) => ((Rank::new($m),    Rank::infinite(), Rank::infinite()));
    (_          $dl:literal _          ) => ((Rank::infinite(), Rank::new($dl),   Rank::infinite()));
    ($m:literal $dl:literal _          ) => ((Rank::new($m),    Rank::new($dl),   Rank::infinite()));
    (_          _           $dr:literal) => ((Rank::infinite(), Rank::infinite(), Rank::new($dr)  ));
    ($m:literal _           $dr:literal) => ((Rank::new($m),    Rank::infinite(), Rank::new($dr)  ));
    (_          $dl:literal $dr:literal) => ((Rank::infinite(), Rank::new($dl),   Rank::new($dr)  ));
    ($m:literal $dl:literal $dr:literal) => ((Rank::new($m),    Rank::new($dl),   Rank::new($dr)  ));
}

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
