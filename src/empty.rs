use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational, Zero};

use crate::JArray;
use crate::verbs::VerbImpl;

pub trait HasEmpty {
    fn empty() -> Self;
}

macro_rules! impl_empty {
    ($t:ty, $e:expr) => {
        impl HasEmpty for $t {
            fn empty() -> $t {
                $e
            }
        }
    };
}

impl_empty!(char, ' ');
impl_empty!(u8, 0);
impl_empty!(i64, 0);
impl_empty!(BigInt, BigInt::zero());
impl_empty!(BigRational, BigRational::zero());
impl_empty!(f64, 0.);
impl_empty!(Complex64, Complex64::zero());
impl_empty!(JArray, JArray::BoolArray(Array::from_elem(IxDyn(&[0]), 0)));
impl_empty!(
    VerbImpl,
    VerbImpl::Null
);
