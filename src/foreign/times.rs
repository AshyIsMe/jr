use crate::{arr0ad, feed, Ctx, JArray, JError};
use anyhow::{anyhow, Context, Result};
use ndarray::{ArcArray, IxDyn};
use std::time::{Duration, Instant};

// 6!:2
//
pub fn f_time_sentence(ctx: &mut Ctx, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
    let counts = match x {
        Some(JArray::IntArray(x)) => Ok(x.iter().collect()),
        None => Ok(vec![&1i64]),
        _ => Err(JError::DomainError).context("x IntArray, y CharArray please"),
    }?;
    match y {
        JArray::CharArray(y) => {
            Ok(JArray::FloatArray(ArcArray::from_shape_vec(
                IxDyn(&[counts.len()]),
                counts
                    .iter()
                    .map(|count| {
                        if **count <= 0 {
                            0.0
                        } else {
                            let times: Vec<f64> = (0..**count)
                                .map(|_i| {
                                    let now = Instant::now();
                                    // AA TODO: handle y better than this?
                                    let _res = feed(&y.iter().collect::<String>(), ctx);
                                    now.elapsed().as_secs_f64()
                                })
                                .collect();
                            // average runtime over count repetitions
                            times.iter().sum::<f64>() / times.len() as f64
                        }
                    })
                    .collect(),
            )?))
        }
        _ => Err(JError::DomainError).context("y CharArray please"),
    }
}
