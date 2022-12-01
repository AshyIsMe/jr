use num::complex::Complex64;
use num::{BigInt, BigRational};

use crate::number::Num;
use crate::verbs::VerbImpl;
use crate::JArray;

#[derive(Clone, Debug)]
pub enum Elem {
    Num(Num),
    Char(char),
    Boxed(JArray),
    Literal(VerbImpl),
}

macro_rules! from_num {
    ($t:ty) => {
        impl From<$t> for Elem {
            fn from(value: $t) -> Self {
                Self::Num(value.into())
            }
        }
    };
}

from_num!(u8);
from_num!(i64);
from_num!(BigInt);
from_num!(BigRational);
from_num!(f64);
from_num!(Complex64);

impl From<char> for Elem {
    fn from(value: char) -> Self {
        Elem::Char(value)
    }
}

impl From<JArray> for Elem {
    fn from(value: JArray) -> Self {
        Elem::Boxed(value)
    }
}

impl From<VerbImpl> for Elem {
    fn from(value: VerbImpl) -> Self {
        Elem::Literal(value)
    }
}

impl PartialEq for Elem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Elem::Num(l), Elem::Num(r)) => l == r,
            (Elem::Boxed(l), Elem::Boxed(r)) => l == r,
            (Elem::Char(l), Elem::Char(r)) => l == r,
            _ => false,
        }
    }
}
