use crate::{feed, Ctx, JArray, JError};
use anyhow::{Context, Result};
use ndarray::ArcArray;
use std::time::Instant;

// 6!:2
//
pub fn f_time_sentence(ctx: &mut Ctx, x: Option<&JArray>, y: &JArray) -> Result<JArray> {
    let counts: Vec<i64> = match x {
        Some(JArray::IntArray(x)) => Ok(x.iter().cloned().collect()),
        Some(JArray::BoolArray(x)) => Ok(x.iter().map(|b| *b as i64).collect()),
        None => Ok(vec![1i64]),
        _ => Err(JError::DomainError).context("x IntArray, y CharArray please"),
    }?;
    match y {
        JArray::CharArray(y) => {
            Ok(JArray::FloatArray(ArcArray::from_shape_vec(
                if let Some(x) = x { x.shape() } else { &[1] },
                counts
                    .iter()
                    .map(|count| {
                        if *count <= 0 {
                            0.0
                        } else {
                            let times: Vec<f64> = (0..*count)
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
