use std::ops;

use anyhow::{anyhow, Context, Result};
use ndarray::prelude::*;
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::{One, ToPrimitive};

use crate::arrays::*;
use crate::modifiers::ModifierImpl;
use crate::JError;
use crate::{primitive_adverbs, primitive_conjunctions, primitive_nouns, primitive_verbs};

use JArray::*;
use Word::*;

type Pos = (usize, usize);

pub fn scan(sentence: &str) -> Result<Vec<Word>> {
    Ok(scan_with_locations(sentence)?
        .into_iter()
        .map(|(_, token)| token)
        .collect())
}

pub fn scan_with_locations(sentence: &str) -> Result<Vec<(Pos, Word)>> {
    let mut words: Vec<(Pos, Word)> = Vec::new();

    let mut skip: usize = 0;

    //TODO support multiline definitions.
    for (i, c) in sentence.chars().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match c {
            '(' => {
                words.push(((i, i), Word::LP));
            }
            ')' => {
                words.push(((i, i), Word::RP));
            }
            c if c.is_whitespace() => (),
            '0'..='9' | '_' => {
                let (l, t) = scan_litnumarray(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
                continue;
            }
            '\'' => {
                let (l, t) = scan_litstring(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
                continue;
            }
            'a'..='z' | 'A'..='Z' => {
                let (l, t) = scan_name(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
                continue;
            }
            _ => {
                let (l, t) = scan_primitive(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
                continue;
            }
        }
    }
    Ok(words)
}

/// adapts an existing parse function, `f`, to handle leading `_` as negative
fn sign_lift<T: ops::Neg<Output = T>>(term: &str, f: impl FnOnce(&str) -> Result<T>) -> Result<T> {
    if term.contains('-') {
        unreachable!("numbers must contain _, not -");
    }
    Ok(if term.starts_with('_') {
        -f(&term[1..])?
    } else {
        f(term)?
    })
}

fn parse_float(term: &str) -> Result<f64> {
    sign_lift(term, |v| Ok(v.parse()?))
}

fn parse_bigint(term: &str) -> Result<BigInt> {
    sign_lift(term, |v| Ok(v.parse()?))
}

enum Num {
    Bool(u8),
    Int(i64),
    ExtInt(BigInt),
    Rational(BigRational),
    Float(f64),
    Complex(Complex64),
}

impl Num {
    fn approx_f64(&self) -> Option<f64> {
        Some(match self {
            Num::Bool(i) => *i as f64,
            Num::Int(i) => *i as f64,
            Num::ExtInt(i) => i.to_f64()?,
            Num::Rational(i) => i.to_f64()?,
            Num::Float(i) => *i,
            Num::Complex(_) => return None,
        })
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

impl Num {
    fn demote(self) -> Num {
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

fn scan_num_token(term: &str) -> Result<Num> {
    Ok(if term.contains('j') {
        Num::Complex(scan_complex(term)?)
    } else if term.contains('r') {
        Num::Rational(scan_rational(term)?)
    } else if term.contains('.') || term.contains('e') {
        Num::Float(parse_float(term)?)
    } else {
        // we can't just demote 'cos bigints never demote
        match sign_lift(&term, |term| Ok(term.parse::<i64>()?)) {
            Ok(x) => Num::Int(x),
            Err(_) => Num::ExtInt(parse_bigint(&term)?),
        }
    }
    .demote())
}

#[inline]
fn arrayise<T>(it: impl IntoIterator<Item = Result<T>>) -> Result<JArray>
where
    T: Clone,
    ArrayD<T>: IntoJArray,
{
    let vec = it.into_iter().collect::<Result<Vec<T>>>()?;
    Ok(if vec.len() == 1 {
        ArrayD::from_elem(IxDyn(&[]), vec.into_iter().next().expect("checked length"))
    } else {
        ArrayD::from_shape_vec(IxDyn(&[vec.len()]), vec).expect("simple shape")
    }
    .into_jarray())
}

fn scan_litnumarray(sentence: &str) -> Result<(usize, Word)> {
    if sentence.is_empty() {
        return Err(JError::custom("Empty number literal"));
    }
    let sentence = sentence
        .chars()
        .take_while(|&c| matches!(c, '0'..='9' | '.' | '_' | 'e' | 'j' | 'r' | ' ' | '\t'))
        .collect::<String>();

    let l = sentence.len() - 1;

    let parts = sentence
        .split_whitespace()
        .map(|term| scan_num_token(term).with_context(|| anyhow!("parsing {term:?}")))
        .collect::<Result<Vec<_>>>()?;

    // priority table: https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions#Numeric_Precisions_in_J
    let parts = if parts.iter().any(|n| matches!(n, Num::Complex(_))) {
        arrayise(parts.into_iter().map(|v| {
            Ok(match v {
                Num::Complex(i) => i,
                other => Complex64::new(other.approx_f64().expect("covered above"), 0.),
            })
        }))?
    } else if parts.iter().any(|n| matches!(n, Num::Float(_))) {
        arrayise(parts.into_iter().map(|v| {
            Ok(match v {
                Num::Complex(_) => unreachable!("covered by above cases"),
                Num::Float(i) => i,
                other => other.approx_f64().expect("covered above"),
            })
        }))?
    } else if parts.iter().any(|n| matches!(n, Num::Rational(_))) {
        arrayise(parts.into_iter().map(|v| {
            Ok(match v {
                Num::Complex(_) | Num::Float(_) => unreachable!("covered by above cases"),
                Num::Rational(i) => i,
                Num::ExtInt(i) => BigRational::new(i, 1.into()),
                Num::Int(i) => BigRational::new(i.into(), 1.into()),
                Num::Bool(i) => BigRational::new(i.into(), 1.into()),
            })
        }))?
    } else if parts.iter().any(|n| matches!(n, Num::ExtInt(_))) {
        arrayise(parts.into_iter().map(|v| {
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) => {
                    unreachable!("covered by above cases")
                }
                Num::ExtInt(i) => i,
                Num::Int(i) => i.into(),
                Num::Bool(i) => i.into(),
            })
        }))?
    } else if parts.iter().any(|n| matches!(n, Num::Int(_))) {
        arrayise(parts.into_iter().map(|v| {
            Ok(match v {
                Num::Complex(_) | Num::Float(_) | Num::Rational(_) | Num::ExtInt(_) => {
                    unreachable!("covered by above cases")
                }
                Num::Int(i) => i,
                Num::Bool(i) => i.into(),
            })
        }))?
    } else {
        arrayise(parts.into_iter().map(|v| {
            Ok(match v {
                Num::Complex(_)
                | Num::Float(_)
                | Num::Rational(_)
                | Num::ExtInt(_)
                | Num::Int(_) => unreachable!("covered by above cases"),
                Num::Bool(i) => i,
            })
        }))?
    };

    Ok((l, Word::Noun(parts)))
}

fn scan_complex(term: &str) -> Result<Complex64> {
    let (real, imaj) = term
        .split_once('j')
        .expect("scan_complex only sees delimited numbers");
    Ok(Complex64::new(parse_float(real)?, parse_float(imaj)?))
}

fn scan_rational(term: &str) -> Result<BigRational> {
    let (numer, denom) = term
        .split_once('r')
        .expect("scan_rational only sees delimited numbers");
    Ok(BigRational::new(numer.parse()?, denom.parse()?))
}

fn scan_litstring(sentence: &str) -> Result<(usize, Word)> {
    if sentence.len() < 2 {
        return Err(JError::custom("Empty literal string"));
    }

    let mut l: usize = usize::MAX;
    let mut prev_c_is_quote: bool = false;
    // strings in j are single quoted: 'foobar'.
    // literal ' chars are included in a string by doubling: 'foo ''lol'' bar'.
    for (i, c) in sentence.chars().enumerate().skip(1) {
        l = i;
        match c {
            '\'' => match prev_c_is_quote {
                true =>
                // double quote in string, literal quote char
                {
                    prev_c_is_quote = false
                }
                false => prev_c_is_quote = true,
            },
            '\n' => {
                if prev_c_is_quote {
                    l -= 1;
                    break;
                } else {
                    return Err(JError::custom("open quote"));
                }
            }
            _ => match prev_c_is_quote {
                true => {
                    //string closed previous char
                    l -= 1;
                    break;
                }
                false => (), //still valid keep iterating
            },
        }
    }

    assert!(l <= sentence.chars().count(), "l past end of string: {}", l);
    let s = sentence
        .chars()
        .take(l)
        .skip(1)
        .collect::<String>()
        .replace("''", "'");
    if s.len() == 1 {
        Ok((
            l,
            Noun(CharArray(ArrayD::from_elem(
                IxDyn(&[]),
                s.chars().nth(0).unwrap(),
            ))),
        ))
    } else {
        Ok((l, char_array(&s)?))
    }
}

pub fn char_array(x: impl AsRef<str>) -> Result<Word> {
    let v: Vec<char> = x.as_ref().chars().collect();
    Word::noun(v)
}

fn scan_name(sentence: &str) -> Result<(usize, Word)> {
    // user defined adverbs/verbs/nouns
    let mut l: usize = usize::MAX;
    let mut p: Option<Word> = None;
    if sentence.is_empty() {
        return Err(JError::custom("Empty name"));
    }
    for (i, c) in sentence.chars().enumerate() {
        l = i;
        // Name is a word that begins with a letter and contains letters, numerals, and
        // underscores. (See Glossary).
        match c {
            'a'..='z' | 'A'..='Z' | '_' => {
                match p {
                    None => (),
                    Some(_) => {
                        // Primitive was found on previous char, backtrack and break
                        l -= 1;
                        break;
                    }
                }
            }
            '.' | ':' => {
                match p {
                    None => {
                        if let Ok(w) = str_to_primitive(&sentence[0..=l]) {
                            p = Some(w);
                        }
                    }
                    Some(_) => {
                        match str_to_primitive(&sentence[0..=l]) {
                            Ok(w) => p = Some(w),
                            Err(_) => {
                                // Primitive was found on previous char, backtrack and break
                                l -= 1;
                                break;
                            }
                        }
                    }
                }
            }
            _ => {
                l -= 1;
                break;
            }
        }
    }
    match p {
        Some(p) => Ok((l, p)),
        None => Ok((l, Word::Name(sentence[0..=l].to_string()))),
    }
}

fn scan_primitive(sentence: &str) -> Result<(usize, Word)> {
    // built in adverbs/verbs
    let mut l: usize = 0;
    let mut p: Option<char> = None;
    //Primitives are 1 to 3 symbols:
    //  - one symbol
    //  - zero or more trailing . or : or both.
    //  - OR {{ }} for definitions
    if sentence.is_empty() {
        return Err(JError::custom("Empty primitive"));
    }
    for (i, c) in sentence.chars().enumerate() {
        l = i;
        match p {
            None => p = Some(c),
            Some(p) => {
                match p {
                    '{' => {
                        if !"{.:".contains(c) {
                            l -= 1;
                            break;
                        }
                    }
                    '}' => {
                        if !"}.:".contains(c) {
                            l -= 1;
                            break;
                        }
                    }
                    //if !"!\"#$%&*+,-./:;<=>?@[\\]^_`{|}~".contains(c) {
                    _ => {
                        if !".:".contains(c) {
                            l -= 1;
                            break;
                        }
                    }
                }
            }
        }
    }
    Ok((l, str_to_primitive(&sentence[..=l])?))
}

fn str_to_primitive(sentence: &str) -> Result<Word> {
    if primitive_nouns().contains(&sentence) {
        Ok(char_array(sentence)?) // TODO - actually lookup the noun
    } else if let Some(refd) = primitive_verbs(&sentence) {
        Ok(Word::Verb(sentence.to_string(), refd))
    } else if primitive_adverbs().contains_key(&sentence) {
        Ok(Word::Adverb(
            sentence.to_string(),
            match primitive_adverbs().get(&sentence) {
                Some(a) => a.clone(),
                None => ModifierImpl::NotImplemented,
            },
        ))
    } else if primitive_conjunctions().contains_key(&sentence) {
        Ok(Word::Conjunction(
            sentence.to_string(),
            match primitive_conjunctions().get(&sentence) {
                Some(a) => a.clone(),
                None => ModifierImpl::NotImplemented,
            },
        ))
    } else {
        match sentence {
            "=:" => Ok(Word::IsGlobal),
            "=." => Ok(Word::IsLocal),
            _ => Err(JError::custom("Invalid primitive")),
        }
    }
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
}
