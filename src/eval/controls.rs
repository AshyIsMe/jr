use std::collections::HashSet;

use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;

use crate::eval::eval_lines;
use crate::verbs::{BivalentOwned, PartialImpl, VerbImpl};
use crate::{JArray, JError, Rank, Word};

enum Resolution {
    Complete,
    InsufficientInput,
    StepTaken,
}

/// true iff resolution completed
pub fn resolve_controls(words: &mut Vec<Word>) -> Result<bool> {
    while match resolve_one_assert(words)? {
        Resolution::StepTaken => true,
        Resolution::Complete => false,
        Resolution::InsufficientInput => return Ok(false),
    } {}

    while match resolve_one_direct_def(words)? {
        Resolution::StepTaken => true,
        Resolution::Complete => false,
        Resolution::InsufficientInput => return Ok(false),
    } {}

    while match resolve_one_stat(words)? {
        Resolution::StepTaken => true,
        Resolution::Complete => false,
        Resolution::InsufficientInput => return Ok(false),
    } {}
    if let Some((pos, remaining)) = words.iter().find_position(|w| w.is_control_symbol()) {
        bail!("control resolution failed: didn't eliminate {remaining:?} at {pos}");
    }
    Ok(true)
}

fn resolve_one_assert(words: &mut Vec<Word>) -> Result<Resolution> {
    let last_assert_start = match words.iter().rposition(|w| matches!(w, Word::Assert)) {
        Some(x) => x,
        None => return Ok(Resolution::Complete),
    };
    let assert_end = words
        .iter()
        .skip(last_assert_start)
        .position(|w| matches!(w, Word::NewLine))
        .map(|p| p + last_assert_start)
        .unwrap_or(words.len());

    // (last_dd_start..dd_end) includes the start and end, which we want to remove from the words
    let mut def = words.drain(last_assert_start..assert_end).collect_vec();

    // ... but not include in the definition
    assert_eq!(Word::Assert, def.remove(0));

    // maybe should remove the trailing newline

    // don't think we need to resolve controls, as controls are not expressions?
    // assert. if. 5 do.
    // ..., no, because if. isn't an expression.

    words.insert(last_assert_start, Word::AssertLine(def));
    Ok(Resolution::StepTaken)
}

fn resolve_one_direct_def(words: &mut Vec<Word>) -> Result<Resolution> {
    let last_dd_start = match words
        .iter()
        .rposition(|w| matches!(w, Word::DirectDefUnknown | Word::DirectDef(_)))
    {
        Some(x) => x,
        None => return Ok(Resolution::Complete),
    };
    let dd_end = match words
        .iter()
        .skip(last_dd_start)
        .position(|w| matches!(w, Word::DirectDefEnd))
    {
        Some(x) => last_dd_start + x + 1,
        None => return Ok(Resolution::InsufficientInput),
    };

    // (last_dd_start..dd_end) includes the start and end, which we want to remove from the words
    let mut def = words.drain(last_dd_start..dd_end).collect_vec();

    // ... but not include in the definition
    let kind = def.remove(0); // start
    assert!(matches!(def.pop(), Some(Word::DirectDefEnd)));

    ensure!(
        resolve_controls(&mut def)?,
        "controls inside a direct def must all be terminated"
    );

    let def = match kind {
        Word::DirectDefUnknown => create_def(infer_type(&def)?, def)?,
        Word::DirectDef(c) => create_def(c, def)?,
        other => unreachable!("matches! above excludes {other:?}"),
    };
    words.insert(last_dd_start, def);
    Ok(Resolution::StepTaken)
}

// yes, I am fully aware that this is what parser frameworks were invented to avoid
fn resolve_one_stat(words: &mut Vec<Word>) -> Result<Resolution> {
    let last_start = match words
        .iter()
        .rposition(|w| matches!(w, Word::If | Word::For(_) | Word::While))
    {
        Some(x) => x,
        None => return Ok(Resolution::Complete),
    };
    let end = match words
        .iter()
        .skip(last_start)
        .position(|w| matches!(w, Word::End))
    {
        Some(x) => last_start + x + 1,
        None => return Ok(Resolution::InsufficientInput),
    };

    // (last_if_start..if_end) includes the start and end, which we want to remove from the words
    let mut def = words.drain(last_start..end).collect_vec();

    // ... but not include in the definition
    let kind = def.remove(0); // start
    assert!(matches!(def.pop(), Some(Word::End)));

    let def = match kind {
        Word::If => Word::IfBlock(def),
        Word::For(ident) => Word::ForBlock(ident, def),
        Word::While => Word::WhileBlock(def),
        other => unreachable!("matches! above excludes {other:?}"),
    };
    words.insert(last_start, def);
    Ok(Resolution::StepTaken)
}

/// https://code.jsoftware.com/wiki/Vocabulary/DirectDefinition#What_kind_of_entity_will_DD_create.3F
fn infer_type(def: &[Word]) -> Result<char> {
    let used_names: HashSet<char> = def
        .iter()
        .filter_map(|x| match x {
            Word::Name(n) if n.len() == 1 => Some(n.chars().next().expect("checked")),
            _ => None,
        })
        .collect();

    Ok(if used_names.contains(&'n') || used_names.contains(&'v') {
        'c'
    } else if used_names.contains(&'m') || used_names.contains(&'u') {
        'a'
    } else if used_names.contains(&'x') {
        // TODO: infer bivalent?
        'd'
    } else {
        'm'
    })
}

pub fn create_def(mode: char, def: Vec<Word>) -> Result<Word> {
    Ok(match mode {
        // sorry not sorry
        'm' => {
            let imp = BivalentOwned {
                biv: BivalentOwned::from_monad(move |ctx, y| {
                    let mut ctx = ctx.nest();
                    ctx.eval_mut()
                        .locales
                        .assign_local("y", Word::Noun(y.clone()))?;
                    eval_lines(&def, &mut ctx)
                        .context("anonymous")
                        .and_then(must_be_noun)
                }),
                ranks: Rank::inf_inf_inf(),
            };
            Word::Verb(VerbImpl::Partial(PartialImpl { imp, def: None }))
        }
        'd' => {
            let imp = BivalentOwned {
                biv: BivalentOwned::from_bivalent(move |ctx, x, y| {
                    let Some(x) = x else {
                        return Err(JError::DomainError).context("explicitly dyadic udf invoked as monad");
                    };
                    let mut ctx = ctx.nest();
                    ctx.eval_mut()
                        .locales
                        .assign_local("x", Word::Noun(x.clone()))?;
                    ctx.eval_mut()
                        .locales
                        .assign_local("y", Word::Noun(y.clone()))?;
                    eval_lines(&def, &mut ctx)
                        .context("anonymous")
                        .and_then(must_be_noun)
                }),
                ranks: Rank::inf_inf_inf(),
            };
            Word::Verb(VerbImpl::Partial(PartialImpl { imp, def: None }))
        }
        other => {
            return Err(JError::NonceError)
                .with_context(|| anyhow!("unsupported direct def: {other}"))
        }
    })
}

fn must_be_noun(v: Word) -> Result<JArray> {
    match v {
        Word::Noun(arr) => Ok(arr),
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("unexpected non-noun in noun context: {v:?}")),
    }
}
