//! https://www.jsoftware.com/help/dictionary/dx001.htm

use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::arrays::BoxArray;
use crate::{JArray, JError};

// 1!:1
pub fn f_read_file(y: &JArray) -> Result<JArray> {
    let path = noun_to_fs_path(y)?;

    match fs::read_to_string(&path) {
        Ok(s) => Ok(JArray::from_string(s)),
        Err(e) => Err(JError::FileNameError)
            .context(e)
            .with_context(|| anyhow!("reading {path:?}")),
    }
}

pub fn arg_to_string_list(y: &BoxArray) -> Result<Vec<String>> {
    if y.shape().len() > 1 {
        return Err(JError::NonceError).context("only list-y args'");
    }

    y.iter().map(|path| {
        let JArray::CharArray(path) = path else { return Err(JError::NonceError).context("string required") };
        if path.shape().len() > 1 {
            return Err(JError::NonceError).context("single string required");
        }

        Ok(path.iter().collect::<String>())
    }).collect()
}

pub fn arg_to_string(y: &BoxArray) -> Result<String> {
    if y.len() != 1 {
        return Err(JError::NonceError).context("only one arg please");
    }

    Ok(arg_to_string_list(y)?
        .into_iter()
        .next()
        .expect("just checked"))
}

pub fn noun_to_fs_path(y: &JArray) -> Result<PathBuf> {
    let JArray::BoxArray(y) = y else { return Err(JError::NonceError).context("only support <'filepath' loading"); };
    let path = arg_to_string(y)?;
    fs::canonicalize(&path).with_context(|| anyhow!("canonicalising {path}"))
}
