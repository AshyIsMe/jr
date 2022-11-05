use anyhow::Result;
use ndarray::prelude::*;
use num::complex::Complex64;
use num::BigRational;

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

fn scan_litnumarray(sentence: &str) -> Result<(usize, Word)> {
    if sentence.is_empty() {
        return Err(JError::custom("Empty number literal"));
    }
    let sentence = sentence
        .chars()
        .take_while(|&c| matches!(c, '0'..='9' | '.' | '_' | 'e' | 'j' | 'r' | ' ' | '\t'))
        .collect::<String>();

    let l = sentence.len() - 1;

    if sentence.contains('j') {
        let a = sentence
            .split_whitespace()
            .map(scan_complex)
            .collect::<Result<Vec<_>>>()?;
        Ok((l, Word::noun(a)?))
    } else if sentence.contains('r') {
        let a = sentence
            .split_whitespace()
            .map(scan_rational)
            .collect::<Result<Vec<_>>>()?;
        Ok((l, Word::noun(a)?))
    } else if sentence.contains('.') || sentence.contains('e') {
        let a = sentence
            .split_whitespace()
            .map(|s| s.replace('_', "-"))
            .map(|s| s.parse::<f64>())
            .collect::<Result<Vec<f64>, std::num::ParseFloatError>>();
        match a {
            Ok(a) => {
                if a.len() == 1 {
                    Ok((l, Noun(FloatArray(ArrayD::from_elem(IxDyn(&[]), a[0])))))
                } else {
                    Ok((
                        l,
                        Noun(FloatArray(ArrayD::from_shape_vec(IxDyn(&[a.len()]), a)?)),
                    ))
                }
            }
            Err(_) => Err(JError::custom("parse float error")),
        }
    } else {
        let a = sentence
            .split_whitespace()
            .map(|s| s.replace('_', "-"))
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<i64>, std::num::ParseIntError>>();
        match a {
            Ok(a) => {
                if a.len() == 1 {
                    // atom
                    if a[0] == 0 || a[0] == 1 {
                        Ok((
                            l,
                            Noun(BoolArray(ArrayD::from_elem(IxDyn(&[]), a[0] as u8))),
                        ))
                    } else {
                        Ok((l, Noun(IntArray(ArrayD::from_elem(IxDyn(&[]), a[0])))))
                    }
                } else if *a.iter().min().unwrap() == 0 && *a.iter().max().unwrap() == 1 {
                    Ok((
                        l,
                        Noun(BoolArray(ArrayD::from_shape_vec(
                            IxDyn(&[a.len()]),
                            a.iter().map(|i| *i as u8).collect(),
                        )?)),
                    ))
                } else {
                    Ok((
                        l,
                        Noun(IntArray(ArrayD::from_shape_vec(IxDyn(&[a.len()]), a)?)),
                    ))
                }
            }
            Err(_) => Err(JError::custom("parse int error")),
        }
    }
}

fn scan_complex(term: &str) -> Result<Complex64> {
    Ok(match term.split_once('j') {
        Some((real, imaj)) => Complex64::new(real.parse()?, imaj.parse()?),
        None => Complex64::new(term.parse()?, 0.),
    })
}

fn scan_rational(term: &str) -> Result<BigRational> {
    Ok(match term.split_once('r') {
        Some((real, imaj)) => BigRational::new(real.parse()?, imaj.parse()?),
        None => BigRational::new(term.parse()?, 1.into()),
    })
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
