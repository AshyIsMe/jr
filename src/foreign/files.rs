//! https://www.jsoftware.com/help/dictionary/dx001.htm

use std::fs;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::{Arrayable, IntoJArray, JArray, JError, Word};

// 1!:1
pub fn f_read_file(y: &Word) -> Result<Word> {
    match y {
        Word::Noun(JArray::BoxArray(arr)) if arr.len() == 1 => {
            let arr = arr
                .iter()
                .next()
                .ok_or(JError::DomainError)
                .context("empty box?")?;
            let arr = arr
                .when_char()
                .ok_or(JError::NonceError)
                .context("can't read boxed non-paths")?;

            if arr.shape().len() > 1 {
                return Err(JError::NonceError).context("multi-dimensional path");
            }

            let path: String = arr.iter().copied().collect();
            match fs::read_to_string(&path) {
                Ok(s) => Ok(s.chars().collect_vec().into_array()?.into_noun()),
                Err(e) => Err(JError::FileNameError)
                    .context(e)
                    .with_context(|| anyhow!("reading {path:?}")),
            }
        }
        _ => Err(JError::NonceError)
            .context("can't read non-paths (hint: pointless box required 1!:1 <'a')"),
    }
}
