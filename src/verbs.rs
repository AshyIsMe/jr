use crate::JArray::*;
use crate::JError;
use crate::Word;

pub fn v_not_implemented<'a>(_x: Option<&Word>, _y: &Word) -> Result<Word, JError> {
    Err(JError {
        message: "verb not implemented yet".to_string(),
    })
}

pub fn v_plus<'a>(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: "monadic + not implemented yet".to_string(),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { a: x }), Word::Noun(IntArray { a: y })) = (x, y) {
                Ok(Word::Noun(IntArray { a: x + y }))
            } else {
                Err(JError {
                    message: "plus not supported for these types yet".to_string(),
                })
            }
        }
    }
}

pub fn v_minus<'a>(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: "monadic - not implemented yet".to_string(),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { a: x }), Word::Noun(IntArray { a: y })) = (x, y) {
                Ok(Word::Noun(IntArray { a: x - y }))
            } else {
                Err(JError {
                    message: "minus not supported for these types yet".to_string(),
                })
            }
        }
    }
}

pub fn v_times<'a>(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: "monadic * not implemented yet".to_string(),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { a: x }), Word::Noun(IntArray { a: y })) = (x, y) {
                Ok(Word::Noun(IntArray { a: x * y }))
            } else {
                Err(JError {
                    message: "times not supported for these types yet".to_string(),
                })
            }
        }
    }
}
