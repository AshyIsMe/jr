use crate::{arr0ad, feed, Ctx, JArray, JError};
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

            // AA TODO: handle y better than this
            let _res = feed(&y.iter().collect::<String>(), ctx);

            let t = now.elapsed().as_secs_f64();
            Ok(JArray::FloatArray(arr0ad(t)))
        }
        (Some(_), _) => Err(JError::DomainError).context("x IntArray, y CharArray please"),
        (None, _) => Err(JError::DomainError).context("y CharArray please"),
    }
}
