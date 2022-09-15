pub mod adverbs;
pub mod arrays;
pub mod scan;
pub mod verbs;

use itertools::Itertools;
use log::{debug, trace};
use std::collections::{HashMap, VecDeque};

pub use crate::adverbs::*;
pub use crate::arrays::*;
pub use crate::scan::*;
pub use crate::verbs::*;

use Word::*;

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
        ("$", VerbImpl::Dollar),
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

fn primitive_adverbs() -> HashMap<&'static str, AdverbImpl> {
    HashMap::from([
        ("~", AdverbImpl::NotImplemented),
        ("/", AdverbImpl::Slash),
        ("/.", AdverbImpl::NotImplemented),
        ("\\", AdverbImpl::NotImplemented),
        ("\\.", AdverbImpl::NotImplemented),
        ("]:", AdverbImpl::NotImplemented),
        ("}", AdverbImpl::CurlyRt),
        ("b.", AdverbImpl::NotImplemented),
        ("f.", AdverbImpl::NotImplemented),
        ("M.", AdverbImpl::NotImplemented),
    ])
}

fn primitive_nouns() -> &'static [&'static str] {
    // TODO
    // https://code.jsoftware.com/wiki/NuVoc
    &["_", "_.", "a.", "a:"]
}

fn primitive_conjunctions() -> &'static [&'static str] {
    // TODO
    // https://code.jsoftware.com/wiki/NuVoc
    &[
        "^:", ".", ":", ":.", "::", ";.", "!.", "!:", "[.", "].", "\"", "`", "`:", "@", "@.", "@:",
        "&", "&.", "&:", "&.:", "d.", "D.", "D:", "F.", "F..", "F.:", "F:", "F:.", "F::", "H.",
        "L:", "S:", "t.",
    ]
}

pub fn eval<'a>(sentence: Vec<Word>) -> Result<Word, JError> {
    // Attempt to parse j properly as per the documentation here:
    // https://www.jsoftware.com/ioj/iojSent.htm
    // https://www.jsoftware.com/help/jforc/parsing_and_execution_ii.htm#_Toc191734586

    let mut queue = VecDeque::from(sentence);
    queue.push_front(Word::StartOfLine);
    let mut stack: VecDeque<Word> = [].into();

    let mut converged = false;
    // loop until queue is empty and stack has stopped changing
    while !converged {
        trace!("stack step: {:?}", stack);

        let fragment = get_fragment(&mut stack);
        trace!("fragment: {:?}", fragment);
        let result: Result<Vec<Word>, JError> = match fragment {
            (ref w, Verb(_, v), Noun(y), any) //monad
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("0 monad");
                Ok(vec![fragment.0, v.exec(None, &Noun(y))?, any])
            }
            (ref w, Verb(us, ref u), Verb(_, ref v), Noun(y)) //monad
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("1 monad");
                Ok(vec![
                    fragment.0,
                    Verb(us, u.clone()),
                    v.exec(None, &Noun(y))?,
                ])
            }
            (ref w, Noun(x), Verb(_, ref v), Noun(y)) //dyad
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("2 dyad");
                Ok(vec![fragment.0, v.exec(Some(&Noun(x)), &Noun(y))?])
            }
            // (V|N) A anything - 3 Adverb
            (ref w, Verb(sv, ref v), Adverb(sa, a), any) //adverb
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) => {
                    debug!("3 adverb V A _");
                    Ok(vec![fragment.0, Verb(format!("{}{}",sv,sa), Box::new(VerbImpl::DerivedVerb{u: Verb(sv,v.clone()), m: Nothing, a: Adverb(sa,a)})), any])
                }
            (ref w, Noun(n), Adverb(sa,a), any) //adverb
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_,_) | Verb(_, _) | Noun(_)
                ) => {
                    debug!("3 adverb N A _");
                    Ok(vec![fragment.0, Verb(format!("m{}",sa), Box::new(VerbImpl::DerivedVerb{u: Nothing, m: Noun(n), a: Adverb(sa,a)})), any])
                }
            // TODO:
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

            _ => match fragment {
                (w1, w2, w3, w4) => if queue.is_empty() {
                    converged = true;
                    Ok(vec![w1, w2, w3, w4])
                } else {
                    Ok(vec![queue.pop_back().unwrap(), w1, w2, w3, w4])
                }
            },
        };

        debug!("result: {:?}", result);

        if let Ok(r) = result {
            stack = vec![r, stack.into()].concat().into(); //push_front
        } else if let Err(e) = result {
            return Err(e);
        }
    }
    trace!("DEBUG stack: {:?}", stack);
    let mut new_stack: VecDeque<Word> = stack
        .into_iter()
        .filter(|w| if let StartOfLine = w { false } else { true })
        .filter(|w| if let Nothing = w { false } else { true })
        .collect::<Vec<Word>>()
        .into();
    trace!("DEBUG new_stack: {:?}", new_stack);
    match new_stack.len() {
        1 => Ok(new_stack.pop_front().unwrap().clone()),
        _ => Err(JError::custom(
            "if you're happy and you know it, syntax error",
        )),
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
