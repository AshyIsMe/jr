use crate::JError;
use crate::Word;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AdverbImpl {
    Slash,
    CurlyRt,
    NotImplemented,
}

impl AdverbImpl {
    pub fn exec<'a>(&'a self, x: Option<&Word>, v: &Word, y: &Word) -> Result<Word, JError> {
        match self {
            AdverbImpl::Slash => a_slash(x, v, y),
            AdverbImpl::CurlyRt => a_curlyrt(x, v, y),
            AdverbImpl::NotImplemented => a_not_implemented(x, v, y),
        }
    }
}

pub fn a_not_implemented(_x: Option<&Word>, _v: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError::custom("adverb not implemented yet"))
}

pub fn a_slash(x: Option<&Word>, v: &Word, y: &Word) -> Result<Word, JError> {
    match x {
        None => match v {
            Word::Verb(_, v) => match y {
                Word::Noun(_) => y
                    .to_cells()?
                    .into_iter()
                    .map(Ok)
                    .reduce(|x, y| v.exec(Some(&x?), &y?))
                    .ok_or(JError::DomainError)?,
                _ => Err(JError::custom("noun expected")),
            },
            _ => Err(JError::custom("verb expected")),
        },
        Some(_x) => Err(JError::custom("dyadic / not implemented yet")),
    }
}

pub fn a_curlyrt(_x: Option<&Word>, _v: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError::custom("adverb not implemented yet"))
}
