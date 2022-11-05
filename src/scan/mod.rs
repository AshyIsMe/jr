mod litnum;
mod number;

use anyhow::Result;
use ndarray::prelude::*;

use crate::arrays::*;
use crate::modifiers::ModifierImpl;
use crate::JError;
use crate::{primitive_adverbs, primitive_conjunctions, primitive_nouns, primitive_verbs};

use litnum::scan_litnumarray;
use JArray::*;
use Word::*;

type Pos = (usize, usize);

pub use litnum::promote_to_array;
pub use number::Num;

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
    if sentence.is_empty() {
        return Err(JError::custom("Empty primitive"));
    }
    let l = identify_primitive(sentence);
    Ok((l, str_to_primitive(&sentence[..=l])?))
}

fn identify_primitive(sentence: &str) -> usize {
    let mut it = sentence.chars();
    let initial = it.next().expect("non-empty input");

    it.take_while(match initial {
        '{' => |c: &char| "{.:".contains(*c),
        '}' => |c: &char| "}.:".contains(*c),
        _ => |c: &char| ".:".contains(*c),
    })
    .count()
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
    use crate::scan::identify_primitive;

    fn ident(sentence: &str) -> usize {
        // oh god please
        identify_primitive(sentence) + 1
    }

    #[test]
    fn identify_prim() {
        assert_eq!(1, ident("{ butts"));
        assert_eq!(2, ident("{. butts"));
        assert_eq!(3, ident("{.. butts"));
        assert_eq!(3, ident("{.: butts"));
        assert_eq!(3, ident("{:. butts"));

        assert_eq!(1, ident("} butts"));
        assert_eq!(2, ident("}. butts"));
        assert_eq!(3, ident("}.. butts"));
        assert_eq!(3, ident("}.: butts"));
        assert_eq!(3, ident("}:. butts"));

        assert_eq!(5, ident("{{{:. butts"));

        assert_eq!(1, ident("a butts"));
        assert_eq!(2, ident("a. butts"));
        assert_eq!(3, ident("a.: butts"));
        assert_eq!(2, ident(":: butts"));
        assert_eq!(1, ident("{a"));
        assert_eq!(1, ident("}a"));
        assert_eq!(1, ident("a{{"));
        assert_eq!(1, ident("a}}"));
    }
}
