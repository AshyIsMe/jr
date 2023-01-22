use crate::{arr0ad, Ctx, JArray, JError};
use anyhow::{anyhow, Context, Result};
use ndarray::array;
use std::time::{Duration, Instant};

// AA TODO
// 6!:2
//
pub fn f_time_sentence(ctx: &mut Ctx, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
    match (x, y) {
        (Some(JArray::IntArray(_x)), JArray::CharArray(_y)) => {
            todo!("x 6!:2 y")
        }
        (None, JArray::CharArray(y)) => {
            let now = Instant::now();

            todo!("6!:2 y");

            let t = now.elapsed().as_secs();
            // AA TODO: as_secs() is an int, wtf?!
            Ok(JArray::IntArray(arr0ad(t as i64)))
        }
        (Some(_), _) => Err(JError::DomainError).context("x IntArray, y CharArray please"),
        (None, _) => Err(JError::DomainError).context("y CharArray please"),
    }
}
