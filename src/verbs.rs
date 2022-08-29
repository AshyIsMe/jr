
use crate::Word;
use crate::JArray::*;
use crate::JError;

pub fn v_not_implemented<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    Err(JError {
        message: String::from("verb not implemented yet"),
    })
}

pub fn v_plus<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: String::from("monadic + not implemented yet"),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x + y }))
            } else {
                Err(JError {
                    message: String::from("plus not supported for these types yet"),
                })
            }
        }
    }
}

pub fn v_minus<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: String::from("monadic - not implemented yet"),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x - y }))
            } else {
                Err(JError {
                    message: String::from("minus not supported for these types yet"),
                })
            }
        }
    }
}

pub fn v_times<'x, 'y>(x: Option<&'x Word>, y: &'y Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: String::from("monadic * not implemented yet"),
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x * y }))
            } else {
                Err(JError {
                    message: String::from("times not supported for these types yet"),
                })
            }
        }
    }
}
