use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::{One, ToPrimitive};

pub enum Num {
    Bool(u8),
    Int(i64),
    ExtInt(BigInt),
    Rational(BigRational),
    Float(f64),
    Complex(Complex64),
}

impl Num {
    pub fn approx_f64(&self) -> Option<f64> {
        Some(match self {
            Num::Bool(i) => *i as f64,
            Num::Int(i) => *i as f64,
            Num::ExtInt(i) => i.to_f64()?,
            Num::Rational(i) => i.to_f64()?,
            Num::Float(i) => *i,
            Num::Complex(_) => return None,
        })
    }

    pub fn demote(self) -> Num {
        match self {
            Num::Complex(c) if float_is_zero(c.im) => Num::Float(c.re).demote(),
            Num::Complex(c) => Num::Complex(c),
            Num::Float(f) => {
                if let Some(i) = float_is_int(f) {
                    Num::Int(i).demote()
                } else {
                    Num::Float(f)
                }
            }
            Num::Rational(r) if r.denom().is_one() => Num::ExtInt(r.numer().clone()),
            Num::Rational(r) => Num::Rational(r),
            Num::ExtInt(i) => Num::ExtInt(i),
            Num::Int(i) if i == 0 || i == 1 => Num::Bool(i as u8),
            Num::Int(i) => Num::Int(i),
            Num::Bool(b) => Num::Bool(b),
        }
    }
}

fn float_is_zero(v: f64) -> bool {
    v.abs() < f64::EPSILON
}

const MAX_SAFE_INTEGER: f64 = 9007199254740991.;

fn float_is_int(v: f64) -> Option<i64> {
    if float_is_zero(v) {
        return Some(0);
    }
    if v.is_infinite() || v.is_nan() {
        return None;
    }
    if v.abs() > MAX_SAFE_INTEGER {
        return None;
    }
    if !float_is_zero(v - v.round()) {
        return None;
    }
    Some(v as i64)
}

macro_rules! impl_from_atom {
    ($t:ty, $j:path) => {
        impl From<$t> for Num {
            fn from(value: $t) -> Num {
                $j(value)
            }
        }
    };
}
impl_from_atom!(u8, Num::Bool);
impl_from_atom!(i64, Num::Int);
impl_from_atom!(BigInt, Num::ExtInt);
impl_from_atom!(BigRational, Num::Rational);
impl_from_atom!(f64, Num::Float);
impl_from_atom!(Complex64, Num::Complex);
