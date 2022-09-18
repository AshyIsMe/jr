use crate::JArray::{ExtIntArray, IntArray};
use crate::JError;

use crate::Word;

// Implementations for Adverbs and Conjuntions
// https://code.jsoftware.com/wiki/Vocabulary/Modifiers
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ModifierImpl {
    NotImplemented,

    //adverbs
    Slash,
    CurlyRt,

    //conjunctions
    HatCo,
}

impl ModifierImpl {
    pub fn exec<'a>(
        &'a self,
        x: Option<&Word>,
        u: &Word,
        v: &Word,
        y: &Word,
    ) -> Result<Word, JError> {
        match self {
            ModifierImpl::NotImplemented => a_not_implemented(x, u, y),
            ModifierImpl::Slash => a_slash(x, u, y),
            ModifierImpl::CurlyRt => a_curlyrt(x, u, y),
            ModifierImpl::HatCo => c_hatco(x, u, v, y),
        }
    }
}

pub fn a_not_implemented(_x: Option<&Word>, _u: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError::custom("adverb not implemented yet"))
}

pub fn a_slash(x: Option<&Word>, u: &Word, y: &Word) -> Result<Word, JError> {
    match x {
        None => match u {
            Word::Verb(_, u) => match y {
                Word::Noun(_) => y
                    .to_cells()?
                    .into_iter()
                    .map(Ok)
                    .reduce(|x, y| u.exec(Some(&x?), &y?))
                    .ok_or(JError::DomainError)?,
                _ => Err(JError::custom("noun expected")),
            },
            _ => Err(JError::DomainError),
        },
        Some(_x) => Err(JError::custom("dyadic / not implemented yet")),
    }
}

pub fn a_curlyrt(_x: Option<&Word>, _u: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError::custom("adverb not implemented yet"))
}

pub fn c_hatco(x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word, JError> {
    match (u, v) {
        (Word::Verb(_, u), Word::Noun(IntArray { a: n })) => {
            if n.len() == 1 {
                let r: Vec<Word> = n
                    .iter()
                    .map(|i| {
                        let mut t = y.clone();
                        for _ in 0..*i {
                            t = u.exec(x, &t).unwrap();
                        }
                        t
                    })
                    .collect();
                //TODO Handle shape of array properly
                Ok(r[0].clone())
            } else {
                //TODO Handle shape of array properly
                todo!("power conjunction noun right argument properly")
            }
        }
        (Word::Verb(_, _), Word::Verb(_, _)) => todo!("power conjunction verb right argument"),
        _ => Err(JError::DomainError),
    }
}
