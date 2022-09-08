mod adverbs;
mod verbs;

use itertools::Itertools;
use ndarray::prelude::*;
use std::collections::{HashMap, VecDeque};
//use std::fmt;

pub use crate::adverbs::*;
pub use crate::verbs::*;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary
// A Word is a part of speech.
#[derive(Clone, PartialEq, Debug)]
pub enum Word {
    LP,
    RP,
    StartOfLine, // used as placeholder when parsing
    Nothing,     // used as placeholder when parsing
    Name(String),

    IsLocal,
    IsGlobal,
    Noun(JArray),
    Verb(String, Box<VerbImpl>),
    Adverb(String, AdverbImpl),
    Conjunction(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum JArray {
    IntArray { a: ArrayD<i64> },
    ExtIntArray { a: ArrayD<i128> }, // TODO: num::bigint::BigInt
    FloatArray { a: ArrayD<f64> },
    BoolArray { a: ArrayD<u8> },
    CharArray { a: ArrayD<char> },
    //RationalArray { ... }, // TODO: num::rational::Rational64
    //ComplexArray { ... },  // TODO: num::complex::Complex64
    //EmptyArray // How do we do this properly?
}

use JArray::*;
use Word::*;

pub fn char_array(x: impl AsRef<str>) -> Word {
    let x = x.as_ref();
    Word::Noun(JArray::CharArray {
        a: ArrayD::from_shape_vec(IxDyn(&[x.chars().count()]), x.chars().collect()).unwrap(),
    })
}

pub fn int_array(v: Vec<i64>) -> Result<Word, JError> {
    Ok(Word::Noun(IntArray {
        a: Array::from_shape_vec(IxDyn(&[v.len()]), v).unwrap(),
    }))
}

////pub fn int_array(x: Vec<i64>) -> Word {
//pub fn int_array(x: Array1<i64>) -> Word {
//Word::Noun(JArray::IntArray {
////v: ArrayD::from_shape_vec(IxDyn(&[x.len()]), x).unwrap(),
//a: x.into_dyn(),
//})
//}

#[derive(Clone, Debug, PartialEq)]
pub enum VerbImpl {
    Plus,
    Minus,
    Times,
    Number,
    NotImplemented,

    DerivedVerb { v: Word, a: Word }, //Adverb modified Verb eg. +/
}

impl VerbImpl {
    fn exec(&self, x: Option<&Word>, y: &Word) -> Result<Word, JError> {
        match self {
            VerbImpl::Plus => v_plus(x, y),
            VerbImpl::Minus => v_minus(x, y),
            VerbImpl::Times => v_times(x, y),
            VerbImpl::Number => v_number(x, y),
            VerbImpl::NotImplemented => v_not_implemented(x, y),
            VerbImpl::DerivedVerb { v, a } => match (v, a) {
                (Verb(s, v), Adverb(_, a)) => a.exec(x, &Verb(s.to_string(), v.clone()), y),
                _ => panic!("invalid DerivedVerb {:?}", self),
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AdverbImpl {
    Insert,
    NotImplemented,
}

impl AdverbImpl {
    fn exec<'a>(&'a self, x: Option<&Word>, v: &Word, y: &Word) -> Result<Word, JError> {
        match self {
            AdverbImpl::Insert => a_insert(x, v, y),
            AdverbImpl::NotImplemented => a_not_implemented(x, v, y),
        }
    }
}

// TODO: https://code.jsoftware.com/wiki/Vocabulary/ErrorMessages
#[derive(Debug)]
pub struct JError {
    message: String,
}

//fn primitive_verbs() -> &'static [&'static str] {
fn primitive_verbs() -> HashMap<&'static str, VerbImpl> {
    HashMap::from([
        ("=", VerbImpl::NotImplemented),
        //("=.", VerbImpl::NotImplemented), IsLocal
        //("=:", VerbImpl::NotImplemented), IsGlobal
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
        ("#", VerbImpl::Number),
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

//fn primitive_adverbs() -> &'static [&'static str] {
//// https://code.jsoftware.com/wiki/NuVoc
//&["~", "/", "/.", "\\", "\\.", "]:", "}", "b.", "f.", "M."]
//}

fn primitive_adverbs() -> HashMap<&'static str, AdverbImpl> {
    HashMap::from([
        ("~", AdverbImpl::NotImplemented),
        ("/", AdverbImpl::Insert),
        ("/.", AdverbImpl::NotImplemented),
        ("\\", AdverbImpl::NotImplemented),
        ("\\.", AdverbImpl::NotImplemented),
        ("]:", AdverbImpl::NotImplemented),
        ("}", AdverbImpl::NotImplemented),
        ("b.", AdverbImpl::NotImplemented),
        ("f.", AdverbImpl::NotImplemented),
        ("M.", AdverbImpl::NotImplemented),
    ])
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

pub fn scan(sentence: &str) -> Result<Vec<Word>, JError> {
    let mut words: Vec<Word> = Vec::new();

    let mut skip: usize = 0;

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
            message: "Empty number literal".to_string(),
        });
    }
    // TODO - Fix - First hacky pass at this. Floats, ExtInt, Rationals, Complex
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

    if sentence[0..=l].contains('j') {
        Err(JError {
            message: "complex numbers not supported yet".to_string(),
        })
    } else if sentence[0..=l].contains('r') {
        Err(JError {
            message: "rational numbers not supported yet".to_string(),
        })
    } else if sentence[0..=l].contains('.') || sentence[0..=l].contains('e') {
        Err(JError {
            message: "floating point numbers not supported yet".to_string(),
        })
    } else {
        let a = sentence[0..=l]
            .split_whitespace()
            .map(|s| s.replace("_", "-"))
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<i64>, std::num::ParseIntError>>();
        match a {
            //Ok(a) => match ArrayD::from_shape_vec(IxDyn(&[a.len()]), a) {
            //Ok(v) => Ok((l, Word::Noun(IntArray { v }))),
            //Err(e) => Err(JError { message: e }),
            //},
            Ok(a) => Ok((
                l,
                Noun(IntArray {
                    a: ArrayD::from_shape_vec(IxDyn(&[a.len()]), a).unwrap(),
                }),
            )),
            Err(_) => Err(JError {
                message: "parse int error".to_string(),
            }),
        }
    }
}

fn scan_litstring(sentence: &str) -> Result<(usize, Word), JError> {
    if sentence.len() < 2 {
        return Err(JError {
            message: "Empty literal string".to_string(),
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
                        message: "open quote".to_string(),
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

    assert!(l <= sentence.chars().count(), "l past end of string: {}", l);
    let s = sentence
        .chars()
        .take(l)
        .skip(1)
        .collect::<String>()
        .replace("''", "'");
    Ok((l, char_array(&s)))
}

fn scan_name(sentence: &str) -> Result<(usize, Word), JError> {
    // user defined adverbs/verbs/nouns
    let mut l: usize = usize::MAX;
    let mut p: Option<Word> = None;
    if sentence.len() == 0 {
        return Err(JError {
            message: "Empty name".to_string(),
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
        None => Ok((l, Word::Name(sentence[0..=l].to_string()))),
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
            message: "Empty primitive".to_string(),
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
    Ok((l, str_to_primitive(&sentence[..=l])?))
}

fn str_to_primitive(sentence: &str) -> Result<Word, JError> {
    if primitive_nouns().contains(&sentence) {
        Ok(char_array(sentence)) // TODO - actually lookup the noun
    } else if primitive_verbs().contains_key(&sentence) {
        let refd = match primitive_verbs().get(&sentence) {
            Some(v) => v.clone(),
            None => VerbImpl::NotImplemented.clone(),
        };
        Ok(Word::Verb(sentence.to_string(), Box::new(refd)))
    } else if primitive_adverbs().contains_key(&sentence) {
        Ok(Word::Adverb(
            sentence.to_string(),
            match primitive_adverbs().get(&sentence) {
                Some(a) => a.clone(),
                None => AdverbImpl::NotImplemented.clone(),
            },
        ))
    } else if primitive_conjunctions().contains(&sentence) {
        Ok(Word::Conjunction(sentence.to_string()))
    } else {
        match sentence {
            "=:" => Ok(Word::IsGlobal),
            "=." => Ok(Word::IsLocal),
            _ => {
                return Err(JError {
                    message: "Invalid primitive".to_string(),
                })
            }
        }
    }
}

pub fn eval<'a>(sentence: Vec<Word>) -> Result<Word, JError> {
    // Attempt to parse j properly as per the documentation here:
    // https://www.jsoftware.com/ioj/iojSent.htm
    // https://www.jsoftware.com/help/jforc/parsing_and_execution_ii.htm#_Toc191734586

    let mut queue = VecDeque::from(sentence);
    queue.push_front(Word::StartOfLine);
    //let mut stack: VecDeque<Word> = VecDeque::from([]);
    let mut stack: VecDeque<Word> = [].into();

    // TODO: broken example: +/ 1 2 3
    // When the queue is empty we need to keep processing the stack until change stops
    while !queue.is_empty() {
        stack.push_front(queue.pop_back().unwrap());
        println!("stack step: {:?}", stack);

        let fragment = get_fragment(&mut stack);
        let result: Result<Vec<Word>, JError> = match fragment {
            (ref w, Verb(_, v), Noun(y), any) //monad
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                println!("0 monad");
                Ok(vec![fragment.0, v.exec(None, &Noun(y)).unwrap(), any])
            }
            (ref w, Verb(us, ref u), Verb(_, ref v), Noun(y)) //monad
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) =>
            {
                println!("1 monad");
                Ok(vec![
                    fragment.0,
                    Verb(us, u.clone()),
                    v.exec(None, &Noun(y)).unwrap(),
                ])
            }
            (ref w, Noun(x), Verb(_, ref v), Noun(y)) //dyad
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) =>
            {
                println!("2 dyad");
                Ok(vec![fragment.0, v.exec(Some(&Noun(x)), &Noun(y)).unwrap()])
            }
            // (V|N) A anything - 3 Adverb
            (ref w, Verb(sv, ref v), Adverb(sa, a), any) //adverb
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) => {
                    println!("3 adverb V A _");
                    //Ok(vec![fragment.0, a.exec(&Verb(sv,v)).unwrap(), any])
                    Ok(vec![fragment.0, Verb("".to_string(), Box::new(VerbImpl::DerivedVerb{v: Verb(sv,v.clone()), a: Adverb(sa,a)})), any])
                }
            // TODO:
            //(ref w, Noun(n), Adverb(sa,a), any) //adverb
                //if matches!(
                    //w,
                    //StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                //) => {
                    //println!("3 adverb N A _");
                    //Ok(vec![fragment.0, a.exec(&Noun(n)).unwrap(), any])
                //}
            //// (V|N) C (V|N) - 4 Conjunction
            //(w, Verb(_, u), Conjunction(a), Verb(_, v)) => println!("4 Conj V C V"),
            //(w, Verb(_, u), Conjunction(a), Noun(m)) => println!("4 Conj V C N"),
            //(w, Noun(n), Conjunction(a), Verb(_, v)) => println!("4 Conj N C V"),
            //(w, Noun(n), Conjunction(a), Noun(m)) => println!("4 Conj N C N"),
            //// (V|N) V V - 5 Fork
            //(w, Verb(_, f), Verb(_, g), Verb(_, h)) => println!("5 Fork V V V"),
            //(w, Noun(n), Verb(_, f), Verb(_, v)) => println!("5 Fork N V V"),
            //// (C|A|V|N) (C|A|V|N) anything - 6 Hook/Adverb
            //// Only the combinations A A, C N, C V, N C, V C, and V V are valid;
            //// the rest result in syntax errors.
            //(w, Adverb(a), Adverb(b), _) => println!("6 Hook/Adverb A A _"),
            //(w, Conjunction(c), Noun(m), _) => println!("6 Hook/Adverb C N _"),
            //(w, Conjunction(c), Verb(_, v), _) => println!("6 Hook/Adverb C V _"),
            //(w, Noun(n), Conjunction(d), _) => println!("6 Hook/Adverb N C _"),
            //(w, Verb(_, u), Conjunction(d), _) => println!("6 Hook/Adverb V C _"),
            //(w, Verb(_, u), Verb(_, v), _) => println!("6 Hook/Adverb V V _"),

            //(w, Verb(_, u), Adverb(b), _) => println!("SYNTAX ERROR 6 Hook/Adverb V A _"),
            //(w, Verb(_, u), Noun(m), _) => println!("SYNTAX ERROR 6 Hook/Adverb V N _"),
            //(w, Noun(n), Adverb(b), _) => println!("SYNTAX ERROR 6 Hook/Adverb N A _"),
            //(w, Noun(n), Verb(_, v), _) => println!("SYNTAX ERROR 6 Hook/Adverb N V _"),
            //(w, Noun(n), Noun(m), _) => println!("SYNTAX ERROR 6 Hook/Adverb N N _"),

            //// (Name|Noun) (IsLocal|IsGlobal) (C|A|V|N) anything - 7 Is
            //(Name(n), IsLocal, Conjunction(c), _) => println!("7 Is Local Name C"),
            //(Name(n), IsLocal, Adverb(a), _) => println!("7 Is Local Name A"),
            //(Name(n), IsLocal, Verb(_, v), _) => println!("7 Is Local Name V"),
            //(Name(n), IsLocal, Noun(m), _) => println!("7 Is Local Name N"),
            //(Noun(n), IsLocal, Conjunction(c), _) => println!("7 Is Local N C"),
            //(Noun(n), IsLocal, Adverb(a), _) => println!("7 Is Local N A"),
            //(Noun(n), IsLocal, Verb(_, v), _) => println!("7 Is Local N V"),
            //(Noun(n), IsLocal, Noun(m), _) => println!("7 Is Local N N"),

            //(Name(n), IsGlobal, Conjunction(c), _) => println!("7 Is Global Name C"),
            //(Name(n), IsGlobal, Adverb(a), _) => println!("7 Is Global Name A"),
            //(Name(n), IsGlobal, Verb(_, v), _) => println!("7 Is Global Name V"),
            //(Name(n), IsGlobal, Noun(m), _) => println!("7 Is Global Name N"),
            //(Noun(n), IsGlobal, Conjunction(c), _) => println!("7 Is Global N C"),
            //(Noun(n), IsGlobal, Adverb(a), _) => println!("7 Is Global N A"),
            //(Noun(n), IsGlobal, Verb(_, v), _) => println!("7 Is Global N V"),
            //(Noun(n), IsGlobal, Noun(m), _) => println!("7 Is Global N N"),

            //// LP (C|A|V|N) RP anything - 8 Paren
            //(LP, Conjunction(c), RP, _) => println!("8 Paren"),
            //(LP, Adverb(a), RP, _) => println!("8 Paren"),
            //(LP, Verb(_, v), RP, _) => println!("8 Paren"),
            //(LP, Noun(m), RP, _) => println!("8 Paren"),

            //_ => Err(JError { message: String::from("fragment doesn't match any execution cases"), }),
            _ => match fragment {
                (w1, ref w2, ref w3, w4) => Ok(vec![w1, w2.clone(), w3.clone(), w4]),
            },
        };

        //println!("result: {:?}", result);

        if let Ok(r) = result {
            //stack.push_front(r);
            stack = vec![r, stack.into()].concat().into();
        } else if let Err(e) = result {
            return Err(e);
        }
    }
    println!("stack: {:?}", stack);
    let mut new_stack: VecDeque<Word> = stack
        .into_iter()
        .filter(|w| if let StartOfLine = w { false } else { true })
        .filter(|w| if let Nothing = w { false } else { true })
        .collect::<Vec<Word>>()
        .into();
    println!("new_stack: {:?}", new_stack);
    match new_stack.len() {
        1 => Ok(new_stack.pop_front().unwrap().clone()),
        _ => Err(JError {
            message: "if you're happy and you know it, syntax error".to_string(),
        }),
    }
}

fn get_fragment<'a, 'b>(stack: &'b mut VecDeque<Word>) -> (Word, Word, Word, Word) {
    match stack.len() {
        0 => (Nothing, Nothing, Nothing, Nothing),
        1 => (stack.pop_front().unwrap(), Nothing, Nothing, Nothing),
        2 => (
            stack.pop_front().unwrap(),
            stack.pop_front().unwrap(),
            Nothing,
            Nothing,
        ),
        3 => (
            stack.pop_front().unwrap(),
            stack.pop_front().unwrap(),
            stack.pop_front().unwrap(),
            Nothing,
        ),
        _ => stack.drain(..4).collect_tuple().unwrap(),
    }
}
