use crate::JError;
use crate::Word;

pub fn a_not_implemented<'a>(
    _x: Option<&Word<'a>>,
    _v: &Word<'a>,
    _y: &Word<'a>,
) -> Result<Word<'a>, JError<'a>> {
    Err(JError {
        message: "adverb not implemented yet",
    })
}

pub fn a_insert<'a>(
    _x: Option<&Word<'a>>,
    _v: &Word<'a>,
    _y: &Word<'a>,
) -> Result<Word<'a>, JError<'a>> {
    Err(JError {
        message: "adverb not implemented yet",
    })
}
