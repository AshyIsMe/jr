use anyhow::{Context, Result};
use itertools::Itertools;

use crate::{JArray, JError, Word};

pub fn f_shell_out(y: &Word) -> Result<Word> {
    let Word::Noun(JArray::CharArray(y)) = y else { return Err(JError::NonceError).context("string required") };
    // TODO: new lines
    let y = y.iter().collect::<String>();
    let result = match y.as_str() {
        "uname" if cfg!(target_os = "linux") => "Linux",
        _ => return Err(JError::NonceError).context("shelling out is disabled"),
    };

    Word::noun(result.chars().collect_vec())
}