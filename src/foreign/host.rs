use anyhow::{Context, Result};

use crate::{arr0d, JArray, JError, Word};

pub fn f_shell_out(y: &Word) -> Result<Word> {
    let Word::Noun(JArray::CharArray(y)) = y else { return Err(JError::NonceError).context("string required") };
    // TODO: new lines
    let y = y.iter().collect::<String>();
    let result = match y.as_str() {
        "uname" if cfg!(target_os = "linux") => "Linux",
        _ => return Err(JError::NonceError).context("shelling out is disabled"),
    };

    Ok(Word::Noun(JArray::from_string(result)))
}

pub fn f_getenv(y: &Word) -> Result<Word> {
    let Word::Noun(JArray::CharArray(y)) = y else { return Err(JError::NonceError).context("string required") };
    // TODO: new lines
    let y = y.iter().collect::<String>();
    use std::env::VarError;
    match std::env::var(y) {
        Ok(result) => Ok(Word::Noun(JArray::from_string(result))),
        Err(VarError::NotPresent) => Ok(Word::Noun(JArray::from(arr0d(0u8)))),
        Err(VarError::NotUnicode(_)) => {
            Err(JError::DomainError).context("environment variable not representable")
        }
    }
}
