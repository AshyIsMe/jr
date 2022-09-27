use std::collections::VecDeque;
use std::iter::repeat;

use itertools::Itertools;
use log::{debug, trace};

use crate::Word::{self, *};
use crate::{JError, ModifierImpl, VerbImpl};

pub fn eval(sentence: Vec<Word>) -> Result<Word, JError> {
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
            (ref w, Verb(_, v), Noun(y), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("0 monad");
                Ok(vec![fragment.0, v.exec(None, &Noun(y))?, any])
            }
            (ref w, Verb(us, ref u), Verb(_, ref v), Noun(y))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("1 monad");
                Ok(vec![
                    fragment.0,
                    Verb(us, u.clone()),
                    v.exec(None, &Noun(y))?,
                ])
            }
            (ref w, Noun(x), Verb(_, ref v), Noun(y))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("2 dyad");
                Ok(vec![fragment.0, v.exec(Some(&Noun(x)), &Noun(y))?])
            }
            // (V|N) A anything - 3 Adverb
            (ref w, Verb(sv, ref v), Adverb(sa, a), any)
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("3 adverb V A _");
                let verb_str = format!("{}{}", sv, sa);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Verb(sv, v.clone())),
                    r: Box::new(Nothing),
                    m: Box::new(Adverb(sa, a)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv), any])
            }
            (ref w, Noun(n), Adverb(sa, a), any)
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("3 adverb N A _");
                let verb_str = format!("m{}", sa);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Noun(n)),
                    r: Box::new(Nothing),
                    m: Box::new(Adverb(sa, a)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv), any])
            }
            //// (V|N) C (V|N) - 4 Conjunction
            (ref w, Verb(su, u), Conjunction(sc, c), Verb(sv, v))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("4 Conj V C V");
                let verb_str = format!("{}{}{}", su, sc, sv);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Verb(su, u.clone())),
                    r: Box::new(Verb(sv, v.clone())),
                    m: Box::new(Conjunction(sc, c)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv)])
            }
            (ref w, Verb(su, u), Conjunction(sc, c), Noun(n))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("4 Conj V C N");
                let verb_str = format!("{}{}", su, sc);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Verb(su, u.clone())),
                    r: Box::new(Noun(n)),
                    m: Box::new(Conjunction(sc, c)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv)])
            }
            (ref w, Noun(m), Conjunction(sc, c), Verb(sv, v))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("4 Conj N C V");
                let verb_str = format!("m{}{}", sc, sv);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Noun(m)),
                    r: Box::new(Verb(sv, v.clone())),
                    m: Box::new(Conjunction(sc, c)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv)])
            }
            (ref w, Noun(m), Conjunction(sc, c), Noun(n))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("4 Conj N C N");
                let verb_str = format!("m{}n", sc);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Noun(m)),
                    r: Box::new(Noun(n)),
                    m: Box::new(Conjunction(sc, c)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv)])
            }
            //// (V|N) V V - 5 Fork
            (ref w, Verb(sf, f), Verb(sg, g), Verb(sh, h))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("5 Fork V V V");
                let verb_str = format!("{}{}{}", sf, sg, sh);
                let fork = VerbImpl::Fork {
                    f: Box::new(Verb(sf, f.clone())),
                    g: Box::new(Verb(sh, h.clone())),
                    h: Box::new(Verb(sg, g.clone())),
                };
                Ok(vec![fragment.0, Verb(verb_str, fork)])
            }
            (ref w, Noun(m), Verb(sg, g), Verb(sh, h))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("5 Fork N V V");
                let verb_str = format!("n{}{}", sg, sh);
                let fork = VerbImpl::Fork {
                    f: Box::new(Noun(m)),
                    g: Box::new(Verb(sh, h.clone())),
                    h: Box::new(Verb(sg, g.clone())),
                };
                Ok(vec![fragment.0, Verb(verb_str, fork)])
            }
            // TODO: The new (old) modifier tridents and bidents:
            // https://code.jsoftware.com/wiki/Vocabulary/Parsing#The_Parsing_Table
            // https://code.jsoftware.com/wiki/Vocabulary/fork#invisiblemodifiers

            // TODO: Figure out how the rest of the hook combinations work.
            // (C|A|V|N) (C|A|V|N) anything - 6 Hook/Adverb
            // Only the combinations A A, C N, C V, N C, V C, and V V are valid;
            // the rest result in syntax errors.
            (ref w, Adverb(sa0, a0), Adverb(sa1, a1), _)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb A A _");
                let adverb_str = format!("{}{}", sa0, sa1);
                let hook = ModifierImpl::DerivedAdverb {
                    l: Box::new(Adverb(sa0, a0.clone())),
                    r: Box::new(Adverb(sa1, a1.clone())),
                };
                Ok(vec![fragment.0, Adverb(adverb_str, hook)])
            }
            //(w, Conjunction(c), Noun(m), _) => println!("6 Hook/Adverb C N _"),
            //(w, Conjunction(c), Verb(_, v), _) => println!("6 Hook/Adverb C V _"),
            //(w, Noun(n), Conjunction(d), _) => println!("6 Hook/Adverb N C _"),
            //(w, Verb(_, u), Conjunction(d), _) => println!("6 Hook/Adverb V C _"),
            (ref w, Verb(su, u), Verb(sv, v), _)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb V V _");
                let verb_str = format!("{}{}", su, sv);
                let hook = VerbImpl::Hook {
                    l: Box::new(Verb(su, u.clone())),
                    r: Box::new(Verb(sv, v.clone())),
                };
                Ok(vec![fragment.0, Verb(verb_str, hook)])
            }
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
            (w1, w2, w3, w4) => match queue.pop_back() {
                Some(v) => Ok(vec![v, w1, w2, w3, w4]),
                None => {
                    converged = true;
                    Ok(vec![w1, w2, w3, w4])
                }
            },
        };

        debug!("result: {:?}", result);
        stack = vec![result?, stack.into()].concat().into(); //push_front
    }
    trace!("DEBUG stack: {:?}", stack);
    let mut new_stack: VecDeque<Word> = stack
        .into_iter()
        .filter(|w| !matches!(w, StartOfLine))
        .filter(|w| !matches!(w, Nothing))
        .collect::<Vec<Word>>()
        .into();
    trace!("DEBUG new_stack: {:?}", new_stack);
    match new_stack.pop_front() {
        Some(val) if new_stack.is_empty() => Ok(val),
        _ => Err(JError::custom(
            "if you're happy and you know it, syntax error",
        )),
    }
}

fn get_fragment(stack: &mut VecDeque<Word>) -> (Word, Word, Word, Word) {
    stack
        .drain(..stack.len().min(4))
        .chain(repeat(Nothing))
        .next_tuple()
        .expect("infinite iterator can't be empty")
}
