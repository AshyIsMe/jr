use crate::int_array;
use crate::JArray::*;
use crate::JError;
use crate::Word;
use ndarray::prelude::*;

pub fn v_not_implemented(_x: Option<&Word>, _y: &Word) -> Result<Word, JError> {
    Err(JError {
        message: "verb not implemented yet".to_string(),
    })
}

pub fn v_plus(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => Err(JError {
            message: "monadic + not implemented yet".to_string(),
        }),
        Some(x) => match (x, y) {
            // TODO extract promotion to function:
            // https://code.jsoftware.com/wiki/Vocabulary/NumericPrecisions
            (Word::Noun(IntArray { a: x }), Word::Noun(IntArray { a: y })) => {
                Ok(Word::Noun(IntArray { a: x + y }))
            }
            (Word::Noun(IntArray { a: x }), Word::Noun(FloatArray { a: y })) => {
                Ok(Word::Noun(FloatArray {
                    a: x.map(|i| *i as f64) + y,
                }))
            }
            (Word::Noun(FloatArray { a: x }), Word::Noun(IntArray { a: y })) => {
                Ok(Word::Noun(FloatArray {
                    a: x + y.map(|i| *i as f64),
                }))
            }
            _ => Err(JError {
                message: "plus not supported for these types yet".to_string(),
            }),
        },
    }
}

pub fn v_minus(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
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

pub fn v_times(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
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

pub fn v_number(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => {
            // Tally
            match y {
                Word::Noun(ja) => match ja {
                    IntArray { a } => Ok(int_array(vec![a.len() as i64]).unwrap()),
                    ExtIntArray { a } => Ok(int_array(vec![a.len() as i64]).unwrap()),
                    FloatArray { a } => Ok(int_array(vec![a.len() as i64]).unwrap()),
                    BoolArray { a } => Ok(int_array(vec![a.len() as i64]).unwrap()),
                    CharArray { a } => Ok(int_array(vec![a.len() as i64]).unwrap()),
                },
                _ => Err(JError {
                    message: "domain error".to_string(),
                }),
            }
        }
        Some(_x) => Err(JError {
            message: "dyadic # not implemented yet".to_string(),
        }), // Copy
    }
}

pub fn v_dollar(x: Option<&Word>, y: &Word) -> Result<Word, JError> {
    match x {
        None => {
            // Shape-of
            match y {
                Word::Noun(ja) => match ja {
                    IntArray { a } => {
                        Ok(int_array(a.shape().iter().map(|i| *i as i64).collect()).unwrap())
                    }
                    ExtIntArray { a } => {
                        Ok(int_array(a.shape().iter().map(|i| *i as i64).collect()).unwrap())
                    }
                    FloatArray { a } => {
                        Ok(int_array(a.shape().iter().map(|i| *i as i64).collect()).unwrap())
                    }
                    BoolArray { a } => {
                        Ok(int_array(a.shape().iter().map(|i| *i as i64).collect()).unwrap())
                    }
                    CharArray { a } => {
                        Ok(int_array(a.shape().iter().map(|i| *i as i64).collect()).unwrap())
                    }
                },
                _ => Err(JError {
                    message: "domain error".to_string(),
                }),
            }
        }
        Some(_x) => Err(JError {
            message: "dyadic $ not implemented yet".to_string(),
        }), // Copy
    }
}
