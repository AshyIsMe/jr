use ndarray::prelude::*;
use std::collections::HashMap;
use std::fmt;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary
// A Word is a part of speech.
#[derive(Clone, PartialEq, Debug)]
pub enum Word {
    LP,
    RP,
    Name(String),

    Noun(JArray),
    Verb(String, Option<VerbImpl>),
    Adverb(String),
    Conjunction(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JArray {
    IntArray { v: ArrayD<i64> },
    ExtIntArray { v: ArrayD<i128> }, // TODO: num::bigint::BigInt
    FloatArray { v: ArrayD<f64> },
    BoolArray { v: ArrayD<u8> },
    CharArray { v: ArrayD<char> },
    //RationalArray { ... }, // TODO: num::rational::Rational64
    //ComplexArray { ... },  // TODO: num::complex::Complex64
    //EmptyArray // How do we do this properly?
}

use JArray::*;

pub fn char_array(x: impl AsRef<str>) -> Word {
    let x = x.as_ref();
    Word::Noun(JArray::CharArray {
        v: ArrayD::from_shape_vec(IxDyn(&[x.len()]), String::from(x).chars().collect()).unwrap(),
    })
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Plus,
    Minus,
    Times,
    NotImplemented,
}

impl VerbImpl {
    fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word, JError> {
        match self {
            VerbImpl::Plus => v_plus(x, y),
            VerbImpl::Minus => v_minus(x, y),
            VerbImpl::Times => v_times(x, y),
            VerbImpl::NotImplemented => v_not_implemented(x, y),
        }
    }
}

//fn primitive_verbs() -> &'static [&'static str] {
fn primitive_verbs() -> HashMap<&'static str, VerbImpl> {
    HashMap::from([
        ("=", VerbImpl::NotImplemented),
        ("=.", VerbImpl::NotImplemented),
        ("=:", VerbImpl::NotImplemented),
        ("<", VerbImpl::NotImplemented),
        ("<.", VerbImpl::NotImplemented),
        ("<:", VerbImpl::NotImplemented),
        (">", VerbImpl::NotImplemented),
        (">.", VerbImpl::NotImplemented),
        (">:", VerbImpl::NotImplemented),
        ("_:", VerbImpl::NotImplemented),
        ("+", VerbImpl::Plus),
        ("+.", VerbImpl::NotImplemented),
        ("+:", VerbImpl::NotImplemented),
        ("*", VerbImpl::Times),
        ("*.", VerbImpl::NotImplemented),
        ("*:", VerbImpl::NotImplemented),
        ("-", VerbImpl::Minus),
        ("-.", VerbImpl::NotImplemented),
        ("-:", VerbImpl::NotImplemented),
        ("%", VerbImpl::NotImplemented),
        ("%.", VerbImpl::NotImplemented),
        ("%:", VerbImpl::NotImplemented),
        ("^", VerbImpl::NotImplemented),
        ("^.", VerbImpl::NotImplemented),
        ("^!.", VerbImpl::NotImplemented),
        ("$", VerbImpl::NotImplemented),
        ("$.", VerbImpl::NotImplemented),
        ("$:", VerbImpl::NotImplemented),
        ("~.", VerbImpl::NotImplemented),
        ("~:", VerbImpl::NotImplemented),
        ("|", VerbImpl::NotImplemented),
        ("|.", VerbImpl::NotImplemented),
        ("|:", VerbImpl::NotImplemented),
        (".:", VerbImpl::NotImplemented),
        ("..", VerbImpl::NotImplemented),
        (",.", VerbImpl::NotImplemented),
        (",", VerbImpl::NotImplemented),
        (",:", VerbImpl::NotImplemented),
        (";", VerbImpl::NotImplemented),
        (";:", VerbImpl::NotImplemented),
        ("#", VerbImpl::NotImplemented),
        ("#.", VerbImpl::NotImplemented),
        ("#:", VerbImpl::NotImplemented),
        ("!", VerbImpl::NotImplemented),
        ("/:", VerbImpl::NotImplemented),
        ("\\:", VerbImpl::NotImplemented),
        ("[", VerbImpl::NotImplemented),
        ("[:", VerbImpl::NotImplemented),
        ("]", VerbImpl::NotImplemented),
        ("{", VerbImpl::NotImplemented),
        ("{.", VerbImpl::NotImplemented),
        ("{:", VerbImpl::NotImplemented),
        ("{::", VerbImpl::NotImplemented),
        ("}.", VerbImpl::NotImplemented),
        ("}:", VerbImpl::NotImplemented),
        ("\".", VerbImpl::NotImplemented),
        ("\":", VerbImpl::NotImplemented),
        ("?", VerbImpl::NotImplemented),
        ("?.", VerbImpl::NotImplemented),
        ("A.", VerbImpl::NotImplemented),
        ("C.", VerbImpl::NotImplemented),
        ("C.!.2", VerbImpl::NotImplemented),
        ("e.", VerbImpl::NotImplemented),
        ("E.", VerbImpl::NotImplemented),
        ("i.", VerbImpl::NotImplemented),
        ("i:", VerbImpl::NotImplemented),
        ("I.", VerbImpl::NotImplemented),
        ("j.", VerbImpl::NotImplemented),
        ("L.", VerbImpl::NotImplemented),
        ("o.", VerbImpl::NotImplemented),
        ("p.", VerbImpl::NotImplemented),
        ("p..", VerbImpl::NotImplemented),
        ("p:", VerbImpl::NotImplemented),
        ("q:", VerbImpl::NotImplemented),
        ("r.", VerbImpl::NotImplemented),
        ("s:", VerbImpl::NotImplemented),
        ("T.", VerbImpl::NotImplemented),
        ("u:", VerbImpl::NotImplemented),
        ("x:", VerbImpl::NotImplemented),
        ("Z:", VerbImpl::NotImplemented),
        ("_9:", VerbImpl::NotImplemented),
        ("_8:", VerbImpl::NotImplemented),
        ("_7:", VerbImpl::NotImplemented),
        ("_6:", VerbImpl::NotImplemented),
        ("_5:", VerbImpl::NotImplemented),
        ("_4:", VerbImpl::NotImplemented),
        ("_3:", VerbImpl::NotImplemented),
        ("_2:", VerbImpl::NotImplemented),
        ("_1:", VerbImpl::NotImplemented),
        ("0:", VerbImpl::NotImplemented),
        ("1:", VerbImpl::NotImplemented),
        ("2:", VerbImpl::NotImplemented),
        ("3:", VerbImpl::NotImplemented),
        ("4:", VerbImpl::NotImplemented),
        ("5:", VerbImpl::NotImplemented),
        ("6:", VerbImpl::NotImplemented),
        ("7:", VerbImpl::NotImplemented),
        ("8:", VerbImpl::NotImplemented),
        ("9", VerbImpl::NotImplemented),
        ("u.", VerbImpl::NotImplemented),
        ("v.", VerbImpl::NotImplemented),
        // TODO Controls need to be handled differently
        ("NB.", VerbImpl::NotImplemented),
        ("{{", VerbImpl::NotImplemented),
        ("}}", VerbImpl::NotImplemented),
        ("assert.", VerbImpl::NotImplemented),
        ("break.", VerbImpl::NotImplemented),
        ("continue.", VerbImpl::NotImplemented),
        ("else.", VerbImpl::NotImplemented),
        ("elseif.", VerbImpl::NotImplemented),
        ("for.", VerbImpl::NotImplemented),
        ("for_ijk.", VerbImpl::NotImplemented), // TODO handle ijk label properly
        ("goto_lbl.", VerbImpl::NotImplemented), // TODO handle lbl properly
        ("label_lbl.", VerbImpl::NotImplemented), // TODO handle lbl properly
        ("if.", VerbImpl::NotImplemented),
        ("return.", VerbImpl::NotImplemented),
        ("select.", VerbImpl::NotImplemented),
        ("case.", VerbImpl::NotImplemented),
        ("fcase.", VerbImpl::NotImplemented),
        ("throw.", VerbImpl::NotImplemented),
        ("try.", VerbImpl::NotImplemented),
        ("catch.", VerbImpl::NotImplemented),
        ("catchd.", VerbImpl::NotImplemented),
        ("catcht.", VerbImpl::NotImplemented),
        ("while.", VerbImpl::NotImplemented),
        ("whilst.", VerbImpl::NotImplemented),
    ])
}

fn primitive_adverbs() -> &'static [&'static str] {
    // https://code.jsoftware.com/wiki/NuVoc
    &["~", "/", "/.", "\\", "\\.", "]:", "}", "b.", "f.", "M."]
}

fn primitive_nouns() -> &'static [&'static str] {
    // https://code.jsoftware.com/wiki/NuVoc
    &["_", "_.", "a.", "a:"]
}

fn primitive_conjunctions() -> &'static [&'static str] {
    // https://code.jsoftware.com/wiki/NuVoc
    &[
        "^:", ".", ":", ":.", "::", ";.", "!.", "!:", "[.", "].", "\"", "`", "`:", "@", "@.", "@:",
        "&", "&.", "&:", "&.:", "d.", "D.", "D:", "F.", "F..", "F.:", "F:", "F:.", "F::", "H.",
        "L:", "S:", "t.",
    ]
}

// TODO: https://code.jsoftware.com/wiki/Vocabulary/ErrorMessages
#[derive(Debug)]
pub struct JError {
    message: String,
}

pub fn scan(sentence: &str) -> Result<Vec<Word>, JError> {
    let mut words: Vec<Word> = Vec::new();

    let mut skip: usize = 0;

    //TODO recursive descent instead of a dumb loop.
    //TODO support multiline definitions.
    for (i, c) in sentence.chars().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        match c {
            '(' => {
                words.push(Word::LP);
            }
            ')' => {
                words.push(Word::RP);
            }
            c if c.is_whitespace() => (),
            '0'..='9' | '_' => {
                let (l, t) = scan_litnumarray(&sentence[i..])?;
                words.push(t);
                skip = l;
                continue;
            }
            '\'' => {
                let (l, t) = scan_litstring(&sentence[i..])?;
                words.push(t);
                skip = l;
                continue;
            }
            'a'..='z' | 'A'..='Z' => {
                let (l, t) = scan_name(&sentence[i..])?;
                words.push(t);
                skip = l;
                continue;
            }
            _ => {
                let (l, t) = scan_primitive(&sentence[i..])?;
                words.push(t);
                skip = l;
                continue;
            }
        }
    }
    Ok(words)
}

fn scan_litnumarray(sentence: &str) -> Result<(usize, Word), JError> {
    let mut l: usize = usize::MAX;
    if sentence.len() == 0 {
        return Err(JError {
            message: String::from("Empty number literal"),
        });
    }
    for (i, c) in sentence.chars().enumerate() {
        l = i;
        match c {
            '0'..='9' | '.' | '_' | 'e' | 'j' | 'r' | ' ' | '\t' => {
                () //still valid keep iterating
            }
            _ => {
                l -= 1;
                break;
            }
        }
    }

    // TODO - Fix - First hacky pass at this. Floats, ExtInt, Rationals, Complex
    let a = sentence[0..=l]
        .split_whitespace()
        .map(|s| s.replace("_", "-"))
        .map(|s| s.parse::<i64>())
        .collect::<Result<Vec<i64>, std::num::ParseIntError>>();
    match a {
        Ok(a) => match ArrayD::from_shape_vec(IxDyn(&[a.len()]), a) {
            Ok(v) => Ok((l, Word::Noun(IntArray { v }))),
            Err(e) => Err(JError {
                message: e.to_string(),
            }),
        },
        Err(e) => Err(JError {
            message: e.to_string(),
        }),
    }
}

fn scan_litstring(sentence: &str) -> Result<(usize, Word), JError> {
    if sentence.len() < 2 {
        return Err(JError {
            message: String::from("Empty literal string"),
        });
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
                    return Err(JError {
                        message: String::from("open quote"),
                    });
                }
            }
            _ => match prev_c_is_quote {
                true => {
                    //string closed previous char
                    l -= 1;
                    break;
                }
                false => {
                    () //still valid keep iterating
                }
            },
        }
    }
    let s = &sentence[1..l].replace("''", "'");
    Ok((l, char_array(s)))
}

fn scan_name(sentence: &str) -> Result<(usize, Word), JError> {
    // user defined adverbs/verbs/nouns
    let mut l: usize = usize::MAX;
    let mut p: Option<Word> = None;
    if sentence.len() == 0 {
        return Err(JError {
            message: String::from("Empty name"),
        });
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
                    None => match str_to_primitive(&sentence[0..=l]) {
                        Ok(w) => p = Some(w),
                        Err(_) => (),
                    },
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
        None => Ok((l, Word::Name(String::from(&sentence[0..=l])))),
    }
}

fn scan_primitive(sentence: &str) -> Result<(usize, Word), JError> {
    // built in adverbs/verbs
    let mut l: usize = 0;
    let mut p: Option<char> = None;
    //Primitives are 1 to 3 symbols:
    //  - one symbol
    //  - zero or more trailing . or : or both.
    //  - OR {{ }} for definitions
    if sentence.len() == 0 {
        return Err(JError {
            message: String::from("Empty primitive"),
        });
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
    Ok((l, str_to_primitive(&sentence.chars().take(l + 1).collect::<String>())?))
}

fn str_to_primitive(sentence: &str) -> Result<Word, JError> {
    if primitive_nouns().contains(&sentence) {
        Ok(char_array(sentence)) // TODO - actually lookup the noun
    } else if primitive_verbs().contains_key(&sentence) {
        let refd = match primitive_verbs().get(&sentence) {
            Some(v) => Some(v.clone()),
            None => None,
        };
        Ok(Word::Verb(String::from(sentence), refd))
    } else if primitive_adverbs().contains(&sentence) {
        Ok(Word::Adverb(String::from(sentence)))
    } else if primitive_conjunctions().contains(&sentence) {
        Ok(Word::Conjunction(String::from(sentence)))
    } else {
        return Err(JError {
            message: String::from("Invalid primitive"),
        });
    }
}

pub fn eval(sentence: Vec<Word>) -> Result<Word, JError> {
    //TODO: implement this properly
    //https://www.jsoftware.com/help/jforc/parsing_and_execution_ii.htm#_Toc191734586
    if sentence.len() == 3 {
        match &sentence[1] {
            Word::Verb(v, f) => match f {
                Some(f) => f.exec(Some(&sentence[0]), &sentence[2]),
                None => Err(JError {
                    message: String::from(v.to_owned() + "not supported yet"),
                }),
            },
            _ => Err(JError {
                message: String::from("not supported yet"),
            }),
        }
    } else {
        Err(JError {
            message: String::from("not supported yet"),
        })
    }
}

fn v_not_implemented<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    Err(JError {
        message: String::from("verb not implemented yet"),
    })
}

fn v_plus<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: String::from("monadic + not implemented yet"),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x + y }))
            } else {
                Err(JError {
                    message: String::from("plus not supported for these types yet"),
                })
            }
        }
    }
}

fn v_minus<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: String::from("monadic - not implemented yet"),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x - y }))
            } else {
                Err(JError {
                    message: String::from("minus not supported for these types yet"),
                })
            }
        }
    }
}

fn v_times<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: String::from("monadic * not implemented yet"),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x * y }))
            } else {
                Err(JError {
                    message: String::from("times not supported for these types yet"),
                })
            }
        }
    }
}
