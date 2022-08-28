use ndarray::prelude::*;
use std::collections::HashMap;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary
// A Word is a part of speech.
#[derive(Clone, Debug, PartialEq)]
pub enum Word<'a> {
    LP,
    RP,
    Name(String),

    Noun(JArray),
    Verb(String, Option<&'a Verbfn>),
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

type Verbfn = for<'a> fn(Option<&Word>, &'a Word) -> Result<Word<'a>, JError>;

use JArray::*;

pub fn char_array(x: impl AsRef<str>) -> Word<'static> {
    let x = x.as_ref();
    Word::Noun(JArray::CharArray {
        v: ArrayD::from_shape_vec(IxDyn(&[x.len()]), String::from(x).chars().collect()).unwrap(),
    })
}

//fn primitive_verbs() -> &'static [&'static str] {
fn primitive_verbs() -> HashMap<&'static str, Verbfn> {
    HashMap::from([
        ("=", v_not_implemented),
        ("=.", v_not_implemented),
        ("=:", v_not_implemented),
        ("<", v_not_implemented),
        ("<.", v_not_implemented),
        ("<:", v_not_implemented),
        (">", v_not_implemented),
        (">.", v_not_implemented),
        (">:", v_not_implemented),
        ("_:", v_not_implemented),
        ("+", v_plus),
        ("+.", v_not_implemented),
        ("+:", v_not_implemented),
        ("*", v_not_implemented),
        ("*.", v_not_implemented),
        ("*:", v_not_implemented),
        ("-", v_not_implemented),
        ("-.", v_not_implemented),
        ("-:", v_not_implemented),
        ("%", v_not_implemented),
        ("%.", v_not_implemented),
        ("%:", v_not_implemented),
        ("^", v_not_implemented),
        ("^.", v_not_implemented),
        ("^!.", v_not_implemented),
        ("$", v_not_implemented),
        ("$.", v_not_implemented),
        ("$:", v_not_implemented),
        ("~.", v_not_implemented),
        ("~:", v_not_implemented),
        ("|", v_not_implemented),
        ("|.", v_not_implemented),
        ("|:", v_not_implemented),
        (".:", v_not_implemented),
        ("..", v_not_implemented),
        (",.", v_not_implemented),
        (",", v_not_implemented),
        (",:", v_not_implemented),
        (";", v_not_implemented),
        (";:", v_not_implemented),
        ("#", v_not_implemented),
        ("#.", v_not_implemented),
        ("#:", v_not_implemented),
        ("!", v_not_implemented),
        ("/:", v_not_implemented),
        ("\\:", v_not_implemented),
        ("[", v_not_implemented),
        ("[:", v_not_implemented),
        ("]", v_not_implemented),
        ("{", v_not_implemented),
        ("{.", v_not_implemented),
        ("{:", v_not_implemented),
        ("{::", v_not_implemented),
        ("}.", v_not_implemented),
        ("}:", v_not_implemented),
        ("\".", v_not_implemented),
        ("\":", v_not_implemented),
        ("?", v_not_implemented),
        ("?.", v_not_implemented),
        ("A.", v_not_implemented),
        ("C.", v_not_implemented),
        ("C.!.2", v_not_implemented),
        ("e.", v_not_implemented),
        ("E.", v_not_implemented),
        ("i.", v_not_implemented),
        ("i:", v_not_implemented),
        ("I.", v_not_implemented),
        ("j.", v_not_implemented),
        ("L.", v_not_implemented),
        ("o.", v_not_implemented),
        ("p.", v_not_implemented),
        ("p..", v_not_implemented),
        ("p:", v_not_implemented),
        ("q:", v_not_implemented),
        ("r.", v_not_implemented),
        ("s:", v_not_implemented),
        ("T.", v_not_implemented),
        ("u:", v_not_implemented),
        ("x:", v_not_implemented),
        ("Z:", v_not_implemented),
        ("_9:", v_not_implemented),
        ("_8:", v_not_implemented),
        ("_7:", v_not_implemented),
        ("_6:", v_not_implemented),
        ("_5:", v_not_implemented),
        ("_4:", v_not_implemented),
        ("_3:", v_not_implemented),
        ("_2:", v_not_implemented),
        ("_1:", v_not_implemented),
        ("0:", v_not_implemented),
        ("1:", v_not_implemented),
        ("2:", v_not_implemented),
        ("3:", v_not_implemented),
        ("4:", v_not_implemented),
        ("5:", v_not_implemented),
        ("6:", v_not_implemented),
        ("7:", v_not_implemented),
        ("8:", v_not_implemented),
        ("9", v_not_implemented),
        ("u.", v_not_implemented),
        ("v.", v_not_implemented),
        // TODO Controls need to be handled differently
        ("NB.", v_not_implemented),
        ("{{", v_not_implemented),
        ("}}", v_not_implemented),
        ("assert.", v_not_implemented),
        ("break.", v_not_implemented),
        ("continue.", v_not_implemented),
        ("else.", v_not_implemented),
        ("elseif.", v_not_implemented),
        ("for.", v_not_implemented),
        ("for_ijk.", v_not_implemented), // TODO handle ijk label properly
        ("goto_lbl.", v_not_implemented), // TODO handle lbl properly
        ("label_lbl.", v_not_implemented), // TODO handle lbl properly
        ("if.", v_not_implemented),
        ("return.", v_not_implemented),
        ("select.", v_not_implemented),
        ("case.", v_not_implemented),
        ("fcase.", v_not_implemented),
        ("throw.", v_not_implemented),
        ("try.", v_not_implemented),
        ("catch.", v_not_implemented),
        ("catchd.", v_not_implemented),
        ("catcht.", v_not_implemented),
        ("while.", v_not_implemented),
        ("whilst.", v_not_implemented),
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
        Ok(Word::Verb(String::from(sentence), primitive_verbs().get(&sentence)))
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
            Word::Verb(v, f) => {
                match f {
                  Some(f) => f(Some(&sentence[0]), &sentence[2]),
                  None => Err(JError { message: String::from(v.to_owned() + "not supported yet")})
                }
            }
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

fn v_not_implemented<'a>(x: Option<&Word>, y: &'a Word) -> Result<Word<'a>, JError> {
    Err(JError {
        message: String::from("verb not implemented yet"),
    })
}

fn v_plus<'a>(x: Option<&Word>, y: &'a Word) -> Result<Word<'a>, JError> {
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
