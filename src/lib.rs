use ndarray::prelude::*;
use std::collections::HashMap;
use std::fmt;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary
// A Word is a part of speech.
#[derive(Clone)]
pub enum Word {
    LP,
    RP,
    Name(String),

    Noun(JArray),
    Verb(String, Option<Box<Verbfn>>),
    Adverb(String),
    Conjunction(String),
}

impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                LP => "LP",
                _ => "whatevs",
            }
        )
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        unimplemented!()
    }
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

type Verbfn = for<'x, 'y> fn(Option<&'x Word>, &'y Word) -> Result<Word, JError>;

use JArray::*;

pub fn char_array(x: impl AsRef<str>) -> Word {
    let x = x.as_ref();
    Word::Noun(JArray::CharArray {
        v: ArrayD::from_shape_vec(IxDyn(&[x.len()]), String::from(x).chars().collect()).unwrap(),
    })
}

//fn primitive_verbs() -> &'static [&'static str] {
fn primitive_verbs() -> HashMap<&'static str, Box<Verbfn>> {
    HashMap::from([
        ("=", Box::new(v_not_implemented as Verbfn)),
        ("=.", Box::new(v_not_implemented)),
        ("=:", Box::new(v_not_implemented)),
        ("<", Box::new(v_not_implemented)),
        ("<.", Box::new(v_not_implemented)),
        ("<:", Box::new(v_not_implemented)),
        (">", Box::new(v_not_implemented)),
        (">.", Box::new(v_not_implemented)),
        (">:", Box::new(v_not_implemented)),
        ("_:", Box::new(v_not_implemented)),
        ("+", Box::new(v_plus as Verbfn)),
        ("+.", Box::new(v_not_implemented)),
        ("+:", Box::new(v_not_implemented)),
        ("*", Box::new(v_not_implemented)),
        ("*.", Box::new(v_not_implemented)),
        ("*:", Box::new(v_not_implemented)),
        ("-", Box::new(v_not_implemented)),
        ("-.", Box::new(v_not_implemented)),
        ("-:", Box::new(v_not_implemented)),
        ("%", Box::new(v_not_implemented)),
        ("%.", Box::new(v_not_implemented)),
        ("%:", Box::new(v_not_implemented)),
        ("^", Box::new(v_not_implemented)),
        ("^.", Box::new(v_not_implemented)),
        ("^!.", Box::new(v_not_implemented)),
        ("$", Box::new(v_not_implemented)),
        ("$.", Box::new(v_not_implemented)),
        ("$:", Box::new(v_not_implemented)),
        ("~.", Box::new(v_not_implemented)),
        ("~:", Box::new(v_not_implemented)),
        ("|", Box::new(v_not_implemented)),
        ("|.", Box::new(v_not_implemented)),
        ("|:", Box::new(v_not_implemented)),
        (".:", Box::new(v_not_implemented)),
        ("..", Box::new(v_not_implemented)),
        (",.", Box::new(v_not_implemented)),
        (",", Box::new(v_not_implemented)),
        (",:", Box::new(v_not_implemented)),
        (";", Box::new(v_not_implemented)),
        (";:", Box::new(v_not_implemented)),
        ("#", Box::new(v_not_implemented)),
        ("#.", Box::new(v_not_implemented)),
        ("#:", Box::new(v_not_implemented)),
        ("!", Box::new(v_not_implemented)),
        ("/:", Box::new(v_not_implemented)),
        ("\\:", Box::new(v_not_implemented)),
        ("[", Box::new(v_not_implemented)),
        ("[:", Box::new(v_not_implemented)),
        ("]", Box::new(v_not_implemented)),
        ("{", Box::new(v_not_implemented)),
        ("{.", Box::new(v_not_implemented)),
        ("{:", Box::new(v_not_implemented)),
        ("{::", Box::new(v_not_implemented)),
        ("}.", Box::new(v_not_implemented)),
        ("}:", Box::new(v_not_implemented)),
        ("\".", Box::new(v_not_implemented)),
        ("\":", Box::new(v_not_implemented)),
        ("?", Box::new(v_not_implemented)),
        ("?.", Box::new(v_not_implemented)),
        ("A.", Box::new(v_not_implemented)),
        ("C.", Box::new(v_not_implemented)),
        ("C.!.2", Box::new(v_not_implemented)),
        ("e.", Box::new(v_not_implemented)),
        ("E.", Box::new(v_not_implemented)),
        ("i.", Box::new(v_not_implemented)),
        ("i:", Box::new(v_not_implemented)),
        ("I.", Box::new(v_not_implemented)),
        ("j.", Box::new(v_not_implemented)),
        ("L.", Box::new(v_not_implemented)),
        ("o.", Box::new(v_not_implemented)),
        ("p.", Box::new(v_not_implemented)),
        ("p..", Box::new(v_not_implemented)),
        ("p:", Box::new(v_not_implemented)),
        ("q:", Box::new(v_not_implemented)),
        ("r.", Box::new(v_not_implemented)),
        ("s:", Box::new(v_not_implemented)),
        ("T.", Box::new(v_not_implemented)),
        ("u:", Box::new(v_not_implemented)),
        ("x:", Box::new(v_not_implemented)),
        ("Z:", Box::new(v_not_implemented)),
        ("_9:", Box::new(v_not_implemented)),
        ("_8:", Box::new(v_not_implemented)),
        ("_7:", Box::new(v_not_implemented)),
        ("_6:", Box::new(v_not_implemented)),
        ("_5:", Box::new(v_not_implemented)),
        ("_4:", Box::new(v_not_implemented)),
        ("_3:", Box::new(v_not_implemented)),
        ("_2:", Box::new(v_not_implemented)),
        ("_1:", Box::new(v_not_implemented)),
        ("0:", Box::new(v_not_implemented)),
        ("1:", Box::new(v_not_implemented)),
        ("2:", Box::new(v_not_implemented)),
        ("3:", Box::new(v_not_implemented)),
        ("4:", Box::new(v_not_implemented)),
        ("5:", Box::new(v_not_implemented)),
        ("6:", Box::new(v_not_implemented)),
        ("7:", Box::new(v_not_implemented)),
        ("8:", Box::new(v_not_implemented)),
        ("9", Box::new(v_not_implemented)),
        ("u.", Box::new(v_not_implemented)),
        ("v.", Box::new(v_not_implemented)),
        // TODO Controls need to be handled differently
        ("NB.", Box::new(v_not_implemented)),
        ("{{", Box::new(v_not_implemented)),
        ("}}", Box::new(v_not_implemented)),
        ("assert.", Box::new(v_not_implemented)),
        ("break.", Box::new(v_not_implemented)),
        ("continue.", Box::new(v_not_implemented)),
        ("else.", Box::new(v_not_implemented)),
        ("elseif.", Box::new(v_not_implemented)),
        ("for.", Box::new(v_not_implemented)),
        ("for_ijk.", Box::new(v_not_implemented)), // TODO handle ijk label properly
        ("goto_lbl.", Box::new(v_not_implemented)), // TODO handle lbl properly
        ("label_lbl.", Box::new(v_not_implemented)), // TODO handle lbl properly
        ("if.", Box::new(v_not_implemented)),
        ("return.", Box::new(v_not_implemented)),
        ("select.", Box::new(v_not_implemented)),
        ("case.", Box::new(v_not_implemented)),
        ("fcase.", Box::new(v_not_implemented)),
        ("throw.", Box::new(v_not_implemented)),
        ("try.", Box::new(v_not_implemented)),
        ("catch.", Box::new(v_not_implemented)),
        ("catchd.", Box::new(v_not_implemented)),
        ("catcht.", Box::new(v_not_implemented)),
        ("while.", Box::new(v_not_implemented)),
        ("whilst.", Box::new(v_not_implemented)),
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
    Ok((l, str_to_primitive(&sentence[0..=l])?))
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
                Some(f) => f(Some(&sentence[0]), &sentence[2]),
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
    //Clearly this isn't gonna scale... figure out a dispatch table or something

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
