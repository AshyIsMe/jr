use crate::JArray::*;
use crate::JError;
use crate::Word;

pub fn v_not_implemented<'a>(_x: Option<&Word<'a>>, _y: &Word<'a>) -> Result<Word<'a>, JError<'a>> {
    Err(JError {
        message: "verb not implemented yet",
    })
}

pub fn v_plus<'a>(x: Option<&Word<'a>>, y: &Word<'a>) -> Result<Word<'a>, JError<'a>> {
    match x {
        None => Err(JError {
            message: "monadic + not implemented yet",
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x + y }))
            } else {
                Err(JError {
                    message: "plus not supported for these types yet",
                })
            }
        }
    }
}

pub fn v_minus<'a>(x: Option<&Word<'a>>, y: &Word<'a>) -> Result<Word<'a>, JError<'a>> {
    match x {
        None => Err(JError {
            message: "monadic - not implemented yet",
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x - y }))
            } else {
                Err(JError {
                    message: "minus not supported for these types yet",
                })
            }
        }
    }
}

pub fn v_times<'a>(x: Option<&Word<'a>>, y: &Word<'a>) -> Result<Word<'a>, JError<'a>> {
    match x {
        None => Err(JError {
            message: "monadic * not implemented yet",
        }),
        Some(x) => {
            if let (Word::Noun(IntArray { v: x }), Word::Noun(IntArray { v: y })) = (x, y) {
                Ok(Word::Noun(IntArray { v: x * y }))
            } else {
                Err(JError {
                    message: "times not supported for these types yet",
                })
            }
        }
    }
}
