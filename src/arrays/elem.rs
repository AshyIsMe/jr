use num::complex::Complex64;
use num::{BigInt, BigRational};
use std::cmp::Ordering;
use std::fmt::Debug;

use crate::number::Num;
use crate::JArray;

#[derive(Clone, Debug)]
pub enum Elem {
    Num(Num),
    Char(char),
    Boxed(JArray),
}

macro_rules! from_num {
    ($t:ty) => {
        impl From<$t> for Elem {
            fn from(value: $t) -> Self {
                Self::Num(value.into())
            }
        }

        impl From<&$t> for Elem {
            fn from(value: &$t) -> Self {
                Self::Num(value.clone().into())
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

impl From<&char> for Elem {
    fn from(value: &char) -> Self {
        Elem::Char(*value)
    }
}

impl From<JArray> for Elem {
    fn from(value: JArray) -> Self {
        Elem::Boxed(value)
    }
}
impl From<&JArray> for Elem {
    fn from(value: &JArray) -> Self {
        Elem::Boxed(value.clone())
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

impl PartialOrd for Elem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Elem::*;
        // > The types: numeric or empty, symbol, literal (1 byte or 2 byte characters), and boxed, are so ordered
        // imagine if they could use the same name for types at any juncture
        // I think "symbol" means what we're calling "literal", and "literal" means "char"? (???)
        match (self, other) {
            (Num(l), Num(r)) => l.partial_cmp(r),
            (Num(_), _) => Some(Ordering::Less),
            (_, Num(_)) => Some(Ordering::Greater),

            (Char(l), Char(r)) => l.partial_cmp(r),
            (Char(_), _) => Some(Ordering::Less),
            (_, Char(_)) => Some(Ordering::Greater),

            _ => None,
        }
    }
}
