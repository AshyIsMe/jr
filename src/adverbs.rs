use crate::JError;
use crate::Word;

#[derive(Copy, Clone, Debug, PartialEq)]
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
    Err(JError {
        message: "adverb not implemented yet".to_string(),
    })
}

pub fn a_slash(x: Option<&Word>, v: &Word, y: &Word) -> Result<Word, JError> {
    match x {
        None => match v {
            Word::Verb(_, v) => match y {
                Word::Noun(_) => match y
                    .to_cells()
                    .unwrap()
                    .into_iter()
                    .reduce(|x, y| v.exec(Some(&x), &y).unwrap())
                {
                    Some(w) => Ok(w.clone()),
                    None => Err(JError {
                        message: "domain error".to_string(),
                    }),
                },
                _ => Err(JError {
                    message: "noun expected".to_string(),
                }),
            },
            _ => Err(JError {
                message: "verb expected".to_string(),
            }),
        },
        Some(_x) => Err(JError {
            message: "dyadic / not implemented yet".to_string(),
        }),
    }
}

pub fn a_curlyrt(_x: Option<&Word>, _v: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError {
        message: "adverb not implemented yet".to_string(),
    })
}
