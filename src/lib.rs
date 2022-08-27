use ndarray::prelude::*;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary
// A Word is a part of speech.
#[derive(Debug, PartialEq)]
pub enum Word {
    LP,
    RP,
    Name(String),

    Noun(String),
    Verb(String),
    Adverb(String),
    Conjunction(String),

    LitNumArray(String), // collapse these into Noun?
    LitString(String),

    IntArray { v: ArrayD<i64> },
    ExtIntArray { v: ArrayD<i128> }, // TODO: real arbitrary bignum support
    FloatArray { v: ArrayD<f64> },
    BoolArray { v: ArrayD<u8> },
    CharArray { s: Array1<u8>, v: String },
    //RationalArray { ... }, // TODO
    //ComplexArray { ... },  // TODO
    //EmptyArray // How do we do this properly?
}

#[rustfmt::skip]
fn primitive_verbs() -> Vec<String> {
    // https://code.jsoftware.com/wiki/NuVoc
    vec![
    "=","=.","=:",
    "<","<.","<:",
    ">",">.",">:",
    "_:",

    "+","+.","+:",
    "*","*.","*:",
    "-","-.","-:",
    "%","%.","%:",

    "^","^.",
    "^!.",
    "$","$.","$:",
    "~.","~:",
    "|","|.","|:",

    ".:",
    "..",

    ",.",",",",:",
    ";",
    ";:",

    "#","#.","#:",
    "!",
    "/:",
    "\\:",

    "[",
    "[:",
    "]",
    "{","{.","{:","{::",
    "}.","}:",

    "\".","\":",
    "?","?.",

    "A.",
    "C.","C.!.2",
    "e.",
    "E.",

    "i.","i:",
    "I.","j.","L.",
    "o.","p.","p..",

    "p:","q:","r.",
    "s:",
    "T.","u:","x:",
    "Z:",
    "_9:","_8:","_7:","_6:","_5:","_4:","_3:","_2:","_1:","0:","1:","2:","3:","4:","5:","6:","7:","8:","9",
    "u.","v.",

    // TODO Controls need to be handled differently
    "NB.",
    "{{","}}",
    "assert.", "break.", "continue.",
    "else.", "elseif.", "for.",
    "for_ijk.",   // TODO handle ijk label properly
    "goto_lbl.",  // TODO handle lbl properly
    "label_lbl.", // TODO handle lbl properly
    "if.", "return.", "select.", "case.", "fcase.",
    "throw.", "try.", "catch.", "catchd.", "catcht.",
    "while.", "whilst.",
    ].into_iter().map(String::from).collect()
}
fn primitive_adverbs() -> Vec<String> {
    // https://code.jsoftware.com/wiki/NuVoc
    vec!["~", "/", "/.", "\\", "\\.", "]:", "}", "b.", "f.", "M."]
        .into_iter()
        .map(String::from)
        .collect()
}

fn primitive_nouns() -> Vec<String> {
    // https://code.jsoftware.com/wiki/NuVoc
    vec!["_", "_.", "a.", "a:"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn primitive_conjunctions() -> Vec<String> {
    // https://code.jsoftware.com/wiki/NuVoc
    vec![
        "^:", ".", ":", ":.", "::", ";.", "!.", "!:", "[.", "].", "\"", "`", "`:", "@", "@.", "@:",
        "&", "&.", "&:", "&.:", "d.", "D.", "D:", "F.", "F..", "F.:", "F:", "F:.", "F::", "H.",
        "L:", "S:", "t.",
    ]
    .into_iter()
    .map(String::from)
    .collect()
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
            Ok(v) => Ok((l, Word::IntArray { v })),
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
    Ok((
        l,
        Word::LitString(String::from(&sentence[1..l]).replace("''", "'")),
    ))
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
    if primitive_nouns().contains(&String::from(sentence)) {
        Ok(Word::Noun(String::from(sentence)))
    } else if primitive_verbs().contains(&String::from(sentence)) {
        Ok(Word::Verb(String::from(sentence)))
    } else if primitive_adverbs().contains(&String::from(sentence)) {
        Ok(Word::Adverb(String::from(sentence)))
    } else if primitive_conjunctions().contains(&String::from(sentence)) {
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
    if sentence.len() == 2 {
        v_d_plus(sentence[0], sentence[1])
    } else {
        Err(JError {
            message: String::from("not supported yet"),
        })
    }
}

fn v_d_plus(x: Word, y: Word) -> Result<Word, JError> {
    //Clearly this isn't gonna scale... figure out a dispatch table or something

    // How do I match or `if let` for enum variants?
    if let Word::IntArray(x) = x {
        if let Word::IntArray(y) = y {
            println!("int plus!");
            let r = x + y;
            match r {
                Ok(v) => Ok(Word::IntArray { v }),
                Err(e) => Err(JError {
                    message: e.to_string(),
                }),
            }
        } else {
            Err(JError {
                message: String::from("plus not supported for these types yet"),
            })
        }
    } else {
        Err(JError {
            message: String::from("plus not supported for these types yet"),
        })
    }
}
