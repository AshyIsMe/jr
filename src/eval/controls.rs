use crate::verbs::VerbImpl;
use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use std::collections::HashSet;

use crate::{JError, Word};

enum Resolution {
    Complete,
    InsufficientInput,
    StepTaken,
}

/// true iff resolution completed
pub fn resolve_controls(words: &mut Vec<Word>) -> Result<bool> {
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

fn create_def(mode: char, def: Vec<Word>) -> Result<Word> {
    Ok(match mode {
        // sorry not sorry
        'm' => Word::Verb("anon".to_string(), VerbImpl::Anonymous(false, def)),
        'd' => Word::Verb("anon".to_string(), VerbImpl::Anonymous(true, def)),
        other => {
            return Err(JError::NonceError)
                .with_context(|| anyhow!("unsupported direct def: {other}"))
        }
    })
}
