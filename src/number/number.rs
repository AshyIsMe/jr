use std::cmp::Ordering;
use std::ops;

use num::complex::Complex64;
use num::{BigInt, BigRational, Integer};
use num_traits::{One, ToPrimitive, Zero};

#[derive(Debug, Clone)]
pub enum Num {
    Bool(u8),
    Int(i64),
    ExtInt(BigInt),
    Rational(BigRational),
    Float(f64),
    Complex(Complex64),
}

impl Num {
    pub fn bool(val: bool) -> Num {
        if val {
            Num::Bool(1)
        } else {
            Num::Bool(0)
        }
    }

    /// Int if the decimal part is effectively zero, and it safely fits; otherwise float
    ///
    /// This is then behaviour of a bunch of math operations, like v_floor
    pub fn float_or_int(val: f64) -> Num {
        if let Some(i) = float_is_int(val) {
            Num::Int(i)
        } else {
            Num::Float(val)
        }
    }

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

    /// true if the value looks like a 1, regardless of type; false if it looks like a 0
    pub fn value_bool(&self) -> Option<bool> {
        use Num::*;
        Some(match self {
            Bool(i) => *i == 1,
            Int(i) if *i == 1 => true,
            Int(i) if *i == 0 => false,
            Int(_) => return None,
            ExtInt(i) if i.is_one() => true,
            ExtInt(i) if i.is_zero() => false,
            ExtInt(_) => return None,
            // moderately inefficient, but this is mostsly only used on small lists?
            Float(f) => return float_is_int(*f).and_then(|i| Int(i).value_bool()),
            Rational(r) => return r.to_f64().and_then(|f| Float(f).value_bool()),
            Complex(c) => return complex_is_float(c).and_then(|f| Float(f).value_bool()),
        })
    }

    /// the `usize` in the value, regardless of type
    pub fn value_len(&self) -> Option<usize> {
        use Num::*;
        match self {
            Bool(i) => i.to_usize(),
            Int(i) => i.to_usize(),
            ExtInt(i) => i.to_usize(),
            Rational(f) => f.to_f64().and_then(|f| Float(f).value_len()),
            Float(f) if *f < 0. => None,
            Float(f) => float_is_int(*f).and_then(|i| usize::try_from(i).ok()),
            Complex(c) => complex_is_float(c).and_then(|f| Float(f).value_len()),
        }
    }

    /// the `i64` in the value, regardless of typ
    pub fn value_i64(&self) -> Option<i64> {
        use Num::*;
        match self {
            Bool(i) => Some(i64::from(*i)),
            Int(i) => Some(*i),
            ExtInt(i) => i.to_i64(),
            Float(f) => float_is_int(*f),

            // rational.to_i64 truncates
            Rational(f) => f.to_f64().and_then(|f| Float(f).value_i64()),
            Complex(c) => complex_is_float(c).and_then(|f| Float(f).value_i64()),
        }
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

    pub fn one() -> Self {
        Num::Bool(1)
    }

    pub fn i() -> Self {
        Num::Complex(Complex64::new(0., 1.))
    }
}

fn float_is_zero(v: f64) -> bool {
    v.abs() < f64::EPSILON
}

const MAX_SAFE_INTEGER: f64 = 9007199254740991.;

fn complex_is_float(c: &Complex64) -> Option<f64> {
    if float_is_zero(c.im) {
        Some(c.re)
    } else {
        None
    }
}

pub fn float_is_int(v: f64) -> Option<i64> {
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

#[inline]
fn to_f64_panic(v: impl ToPrimitive) -> f64 {
    v.to_f64()
        .expect("float conversion is infalliable on supported types")
}

#[inline]
fn rational(v: impl Into<BigInt>) -> Num {
    Num::Rational(BigRational::new(v.into(), BigInt::one()))
}

#[inline]
fn complex(v: impl Into<f64>) -> Num {
    Num::Complex(Complex64::new(v.into(), 0.))
}

impl ops::Add for Num {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        use Num::*;
        match promo(self, rhs) {
            (Int(l), Int(r)) => l
                .checked_add(r)
                .map(Int)
                .unwrap_or_else(|| Float(l as f64 + r as f64)),
            (ExtInt(l), ExtInt(r)) => ExtInt(l + r),
            (Rational(l), Rational(r)) => Rational(l + r),
            (Float(l), Float(r)) => Float(l + r),
            (Complex(l), Complex(r)) => Complex(l + r),

            (l, r) => unreachable!("add({l:?}, {r:?})"),
        }
    }
}

impl ops::Sub for Num {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        use Num::*;
        match promo(self, rhs) {
            (Int(l), Int(r)) => l
                .checked_sub(r)
                .map(Int)
                .unwrap_or_else(|| Float(l as f64 - r as f64)),
            (ExtInt(l), ExtInt(r)) => ExtInt(l - r),
            (Rational(l), Rational(r)) => Rational(l - r),
            (Float(l), Float(r)) => Float(l - r),
            (Complex(l), Complex(r)) => Complex(l - r),
            (l, r) => unreachable!("sub({l:?}, {r:?})"),
        }
    }
}

impl ops::Mul for Num {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        use Num::*;
        match promo(self, rhs) {
            (Int(l), Int(r)) => l
                .checked_mul(r)
                .map(Int)
                .unwrap_or_else(|| Float((l as f64) * (r as f64))),
            (ExtInt(l), ExtInt(r)) => ExtInt(l * r),
            (Rational(l), Rational(r)) => Rational(l * r),
            (Float(l), Float(r)) => Float(l * r),
            (Complex(l), Complex(r)) => Complex(l * r),

            (l, r) => unreachable!("mul({l:?}, {r:?})"),
        }
    }
}

macro_rules! sign {
    ($x:expr) => {
        if $x > Zero::zero() {
            1.
        } else if $x < Zero::zero() {
            -1.
        } else {
            0.
        }
    };
}

impl ops::Div for Num {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        use Num::*;
        if rhs.is_zero() {
            if self.is_zero() {
                return Num::zero();
            }
            return match self {
                Bool(l) => Float(sign!(l) * f64::INFINITY),
                Int(l) => Float(sign!(l) * f64::INFINITY),
                ExtInt(l) => Float(sign!(l) * f64::INFINITY),
                Rational(l) => Float(sign!(l) * f64::INFINITY),
                Float(l) => Float(l * f64::INFINITY),
                // don't ask
                Complex(l) => Complex(Complex64::new(0., sign!(l.im) * f64::INFINITY)),
            };
        }

        match promo(self, rhs) {
            (Int(l), Int(r)) => match l.div_rem(&r) {
                (o, 0) => Int(o),
                (_, _) => rational(l) / rational(r),
            },
            (ExtInt(l), ExtInt(r)) => match l.div_rem(&r) {
                (o, r) if r.is_zero() => ExtInt(o),
                (_, _) => rational(l) / rational(r),
            },
            (Rational(l), Rational(r)) => Rational(l / r),
            (Float(l), Float(r)) => Float(l / r),
            (Complex(l), Complex(r)) => Complex(l / r),

            (l, r) => unreachable!("mul({l:?}, {r:?})"),
        }
    }
}

impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        use Num::*;
        // TODO: non-cloning version of promo()?
        match promo(self.clone(), other.clone()) {
            (Int(l), Int(r)) => l == r,
            (ExtInt(l), ExtInt(r)) => l == r,
            (Rational(l), Rational(r)) => l == r,
            (Float(l), Float(r)) => l == r,
            (Complex(l), Complex(r)) => l == r,

            (l, r) => unreachable!("eq({l:?}, {r:?})"),
        }
    }
}

impl PartialOrd for Num {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Num::*;
        // TODO: non-cloning version of promo()?
        match promo(self.clone(), other.clone()) {
            (Int(l), Int(r)) => l.partial_cmp(&r),
            (ExtInt(l), ExtInt(r)) => l.partial_cmp(&r),
            (Rational(l), Rational(r)) => l.partial_cmp(&r),
            (Float(l), Float(r)) => l.partial_cmp(&r),
            (Complex(_), Complex(_)) => None,

            (l, r) => unreachable!("cmp({l:?}, {r:?})"),
        }
    }
}

#[inline]
fn flip<T>((b, a): (T, T)) -> (T, T) {
    (a, b)
}

fn promo(l: Num, r: Num) -> (Num, Num) {
    use Num::*;

    match (&l, &r) {
        (Int(_), Bool(_)) => flip(promo_ordered((r, l))),
        (ExtInt(_), Bool(_)) => flip(promo_ordered((r, l))),
        (ExtInt(_), Int(_)) => flip(promo_ordered((r, l))),
        (Rational(_), Bool(_)) => flip(promo_ordered((r, l))),
        (Rational(_), Int(_)) => flip(promo_ordered((r, l))),
        (Rational(_), ExtInt(_)) => flip(promo_ordered((r, l))),
        (Float(_), Bool(_)) => flip(promo_ordered((r, l))),
        (Float(_), Int(_)) => flip(promo_ordered((r, l))),
        (Float(_), ExtInt(_)) => flip(promo_ordered((r, l))),
        (Float(_), Rational(_)) => flip(promo_ordered((r, l))),
        (Complex(_), Bool(_)) => flip(promo_ordered((r, l))),
        (Complex(_), Int(_)) => flip(promo_ordered((r, l))),
        (Complex(_), ExtInt(_)) => flip(promo_ordered((r, l))),
        (Complex(_), Rational(_)) => flip(promo_ordered((r, l))),
        (Complex(_), Float(_)) => flip(promo_ordered((r, l))),
        _ => promo_ordered((l, r)),
    }
}

fn promo_ordered((l, r): (Num, Num)) -> (Num, Num) {
    use Num::*;

    match (l, r) {
        // already similar, don't touch it
        orig @ (Int(_), Int(_)) => orig,
        orig @ (ExtInt(_), ExtInt(_)) => orig,
        orig @ (Rational(_), Rational(_)) => orig,
        orig @ (Float(_), Float(_)) => orig,
        orig @ (Complex(_), Complex(_)) => orig,

        // bool could technically not promote but most(tm) maths ops want promotion
        (Bool(l), Bool(r)) => (Int(l.into()), Int(r.into())),

        // (bool, int) -> int
        (Bool(l), Int(r)) => (Int(l.into()), Int(r)),

        // (bool|int, extint) -> extint
        (Bool(l), ExtInt(r)) => (ExtInt(l.into()), ExtInt(r)),
        (Int(l), ExtInt(r)) => (ExtInt(l.into()), ExtInt(r)),

        // (bool|int|extint, rational) -> rational
        (Bool(l), Rational(r)) => (rational(l), r.into()),
        (Int(l), Rational(r)) => (rational(l), r.into()),
        (ExtInt(l), Rational(r)) => (rational(l), r.into()),

        // (bool|int|extint|rational) -> float
        (Bool(l), Float(r)) => (f64::from(l).into(), r.into()),
        (Int(l), Float(r)) => (to_f64_panic(l).into(), r.into()),
        (ExtInt(l), Float(r)) => (to_f64_panic(l).into(), r.into()),
        (Rational(l), Float(r)) => (to_f64_panic(l).into(), r.into()),

        (Bool(l), Complex(r)) => (complex(l), r.into()),
        (Int(l), Complex(r)) => (complex(l as f64), r.into()),
        (ExtInt(l), Complex(r)) => (complex(to_f64_panic(l)), r.into()),
        (Rational(l), Complex(r)) => (complex(to_f64_panic(l)), r.into()),
        (Float(l), Complex(r)) => (complex(l), r.into()),

        (l, r) => unreachable!("promo({l:?}, {r:?})"),
    }
}

impl Zero for Num {
    fn zero() -> Self {
        Num::Bool(0)
    }

    fn is_zero(&self) -> bool {
        match self {
            Num::Bool(a) => a.is_zero(),
            Num::Int(a) => a.is_zero(),
            Num::ExtInt(a) => a.is_zero(),
            Num::Rational(a) => a.is_zero(),
            Num::Float(a) => a.is_zero(),
            Num::Complex(a) => a.is_zero(),
        }
    }
}
