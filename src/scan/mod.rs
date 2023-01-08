mod litnum;
#[cfg(test)]
mod test_weird;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use ndarray::prelude::*;

use crate::arrays::*;
use crate::JError;
use crate::{primitive_adverbs, primitive_conjunctions, primitive_nouns, primitive_verbs};

use crate::verbs::VerbImpl;
use litnum::scan_litnumarray;
pub use litnum::scan_num_token;
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
    let mut loc_off = 0;
    for line in sentence.split('\n') {
        words.extend(
            scan_one_line(line)?
                .into_iter()
                .map(|((ps, pe), word)| ((ps + loc_off, pe + loc_off), word)),
        );
        loc_off += line.len();
        words.push(((loc_off, loc_off), Word::NewLine));
    }
    let _ = words.pop();
    Ok(words)
}

fn scan_one_line(sentence: &str) -> Result<Vec<(Pos, Word)>> {
    let mut words: Vec<(Pos, Word)> = Vec::new();
    let mut skip: usize = 0;
    for (i, c) in sentence.char_indices() {
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
            // 0:, 1:..
            c if c.is_ascii_digit() && sentence[i + 1..].starts_with(':') => {
                words.push((
                    (i, i + 1),
                    Word::Verb(VerbImpl::Number((c as u8 - b'0') as f64)),
                ));
                skip = 1;
            }
            // _:
            '_' if sentence[i + 1..].starts_with(":") => {
                words.push(((i, i + 1), Word::Verb(VerbImpl::Number(f64::INFINITY))));
                skip = 1;
            }
            // _0:, _1:, ..
            '_' if sentence[i + 1..].starts_with(|c: char| c.is_ascii_digit())
                && sentence[i + 2..].starts_with(":") =>
            {
                let c = sentence[i + 1..].chars().next().expect("checked");
                words.push((
                    (i, i + 2),
                    Word::Verb(VerbImpl::Number(-((c as u8 - b'0') as f64))),
                ));
                skip = 2;
            }
            '0'..='9' | '_' => {
                let (l, t) = scan_litnumarray(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
            }
            '\'' => {
                let (l, t) = scan_litstring(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
            }
            'a'..='z' | 'A'..='Z' => {
                let (l, t) = scan_name(&sentence[i..])?;
                if matches!(t, Word::Comment) {
                    break;
                }
                words.push(((i, i + l), t));
                skip = l;
            }
            _ => {
                let (l, t) = scan_primitive(&sentence[i..])?;
                words.push(((i, i + l), t));
                skip = l;
            }
        }
    }
    Ok(words)
}

fn scan_litstring(sentence: &str) -> Result<(usize, Word)> {
    assert!(!sentence.contains('\n'));
    if sentence.len() < 2 {
        return Err(JError::OpenQuote).context("quote followed by nothing");
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
                // this parser doesn't see new lines, I don't think?
                if prev_c_is_quote {
                    l -= 1;
                    break;
                } else {
                    return Err(JError::OpenQuote).context("new line byte in string");
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
    if !prev_c_is_quote {
        return Err(JError::OpenQuote).context("finished parsing while in a string");
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
                s.chars().next().unwrap(),
            ))),
        ))
    } else {
        Ok((l, char_array(&s)))
    }
}

pub fn char_array(x: impl AsRef<str>) -> Word {
    Noun(JArray::from_string(x))
}

fn scan_name(sentence: &str) -> Result<(usize, Word)> {
    assert!(!sentence.contains('\n'));
    let mut it = sentence.chars().peekable();
    let base: String = it
        .peeking_take_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_'))
        .collect();
    let suffix = it.peek().filter(|c| matches!(c, '.' | ':')).copied();

    if let Some(suffix) = suffix {
        if let Some(primitive) = str_to_primitive(&format!("{base}{suffix}"))? {
            return Ok((base.len(), primitive));
        }
    }

    if base.is_empty() {
        return Err(JError::SyntaxError)
            .context("generated an empty name (this is probably a parser bug)");
    }

    Ok((
        base.len() - 1,
        str_to_primitive(&base)?.unwrap_or_else(|| Word::Name(base)),
    ))
}

fn scan_primitive(sentence: &str) -> Result<(usize, Word)> {
    assert!(!sentence.contains('\n'));
    if sentence.is_empty() {
        return Err(JError::custom("Empty primitive"));
    }

    if let Some(kinded) = sentence.strip_prefix("{{)") {
        let mode = kinded
            .chars()
            .next()
            .ok_or_else(|| anyhow!("unexpected empty type in kinded direct definition"))?;
        return Ok((3 + mode.len_utf8(), Word::DirectDef(mode)));
    }

    let l = identify_primitive(sentence);
    let term = sentence.chars().take(l + 1).collect::<String>();
    Ok((
        l,
        str_to_primitive(&term)?
            .ok_or_else(|| anyhow!("parsed as a primitive, but unrecognised: {term:?}"))?,
    ))
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

pub fn str_to_primitive(sentence: &str) -> Result<Option<Word>> {
    Ok(Some(if let Some(n) = primitive_nouns(sentence) {
        n
    } else if let Some(v) = primitive_verbs(sentence) {
        Word::Verb(v)
    } else if let Some(a) = primitive_adverbs(sentence) {
        Word::Adverb(a)
    } else if let Some(c) = primitive_conjunctions(sentence) {
        Word::Conjunction(c)
    } else {
        if let Some(x) = sentence.strip_prefix("for_") {
            if let Some(x) = x.strip_suffix(".") {
                return Ok(Some(Word::For(Some(x.to_string()))));
            }
        }
        match sentence {
            "=:" => Word::IsGlobal,
            "=." => Word::IsLocal,
            "{{" => Word::DirectDefUnknown,
            "}}" => Word::DirectDefEnd,
            "if." => Word::If,
            "do." => Word::Do,
            "else." => Word::Else,
            "elseif." => Word::ElseIf,
            "end." => Word::End,
            "for." => Word::For(None),
            "while." => Word::While,
            "assert." => Word::Assert,
            "try." => Word::Try,
            "catch." => Word::Catch,
            "catchd." => Word::CatchD,
            "catcht." => Word::CatchT,
            "throw." => Word::Throw,
            "return." => Word::Return,
            "NB." => Word::Comment,
            "0:" => Verb(VerbImpl::Number(0.)),
            _ => return Ok(None),
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::{scan, Word};
    use crate::scan::{identify_primitive, scan_litstring};
    use crate::{JArray, JError};

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

    #[test]
    fn unclosed_string() {
        assert!(matches!(
            JError::extract(&scan_litstring("' 123").unwrap_err()),
            Some(JError::OpenQuote)
        ));
        assert!(matches!(
            JError::extract(&scan_litstring("'").unwrap_err()),
            Some(JError::OpenQuote)
        ));
    }

    #[test]
    fn names() {
        let result = dbg!(scan("i.2 3").unwrap());
        assert_eq!(2, result.len());
        assert!(matches!(result[0], Word::Verb(_)));
        assert_eq!(result[1], Word::Noun(JArray::from_list(vec![2i64, 3])));
    }
}
