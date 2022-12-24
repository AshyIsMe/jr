use anyhow::Result;

use crate::{arr0d, JArray, Word};

pub fn f_os_type() -> Result<Word> {
    Ok(Word::Noun(JArray::IntArray(arr0d(
        match std::env::consts::OS {
            "linux" => 5i64,
            "windows" => 6,
            _ => -1,
        },
    ))))
}
