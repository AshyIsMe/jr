mod controls;
mod ctl_if;

use std::collections::VecDeque;
use std::iter::repeat;

use crate::{arr0d, Ctx, JArray};
use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use log::{debug, trace};

use crate::error::JError;
// TODO: oh come on, this is clearly an eval concept
pub use crate::eval::controls::create_def;
// TODO: oh come on, this is clearly an eval concept
pub use crate::eval::controls::resolve_controls;

use crate::eval::ctl_if::control_if;
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
    let buf = ctx
        .input_buffers
        .as_mut()
        .ok_or(JError::ControlError)
        .context("non-root context at feed")?;

    if buf.is_suspended() {
        if line != ")" {
            buf.input_push(line)?;
            return Ok(EvalOutput::Suspension);
        }
        return eval_suspendable(vec![], ctx);
    }
    let mut tokens = crate::scan(&format!("{}{line}", buf.other_input_buffer))?;
    if !resolve_controls(&mut tokens)? {
        buf.other_input_buffer.push_str(line);
        buf.other_input_buffer.push('\n');
        return Ok(EvalOutput::InDefinition);
    }
    buf.other_input_buffer.clear();
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
    for (rel_pos, sentence) in sentence
        .split(|w| matches!(w, Word::NewLine))
        .enumerate()
        .filter(|(_, sub)| !sub.is_empty())
    {
        word = eval(sentence.to_vec(), ctx)
            .with_context(|| anyhow!("evaluating line {} *of block*: {sentence:?}", rel_pos + 1))?;
    }
    Ok(word)
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

    let qs = if let Some(mut sus) = ctx.input_buffers.as_mut().and_then(|b| b.take_suspension()) {
        assert!(
            sentence.is_empty(),
            "this function is called either with a suspended ctx *xor* a sentence"
        );

        debug!("restoring onto {:?}", sus.qs.stack);
        sus.qs
            .stack
            // after `mode :`
            .insert(2, Word::Noun(JArray::from_string(sus.data)));
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
        let fragment = resolve_names(fragment, ctx)?;

        let result: Result<Vec<Word>> = match fragment {
            (IfBlock(def), b, c, d) => {
                control_if(ctx, &def)?;
                Ok(vec![b, c, d])
            }
            (AssertLine(def), b, c, d) => {
                let _word = eval_lines(&def, ctx).context("assert body")?;
                // TODO: actually assert
                Ok(vec![b, c, d])
            }
            (ref w, Verb(v), Noun(y), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("0 monad");
                Ok(vec![
                    fragment.0,
                    v.exec(ctx, None, &y).map(Word::Noun)?,
                    any,
                ])
            }
            (ref w, Verb(ref u), Verb(ref v), Noun(y))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                debug!("1 monad");
                Ok(vec![
                    fragment.0,
                    Verb(u.clone()),
                    v.exec(ctx, None, &y).map(Word::Noun)?,
                ])
            }
            (ref w, Noun(x), Verb(ref v), Noun(y))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                debug!("2 dyad");
                Ok(vec![
                    fragment.0,
                    v.exec(ctx, Some(&x), &y)
                        .context("evaluating 2 dyad")
                        .map(Word::Noun)?,
                ])
            }
            // (V|N) A anything - 3 Adverb
            (ref w, u @ Verb(_), Adverb(a), any)
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                debug!("3 adverb V A _");
                Ok(vec![fragment.0, a.form_adverb(ctx, &u)?, any])
            }
            (ref w, n @ Noun(_), Adverb(a), any)
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                Ok(vec![fragment.0, a.form_adverb(ctx, &n)?, any])
            }
            //// (V|N) C (V|N) - 4 Conjunction
            (ref w, l, Conjunction(c), r)
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) && matches!(l, Verb(_) | Noun(_) | Name(_))
                    && matches!(r, Verb(_) | Noun(_) | Name(_))
                    // hack: noun noun conj handled by the parser
                    && !(matches!(l, Noun(_)) && matches!(r, Noun(_))) =>
            {
                debug!("4 Conj");
                let (farcical, formed) = c.form_conjunction(ctx, &l, &r)?;
                assert!(!farcical);
                Ok(vec![fragment.0, formed])
            }
            (ref w, Noun(ref m), Conjunction(ref c), Noun(n))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                debug!("4 Conj N C N");
                let (farcical, formed) =
                    c.form_conjunction(ctx, &Word::Noun(m.clone()), &Word::Noun(n.clone()))?;
                if !farcical {
                    Ok(vec![fragment.0, formed])
                } else {
                    queue.push_back(fragment.0);
                    stack.push_front(fragment.2);
                    stack.push_front(fragment.1);
                    debug!("suspending {queue:?} {stack:?}");
                    ctx.input_buffers
                        .as_mut()
                        .ok_or(JError::ControlError)
                        .context("suspension without buffers")?
                        .suspend(Qs { queue, stack })?;
                    return Ok(EvalOutput::Suspension);
                }
            }
            //// (V|N) V V - 5 Fork
            (ref w, Verb(f), Verb(g), Verb(h))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                debug!("5 Fork V V V");
                let fork = VerbImpl::Fork {
                    f: Box::new(Verb(f.clone())),
                    g: Box::new(Verb(g.clone())),
                    h: Box::new(Verb(h.clone())),
                };
                Ok(vec![fragment.0, Verb(fork)])
            }
            (ref w, Noun(m), Verb(g), Verb(h))
                if matches!(
                    w,
                    StartOfLine | IsGlobal | IsLocal | LP | Adverb(_) | Verb(_) | Noun(_)
                ) =>
            {
                debug!("5 Fork N V V");
                let fork = VerbImpl::Fork {
                    f: Box::new(Noun(m)),
                    g: Box::new(Verb(g.clone())),
                    h: Box::new(Verb(h.clone())),
                };
                Ok(vec![fragment.0, Verb(fork)])
            }
            // TODO: The new (old) modifier tridents and bidents:
            // https://code.jsoftware.com/wiki/Vocabulary/Parsing#The_Parsing_Table
            // https://code.jsoftware.com/wiki/Vocabulary/fork#invisiblemodifiers

            // TODO: Figure out how the rest of the hook combinations work.
            // (C|A|V|N) (C|A|V|N) anything - 6 Hook/Adverb
            // Only the combinations A A, C N, C V, N C, V C, and V V are valid;
            // the rest result in syntax errors.
            (ref w, Adverb(_), Adverb(_), _any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb A A _");
                bail!("unable to bond adverbs")
                // let adverb_str = format!("{} {}", sa0, sa1);
                // let da = ModifierImpl::DerivedAdverb {
                //     l: Box::new(Adverb(sa0, a0.clone())),
                //     r: Box::new(Adverb(sa1, a1.clone())),
                // };
                // Ok(vec![fragment.0, Adverb(adverb_str, da), any])
            }
            (ref w, Conjunction(c), Noun(n), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb C N _");
                let da = ModifierImpl::DerivedAdverb {
                    c: Box::new(c),
                    u: Box::new(Noun(n)),
                };
                Ok(vec![fragment.0, Adverb(da), any])
            }
            (ref w, Conjunction(c), Verb(v), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb C V _");
                let da = ModifierImpl::DerivedAdverb {
                    c: Box::new(c),
                    u: Box::new(Verb(v.clone())),
                };
                Ok(vec![fragment.0, Adverb(da), any])
            }
            //(w, Noun(n), Conjunction(d), _) => println!("6 Hook/Adverb N C _"),
            //(w, Verb(u), Conjunction(d), _) => println!("6 Hook/Adverb V C _"),
            (ref w, Verb(u), Verb(v), any)
                if matches!(w, StartOfLine | IsGlobal | IsLocal | LP) =>
            {
                debug!("6 Hook/Adverb V V _");
                let hook = VerbImpl::Hook {
                    l: Box::new(Verb(u.clone())),
                    r: Box::new(Verb(v.clone())),
                };
                Ok(vec![fragment.0, Verb(hook), any])
            }
            //(w, Verb(u), Adverb(b), _) => println!("SYNTAX ERROR 6 Hook/Adverb V A _"),
            //(w, Verb(u), Noun(m), _) => println!("SYNTAX ERROR 6 Hook/Adverb V N _"),
            //(w, Noun(n), Adverb(b), _) => println!("SYNTAX ERROR 6 Hook/Adverb N A _"),
            //(w, Noun(n), Verb(v), _) => println!("SYNTAX ERROR 6 Hook/Adverb N V _"),
            //(w, Noun(n), Noun(m), _) => println!("SYNTAX ERROR 6 Hook/Adverb N N _"),

            //// (Name|Noun) (IsLocal|IsGlobal) (C|A|V|N) anything - 7 Is
            (Name(n), IsLocal, w, any)
                if matches!(w, Conjunction(_) | Adverb(_) | Verb(_) | Noun(_)) =>
            {
                debug!("7 Is Local Name w");
                ctx.eval_mut().locales.assign_local(n, w.clone())?;
                Ok(vec![w.clone(), any])
            }
            (Noun(names), IsLocal, w, any)
                if matches!(w, Conjunction(_) | Adverb(_) | Verb(_) | Noun(_)) =>
            {
                debug!("7 Is Local Noun w");
                let (arr, names) = string_assignment(names, w)?;

                for (name, val) in names.into_iter().zip(arr.outer_iter()) {
                    ctx.eval_mut()
                        .locales
                        .assign_local(name, Noun(v_open(&val.into_owned())?))?;
                }
                Ok(vec![any])
            }
            (Name(n), IsGlobal, w, any)
                if matches!(w, Conjunction(_) | Adverb(_) | Verb(_) | Noun(_)) =>
            {
                debug!("7 Is Global Name w");
                ctx.eval_mut().locales.assign_global(n, w.clone())?;
                Ok(vec![w, any])
            }
            (Noun(names), IsGlobal, w, any)
                if matches!(w, Conjunction(_) | Adverb(_) | Verb(_) | Noun(_)) =>
            {
                debug!("7 Is Global Noun w");
                let (arr, names) = string_assignment(names, w)?;

                for (name, val) in names.into_iter().zip(arr.outer_iter()) {
                    ctx.eval_mut()
                        .locales
                        .assign_global(name, Noun(v_open(&val.into_owned())?))?;
                }
                Ok(vec![any])
            }
            //// LP (C|A|V|N) RP anything - 8 Paren
            (LP, w, RP, any) if matches!(w, Conjunction(_) | Adverb(_) | Verb(_) | Noun(_)) => {
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
        Some(val) => Err(JError::SyntaxError).with_context(|| {
            anyhow!("expected a single output value but found {val:#?} followed by {new_stack:#?}")
        }),
        None => Err(JError::SyntaxError)
            .with_context(|| anyhow!("expected a single output value but found nothing")),
    }
}

fn string_assignment(names: JArray, w: Word) -> Result<(JArray, Vec<String>)> {
    let Noun(arr) = w else { return Err(JError::NonceError).context("non-noun on the right of noun assignment"); };
    let JArray::CharArray(names) = names else { return Err(JError::NonceError).context("assigning to char arrays only please"); };
    if names.shape().len() != 1 {
        return Err(JError::NonceError)
            .context("lists of chars (strings), not multi-dimensional char arrays");
    }
    // presumably this is supposed to be "words"
    let names = names.iter().collect::<String>();
    let names = names
        .split_whitespace()
        .map(|s| s.to_string())
        .collect_vec();
    if names.len() == 1 {
        return Ok((JArray::BoxArray(arr0d(arr)), names));
    }

    if arr.len_of_0() != names.len() {
        return Err(JError::LengthError).with_context(|| {
            anyhow!("wrong number of names for an array: {arr:#?} into {names:?}")
        });
    }
    Ok((arr, names))
}

fn get_fragment(stack: &mut VecDeque<Word>) -> (Word, Word, Word, Word) {
    stack
        .drain(..stack.len().min(4))
        .chain(repeat(Nothing))
        .next_tuple()
        .expect("infinite iterator can't be empty")
}

pub fn resolve_names(
    fragment: (Word, Word, Word, Word),
    ctx: &Ctx,
) -> Result<(Word, Word, Word, Word)> {
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
            Name(ref n) => resolved_words.push(ctx.eval().locales.lookup(n)?.unwrap_or(w).clone()),
            _ => resolved_words.push(w.clone()),
        }
    }
    resolved_words.reverse();

    let l = words.len() - resolved_words.len();
    let new_words = [&words[..l], &resolved_words[..]].concat();
    Ok(new_words.iter().cloned().collect_tuple().unwrap())
}
