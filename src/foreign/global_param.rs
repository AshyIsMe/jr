use anyhow::Result;

use crate::Word;

pub fn f_os_type() -> Result<Word> {
    Word::noun(vec![match std::env::consts::OS {
        "linux" => 5i64,
        "windows" => 6,
        _ => -1,
    }])
}
