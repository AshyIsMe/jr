use crate::JArray::*;
use crate::JError;
use crate::Word;
use ndarray::prelude::*;

pub fn a_not_implemented(_x: Option<&Word>, _v: &Word, _y: &Word) -> Result<Word, JError> {
    Err(JError {
        message: "adverb not implemented yet".to_string(),
    })
}

pub fn a_insert(x: Option<&Word>, v: &Word, y: &Word) -> Result<Word, JError> {
    match x {
        None => match v {
            Word::Verb(_, v) => match y {
                Word::Noun(y) => match y {
                    IntArray { a: y } => Ok(Word::Noun(IntArray {
                        a: y.outer_iter()
                            .reduce(|x, y| {
                                match v
                                    .exec(
                                        Some(&Word::Noun(IntArray { a: x.into_owned() })),
                                        &&Word::Noun(IntArray { a: y.into_owned() }),
                                    )
                                    .unwrap()
                                {
                                    //Word::Noun(IntArray { a: r }) => r.view(),
                                    Word::Noun(IntArray { a: r }) => {
                                        todo!("i clearly still don't 'get' ownership")
                                    }
                                    _ => panic!("wow"),
                                }
                            })
                            .unwrap()
                            .to_owned(),
                    })),
                    _ => todo!("implement all noun types - this is tedious, can we be generic?"),
                },
                _ => Err(JError {
                    message: "noun expected".to_string(),
                }),
            },
            _ => Err(JError {
                message: "verd expected".to_string(),
            }),
        },
        Some(x) => {
            todo!("dyadic")
        }
    }
}
