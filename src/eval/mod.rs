mod controls;

use std::collections::VecDeque;
use std::iter::repeat;

use crate::{Ctx, Elem, JArray, Num};
use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use log::{debug, trace};
use num_traits::Zero;

use crate::error::JError;
// TODO: oh come on, this is clearly an eval concept
pub use crate::eval::controls::resolve_controls;
use crate::modifiers::ModifierImpl;
use crate::verbs::{v_open, VerbImpl};
use crate::Word::{self, *};

#[derive(Clone, Debug)]
pub struct Qs {
    queue: VecDeque<Word>,
    stack: VecDeque<Word>,
}

#[derive(Debug)]
pub enum EvalOutput {
    Regular(Word),
    Suspension,
    InDefinition,
}

impl EvalOutput {
    pub fn when_word(self) -> Result<Word> {
        match self {
            EvalOutput::Regular(word) => Ok(word),
            other => bail!("expecting a word but found {other:?}"),
        }
    }
}

pub fn feed(line: &str, ctx: &mut Ctx) -> Result<EvalOutput> {
    if ctx.is_suspended() {
        if line != ")" {
            ctx.input_push(line)?;
            return Ok(EvalOutput::Suspension);
        }
        return eval_suspendable(vec![], ctx);
    }
    let mut tokens = crate::scan(&format!("{}{line}", ctx.other_input_buffer))?;
    if !resolve_controls(&mut tokens)? {
        ctx.other_input_buffer.push_str(line);
        ctx.other_input_buffer.push('\n');
        return Ok(EvalOutput::InDefinition);
    }
    ctx.other_input_buffer.clear();
    debug!("tokens: {:?}", tokens);
    eval_suspendable(tokens, ctx).with_context(|| anyhow!("evaluating {:?}", line))
}

pub fn eval(sentence: Vec<Word>, ctx: &mut Ctx) -> Result<Word> {
    match eval_suspendable(sentence, ctx)? {
        EvalOutput::Regular(word) => Ok(word),
        EvalOutput::InDefinition | EvalOutput::Suspension => Err(JError::StackSuspension)
            .context("suspended in a context which doesn't support suspension"),
    }
}

pub fn eval_lines(sentence: &[Word], ctx: &mut Ctx) -> Result<Word> {
    // should not be returned?
    let mut word = Word::Nothing;
    for sentence in sentence
        .split(|w| matches!(w, Word::NewLine))
        .filter(|sub| !sub.is_empty())
    {
        word = eval(sentence.to_vec(), ctx)?;
    }
    Ok(word)
}

fn split_once(words: &[Word], f: impl Fn(&Word) -> bool) -> Option<(&[Word], &[Word])> {
    words
        .iter()
        .position(f)
        .map(|p| (&words[..p], &words[p + 1..]))
}

pub fn eval_suspendable(sentence: Vec<Word>, ctx: &mut Ctx) -> Result<EvalOutput> {
    if sentence
        .iter()
        .any(|w| w.is_control_symbol() || matches!(w, Word::NewLine))
    {
        bail!("invalid eval invocation: controls and newlines must have been eliminated: {sentence:?}");
    }
    // Attempt to parse j properly as per the documentation here:
    // https://www.jsoftware.com/ioj/iojSent.htm
    // https://www.jsoftware.com/help/jforc/parsing_and_execution_ii.htm#_Toc191734586

    let qs = if let Some(mut sus) = ctx.take_suspension() {
        assert!(
            sentence.is_empty(),
            "this function is called either with a suspended ctx *xor* a sentence"
        );

        debug!("restoring onto {:?}", sus.qs.stack);
        sus.qs
            .stack
            // after `mode :`
            .insert(2, Word::noun(sus.data.chars().collect_vec())?);
        sus.qs
    } else {
        let mut queue = VecDeque::from(sentence);
        queue.push_front(Word::StartOfLine);
        Qs {
            queue,
            stack: VecDeque::new(),
        }
    };

    let mut stack = qs.stack;
    let mut queue = qs.queue;
    debug!("starting eval: {stack:?} {queue:?}");

    let mut converged = false;
    // loop until queue is empty and stack has stopped changing
    while !converged {
        trace!("stack step: {:?}", stack);

        let fragment = get_fragment(&mut stack);
        trace!("fragment: {:?}", fragment);
        let fragment = resolve_names(fragment, ctx);

        let result: Result<Vec<Word>> = match fragment {
            (IfBlock(def), b, c, d) => {
                debug!("control if.");
                if def.iter().any(|w| matches!(w, Word::ElseIf)) {
                    return Err(JError::NonceError).context("no elsif.");
                }
                let (cond, follow) = split_once(&def, |w| matches!(w, Word::Do))
                    .ok_or(JError::SyntaxError)
                    .context("no do. in if.")?;
                let (tru, fal) = match split_once(follow, |w| matches!(w, Word::Else)) {
                    Some(x) => x,
                    None => (follow, [].as_slice()),
                };
                let cond = match eval(cond.to_vec(), ctx).context("if statement conditional")? {
                    Word::Noun(arr) => arr
                        .into_elems()
                        .into_iter()
                        .next()
                        .map(|e| e != Elem::Num(Num::zero()))
                        .unwrap_or_default(),
                    _ => {
                        return Err(JError::NounResultWasRequired)
                            .context("evaluating if conditional")
                    }
                };
                if cond {
                    let _ = eval_lines(tru, ctx).context("true block")?;
                } else if !fal.is_empty() {
                    let _ = eval_lines(fal, ctx).context("false block")?;
                }
                Ok(vec![b, c, d])
            }
            (ref w, Verb(_, v), Noun(y), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("0 monad");
                Ok(vec![
                    fragment.0,
                    v.exec(ctx, None, &Noun(y)).map(Word::Noun)?,
                    any,
                ])
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
                    v.exec(ctx, None, &Noun(y)).map(Word::Noun)?,
                ])
            }
            (ref w, Noun(x), Verb(_, ref v), Noun(y))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("2 dyad");
                Ok(vec![
                    fragment.0,
                    v.exec(ctx, Some(&Noun(x)), &Noun(y))
                        .context("evaluating 2 dyad")
                        .map(Word::Noun)?,
                ])
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
                let verb_str = format!("{} {} {}", su, sc, sv);
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
                let verb_str = format!("{} {}", su, sc);
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
                let verb_str = format!("m {} {}", sc, sv);
                let dv = VerbImpl::DerivedVerb {
                    l: Box::new(Noun(m)),
                    r: Box::new(Verb(sv, v.clone())),
                    m: Box::new(Conjunction(sc, c)),
                };
                Ok(vec![fragment.0, Verb(verb_str, dv)])
            }
            (ref w, Noun(ref m), Conjunction(ref sc, ref c), Noun(n))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("4 Conj N C N");
                if c.farcical(&m, &n)? {
                    queue.push_back(fragment.0);
                    stack.push_front(fragment.2);
                    stack.push_front(fragment.1);
                    debug!("suspending {queue:?} {stack:?}");
                    ctx.suspend(Qs { queue, stack })?;
                    return Ok(EvalOutput::Suspension);
                }
                // circumventing c_cor's implementation for nouns, but.. there's no way for a conj
                // to otherwise get "called", which is the observed behaviour; implicit call at eval
                // time
                let noun_hack = match (m.single_math_num(), sc.as_str()) {
                    (Some(n), ":") if n.is_zero() => true,
                    _ => false,
                };
                if noun_hack {
                    Ok(vec![fragment.0, Noun(n)])
                } else {
                    let verb_str = format!("m {} n", sc);
                    let dv = VerbImpl::DerivedVerb {
                        l: Box::new(Noun(m.clone())),
                        r: Box::new(Noun(n)),
                        m: Box::new(Conjunction(sc.clone(), c.clone())),
                    };
                    Ok(vec![fragment.0, Verb(verb_str, dv)])
                }
            }
            //// (V|N) V V - 5 Fork
            (ref w, Verb(sf, f), Verb(sg, g), Verb(sh, h))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_, _) | Verb(_, _) | Noun(_)
                ) =>
            {
                debug!("5 Fork V V V");
                let verb_str = format!("{} {} {}", sf, sg, sh);
                let fork = VerbImpl::Fork {
                    f: Box::new(Verb(sf, f.clone())),
                    g: Box::new(Verb(sg, g.clone())),
                    h: Box::new(Verb(sh, h.clone())),
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
                let verb_str = format!("n {} {}", sg, sh);
                let fork = VerbImpl::Fork {
                    f: Box::new(Noun(m)),
                    g: Box::new(Verb(sg, g.clone())),
                    h: Box::new(Verb(sh, h.clone())),
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
            (ref w, Adverb(sa0, a0), Adverb(sa1, a1), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb A A _");
                let adverb_str = format!("{} {}", sa0, sa1);
                let da = ModifierImpl::DerivedAdverb {
                    l: Box::new(Adverb(sa0, a0.clone())),
                    r: Box::new(Adverb(sa1, a1.clone())),
                };
                Ok(vec![fragment.0, Adverb(adverb_str, da), any])
            }
            (ref w, Conjunction(sc, c), Noun(n), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb C N _");
                let adverb_str = format!("{} n", sc);
                let da = ModifierImpl::DerivedAdverb {
                    l: Box::new(Conjunction(sc, c.clone())),
                    r: Box::new(Noun(n)),
                };
                Ok(vec![fragment.0, Adverb(adverb_str, da), any])
            }
            (ref w, Conjunction(sc, c), Verb(sv, v), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb C V _");
                let adverb_str = format!("{} {}", sc, sv);
                let da = ModifierImpl::DerivedAdverb {
                    l: Box::new(Conjunction(sc, c.clone())),
                    r: Box::new(Verb(sv, v.clone())),
                };
                Ok(vec![fragment.0, Adverb(adverb_str, da), any])
            }
            //(w, Noun(n), Conjunction(d), _) => println!("6 Hook/Adverb N C _"),
            //(w, Verb(_, u), Conjunction(d), _) => println!("6 Hook/Adverb V C _"),
            (ref w, Verb(su, u), Verb(sv, v), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb V V _");
                let verb_str = format!("{} {}", su, sv);
                let hook = VerbImpl::Hook {
                    l: Box::new(Verb(su, u.clone())),
                    r: Box::new(Verb(sv, v.clone())),
                };
                Ok(vec![fragment.0, Verb(verb_str, hook), any])
            }
            //(w, Verb(_, u), Adverb(b), _) => println!("SYNTAX ERROR 6 Hook/Adverb V A _"),
            //(w, Verb(_, u), Noun(m), _) => println!("SYNTAX ERROR 6 Hook/Adverb V N _"),
            //(w, Noun(n), Adverb(b), _) => println!("SYNTAX ERROR 6 Hook/Adverb N A _"),
            //(w, Noun(n), Verb(_, v), _) => println!("SYNTAX ERROR 6 Hook/Adverb N V _"),
            //(w, Noun(n), Noun(m), _) => println!("SYNTAX ERROR 6 Hook/Adverb N N _"),

            //// (Name|Noun) (IsLocal|IsGlobal) (C|A|V|N) anything - 7 Is
            (Name(n), IsLocal, w, any)
                if matches!(w, Conjunction(_, _) | Adverb(_, _) | Verb(_, _) | Noun(_)) =>
            {
                debug!("7 Is Local Name w");
                ctx.alias(n, w.clone());
                Ok(vec![w.clone(), any])
            }
            (Noun(names), IsLocal, w, any)
                if matches!(w, Conjunction(_, _) | Adverb(_, _) | Verb(_, _) | Noun(_)) =>
            {
                let Noun(arr) = w else { return Err(JError::NonceError).context("non-noun on the right of noun assignment"); };
                let JArray::CharArray(names) = names else { return Err(JError::NonceError).context("assigning to char arrays only please"); };
                if names.shape().len() != 1 {
                    return Err(JError::NonceError)
                        .context("lists of chars (strings), not multi-dimensional char arrays");
                }
                // presumably this is supposed to be "words"
                let names = names.iter().collect::<String>();
                let names = names.split_whitespace().collect_vec();
                if arr.shape()[0] != names.len() {
                    return Err(JError::LengthError).context("wrong number of names for an array");
                }

                for (name, val) in names.into_iter().zip(arr.outer_iter()) {
                    ctx.alias(name, Noun(v_open(&val.into_owned())?));
                }
                Ok(vec![any])
            }
            (Name(n), IsGlobal, w, any)
                if matches!(w, Conjunction(_, _) | Adverb(_, _) | Verb(_, _) | Noun(_)) =>
            {
                debug!("7 Is Local Name w");
                ctx.alias(n, w.clone());
                Ok(vec![any])
            }
            //// LP (C|A|V|N) RP anything - 8 Paren
            (LP, w, RP, any)
                if matches!(w, Conjunction(_, _) | Adverb(_, _) | Verb(_, _) | Noun(_)) =>
            {
                debug!("8 Paren");
                Ok(vec![w.clone(), any])
            }
            (w1, w2, w3, w4) => match queue.pop_back() {
                Some(v) => Ok(vec![v, w1, w2, w3, w4]),
                None => {
                    converged = true;
                    Ok(vec![w1, w2, w3, w4])
                }
            },
        };

        stack.retain(|w| !matches!(w, Nothing));

        debug!("result: {:?} with {stack:?}", result);
        stack = vec![result?, stack.into()].concat().into(); //push_front

        stack.retain(|w| !matches!(w, Nothing));
    }
    trace!("DEBUG stack: {:?}", stack);
    let mut new_stack: VecDeque<Word> = stack
        .into_iter()
        .filter(|w| !matches!(w, StartOfLine))
        .filter(|w| !matches!(w, Nothing))
        .collect::<Vec<Word>>()
        .into();
    debug!("finish eval: {:?}", new_stack);
    if new_stack.is_empty() {
        return Ok(EvalOutput::Regular(Word::Nothing));
    }
    match new_stack.pop_front() {
        Some(val) if new_stack.is_empty() => Ok(EvalOutput::Regular(val)),
        _ => Err(JError::SyntaxError)
            .with_context(|| anyhow!("expected a single output value but found {new_stack:?}")),
    }
}

fn get_fragment(stack: &mut VecDeque<Word>) -> (Word, Word, Word, Word) {
    stack
        .drain(..stack.len().min(4))
        .chain(repeat(Nothing))
        .next_tuple()
        .expect("infinite iterator can't be empty")
}

pub fn resolve_names(fragment: (Word, Word, Word, Word), ctx: &Ctx) -> (Word, Word, Word, Word) {
    let words = vec![
        fragment.0.clone(),
        fragment.1.clone(),
        fragment.2.clone(),
        fragment.3.clone(),
    ];

    // Resolve Names only on the RHS of IsLocal/IsGlobal
    let mut resolved_words = Vec::new();
    for w in words.iter().rev() {
        match w {
            IsGlobal => break,
            IsLocal => break,
            Name(ref n) => resolved_words.push(ctx.resolve(n).unwrap_or(&fragment.0).clone()),
            _ => resolved_words.push(w.clone()),
        }
    }
    resolved_words.reverse();

    let l = words.len() - resolved_words.len();
    let new_words = [&words[..l], &resolved_words[..]].concat();
    new_words.iter().cloned().collect_tuple().unwrap()
}
