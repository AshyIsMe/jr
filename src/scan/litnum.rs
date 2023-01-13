use std::ops;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use num::complex::Complex64;
use num::{BigInt, BigRational};

use crate::number::{promote_to_array, Num};
use crate::{Elem, JError, Word};

pub fn scan_litnumarray(sentence: &str) -> Result<(usize, Word)> {
    assert!(!sentence.contains('\n'));
    assert!(!sentence.is_empty());

    // an array *potentially* extends until the first symbol character
    let sentence = match sentence.find(|c: char| {
        !(c.is_ascii_alphanumeric() || c.is_ascii_whitespace() || ['.', '_', ':'].contains(&c))
    }) {
        Some(c) => &sentence[..c],
        None => sentence,
    };

    // split on the whitespace, and try to parse each 'word', stopping when we can't parse a word
    let parts = sentence
        .split_whitespace()
        .map_while(|term| scan_num_token(term).ok().map(|x| (term, x)))
        .collect_vec();

    // the end is the end of the last successfully parsed term
    let (term, _) = parts
        .last()
        .ok_or(JError::SyntaxError)
        .context("a sentence starting with a digit must contain a valid number")
        .with_context(|| {
            let first_word = sentence.split_whitespace().next().expect("non-empty");
            scan_num_token(first_word).unwrap_err()
        })?;
    let l = term.as_ptr() as usize - sentence.as_ptr() as usize + term.len() - 1;

    // promote_to_array wants the input to be Elem-wrapped
    let parts = parts
        .into_iter()
        .map(|(_term, num)| Elem::Num(num))
        .collect();

    Ok((l, Word::Noun(promote_to_array(parts)?)))
}

pub fn scan_num_token(term: &str) -> Result<Num> {
    if term.contains(':') {
        return Err(anyhow!("colons are not valid in numbers"));
    }
    Ok(if let Some(inf) = parse_infinity(term) {
        Num::Float(inf)
    } else if let Some((base, val)) = term.split_once('b') {
        Num::Int(parse_base(base, val)?)
    } else if term.contains('j') {
        Num::Complex(parse_complex(term)?)
    } else if term.contains('r') {
        Num::Rational(parse_rational(term)?)
    } else if term.contains('.') || term.contains('e') {
        Num::Float(parse_float(term)?)
    } else if term.ends_with('x') {
        Num::ExtInt(parse_bigint_suffixed(term)?)
    } else {
        // we can't just demote 'cos bigints never demote
        match sign_lift(term, |term| Ok(term.parse::<i64>()?)) {
            Ok(x) => Num::Int(x),
            Err(_) => Num::Float(parse_float(term)?),
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
        parse_bigint_plain(numer).context("numerator")?,
        parse_bigint_plain(denom).context("denominator")?,
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

/// a bigint which still has its 'x' suffix
fn parse_bigint_suffixed(term: &str) -> Result<BigInt> {
    let prefix = term
        .strip_suffix('x')
        .ok_or(JError::IllFormedNumber)
        .with_context(|| {
            anyhow!("{term:?} contains an 'x', so it should be an extint, but it is not")
        })?;
    parse_bigint_plain(prefix).context("x-suffixed number")
}

/// a bigint by the standard definition, without the 'x' suffix
fn parse_bigint_plain(term: &str) -> Result<BigInt> {
    sign_lift(term, |v| {
        v.parse()
            .with_context(|| anyhow!("parsing {v:?} as a bigint"))
    })
}

fn parse_base(base: &str, val: &str) -> Result<i64> {
    let base = sign_lift(base, |base| base.parse::<i64>().context("base"))?;
    if base < 2 || base > 36 {
        return Err(JError::NonceError)
            .with_context(|| anyhow!("base {base} is not in supported range (2-36)"));
    }
    i64::from_str_radix(val, u32::try_from(base).expect("just checked"))
        .with_context(|| anyhow!("parsing {val:?} in base {base}"))
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

    use crate::arrays::ArcArrayD;
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
            array![1., 4., 123123123123123123123123123123123.,].into_dyn(),
            litnum_to_array("1 4 123123123123123123123123123123123")
                .when_f64()
                .expect("float array"),
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

    #[test]
    fn scan_litnum_const_funcs() {
        let litnum = |s: &'static str| -> (usize, ArcArrayD<i64>) {
            let (l, w) = super::scan_litnumarray(s).expect("success");
            let arr = match w {
                Word::Noun(arr) => arr,
                _ => unreachable!(),
            };
            (s.len() - l - 1, arr.when_i64().expect("i64").clone())
        };

        assert!(
            super::scan_litnumarray("3:").is_err(),
            "shouldn't get here; calling code should handle it"
        );

        assert_eq!(
            (" 3:".len(), array![7i64, 1, 2].into_dyn().into_shared()),
            litnum("7 1 2 3:")
        );

        assert_eq!(
            (" _3:".len(), array![1i64, -2].into_dyn().into_shared()),
            litnum("1 _2 _3:")
        );

        assert_eq!(
            (" _:".len(), array![1i64, -2].into_dyn().into_shared()),
            litnum("1 _2 _:")
        );
    }
}
