use std::ops;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use crate::number::{promote_to_array, Num};
use crate::{Elem, Word};

pub fn scan_litnumarray(sentence: &str) -> Result<(usize, Word)> {
    let sentence = sentence
        .chars()
        .take_while(|&c| matches!(c, '0'..='9' | '.' | '_' | 'e' | 'j' | 'r' | ' ' | '\t'))
        .collect::<String>();

    let l = sentence.len() - 1;

    let parts = sentence
        .split_whitespace()
        .map(|term| scan_num_token(term).with_context(|| anyhow!("parsing {term:?}")))
        .map_ok(Elem::Num)
        .collect::<Result<Vec<_>>>()?;

    Ok((l, Word::Noun(promote_to_array(parts)?)))
}

pub fn scan_num_token(term: &str) -> Result<Num> {
    Ok(if let Some(inf) = parse_infinity(term) {
        Num::Float(inf)
    } else if term.contains('j') {
        Num::Complex(parse_complex(term)?)
    } else if term.contains('r') {
        Num::Rational(parse_rational(term)?)
    } else if term.contains('.') || term.contains('e') {
        Num::Float(parse_float(term)?)
    } else {
        // we can't just demote 'cos bigints never demote
        match sign_lift(term, |term| Ok(term.parse::<i64>()?)) {
            Ok(x) => Num::Int(x),
            Err(_) => Num::ExtInt(parse_bigint(term)?),
        }
    }
    .demote())
}

// TODO also parse_nan "_."
fn parse_infinity(term: &str) -> Option<f64> {
    if term == "_" {
        Some(f64::INFINITY)
    } else if term == "__" {
        Some(f64::NEG_INFINITY)
    } else {
        None
    }
}

fn parse_complex(term: &str) -> Result<Complex64> {
    let (real, imaj) = term
        .split_once('j')
        .expect("scan_complex only sees delimited numbers");
    Ok(Complex64::new(
        parse_float(real).context("real")?,
        parse_float(imaj).context("imaginary")?,
    ))
}

fn parse_rational(term: &str) -> Result<BigRational> {
    let (numer, denom) = term
        .split_once('r')
        .expect("scan_rational only sees delimited numbers");
    Ok(BigRational::new(
        parse_bigint(numer).context("numerator")?,
        parse_bigint(denom).context("denominator")?,
    ))
}

fn parse_float(term: &str) -> Result<f64> {
    // need to duplicate this here so it's picked up in the complex parsing (this is tested)
    if let Some(inf) = parse_infinity(term) {
        return Ok(inf);
    }
    sign_lift(term, |v| {
        v.parse()
            .with_context(|| anyhow!("parsing {v:?} as a float"))
    })
}

fn parse_bigint(term: &str) -> Result<BigInt> {
    sign_lift(term, |v| {
        v.parse()
            .with_context(|| anyhow!("parsing {v:?} as a bigint"))
    })
}

/// adapts an existing parse function, `f`, to handle leading `_` as negative
#[inline]
fn sign_lift<T: ops::Neg<Output = T>>(term: &str, f: impl FnOnce(&str) -> Result<T>) -> Result<T> {
    if term.contains('-') {
        unreachable!("numbers must contain _, not -");
    }
    Ok(if let Some(stripped) = term.strip_prefix('_') {
        -f(stripped)?
    } else {
        f(term)?
    })
}

#[cfg(test)]
mod tests {
    use ndarray::array;
    use num::complex::Complex64;
    use num::rational::BigRational;
    use num::BigInt;

    use crate::{arr0d, JArray, Word};

    fn litnum_to_array(sentence: &str) -> JArray {
        let (l, word) =
            super::scan_litnumarray(sentence).expect(&format!("scanning success on {sentence:?}"));
        assert_eq!(
            l,
            sentence.len() - 1,
            "totally consumed, not stopping near {:?}",
            &sentence[l..]
        );
        match word {
            Word::Noun(arr) => return arr,
            _ => panic!("scan_litnumarray always returns nouns, not {word:?}"),
        }
    }

    #[test]
    fn scan_litnum_homo() {
        assert_eq!(
            array![1i64, 2, 3].into_dyn(),
            litnum_to_array("1 2 3").when_i64().expect("int array"),
        );

        assert_eq!(
            array![1.1, 2., 3.].into_dyn(),
            litnum_to_array("1.1 2 3").when_f64().expect("float array"),
        );

        assert_eq!(
            array![1u8, 0, 1].into_dyn(),
            litnum_to_array("1 0 1").when_u8().expect("bool array"),
        );

        assert_eq!(
            array![Complex64::new(1., 2.), Complex64::new(0., 1.)].into_dyn(),
            litnum_to_array("1j2 0j1")
                .when_complex()
                .expect("complex array"),
        );

        assert_eq!(
            array![
                BigRational::new(1.into(), 2.into()),
                BigRational::new(2.into(), 3.into())
            ]
            .into_dyn(),
            litnum_to_array("1r2 2r3")
                .when_rational()
                .expect("rational array"),
        );
    }

    #[test]
    fn scan_litnum_promo() {
        assert_eq!(
            array![
                Complex64::new(1., 0.),
                Complex64::new(2.5, 0.),
                Complex64::new(3., 2.)
            ]
            .into_dyn(),
            litnum_to_array("1 2.5 3j2")
                .when_complex()
                .expect("complex array"),
        );

        assert_eq!(
            array![1., 2.5, 0.25].into_dyn(),
            litnum_to_array("1 2.5 1r4")
                .when_f64()
                .expect("float array"),
        );

        assert_eq!(
            array![
                BigRational::new(1.into(), 1.into()),
                BigRational::new(1.into(), 4.into()),
                BigRational::new(16.into(), 1.into())
            ]
            .into_dyn(),
            litnum_to_array("1 1r4 16")
                .when_rational()
                .expect("rational array"),
        );

        assert_eq!(
            array![
                BigInt::from(1),
                4.into(),
                "123123123123123123123123123123123"
                    .parse()
                    .expect("valid literal"),
            ]
            .into_dyn(),
            litnum_to_array("1 4 123123123123123123123123123123123")
                .when_bigint()
                .expect("bigint array"),
        );

        assert_eq!(
            array![1, 2, 1].into_dyn(),
            litnum_to_array("1 2 1").when_i64().expect("int array"),
        );

        assert_eq!(
            array![1u8, 0, 1].into_dyn(),
            litnum_to_array("1 0 1").when_u8().expect("bool array"),
        );
    }

    #[test]
    fn scan_litnum_atom() {
        assert_eq!(
            arr0d(1u8),
            litnum_to_array("1").when_u8().expect("bool array"),
        );

        assert_eq!(
            arr0d(12i64),
            litnum_to_array("12").when_i64().expect("int array"),
        );

        // (3!:0) 1e20
        assert_eq!(
            arr0d(1e20f64),
            litnum_to_array("1e20").when_f64().expect("float array"),
        );

        assert_eq!(
            arr0d(Complex64::new(1., 2.)),
            litnum_to_array("1j2")
                .when_complex()
                .expect("complex array"),
        );

        assert_eq!(
            arr0d(BigRational::new(1.into(), 2.into())),
            litnum_to_array("1r2")
                .when_rational()
                .expect("rational array"),
        );
    }

    #[test]
    fn scan_litnum_demo() {
        assert_eq!(
            array![BigInt::from(1), 4.into(), 1.into()].into_dyn(),
            litnum_to_array("1 4 4r4")
                .when_bigint()
                .expect("bigint array"),
        );

        assert_eq!(
            array![1, 1, 1, 0].into_dyn(),
            litnum_to_array("1j0 1.0 1 0.0")
                .when_u8()
                .expect("bool array"),
        );
    }

    #[test]
    fn scan_litnum_negatory() {
        assert_eq!(
            arr0d(Complex64::new(f64::NEG_INFINITY, f64::NEG_INFINITY)),
            litnum_to_array("__j__")
                .when_complex()
                .expect("complex array"),
        );

        assert_eq!(
            arr0d(BigRational::new((-13i8).into(), (-7).into())),
            litnum_to_array("_13r_7")
                .when_rational()
                .expect("rational array"),
        );

        assert_eq!(
            array![1., f64::INFINITY, 1.].into_dyn(),
            litnum_to_array("1 _ 1").when_f64().expect("float array"),
        );
    }
}
