use std::collections::VecDeque;
use std::iter::repeat;

use itertools::Itertools;
use log::{debug, trace};

use crate::Word::{self, *};
use crate::{JError, VerbImpl};

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
                let conc = format!("{}{}", sv, sa);
                let dv = VerbImpl::DerivedVerb {
                    u: Box::new(Verb(sv, v.clone())),
                    m: Box::new(Nothing),
                    a: Box::new(Adverb(sa, a)),
                };
                Ok(vec![fragment.0, Verb(conc, dv), any])
            }
            (ref w, Noun(n), Adverb(sa, a), any)
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("3 adverb N A _");
                let conc = format!("m{}", sa);
                let dv = VerbImpl::DerivedVerb {
                    u: Box::new(Nothing),
                    m: Box::new(Noun(n)),
                    a: Box::new(Adverb(sa, a)),
                };
                Ok(vec![fragment.0, Verb(conc, dv), any])
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
            (w1, w2, w3, w4) if queue.is_empty() => {
                converged = true;
                Ok(vec![w1, w2, w3, w4])
            }
            (w1, w2, w3, w4) => Ok(vec![queue.pop_back().unwrap(), w1, w2, w3, w4]),
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
    match new_stack.len() {
        1 => Ok(new_stack.pop_front().unwrap()),
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
