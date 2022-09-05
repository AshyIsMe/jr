use crate::JError;
use crate::Word;

pub fn a_not_implemented(_x: Option<&Word>, _v: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError {
        message: "adverb not implemented yet".to_string(),
    })
}

pub fn a_insert(_x: Option<&Word>, _v: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError {
        message: "adverb not implemented yet".to_string(),
    })
}
