use anyhow::{anyhow, Context, Result};
use ndarray::prelude::*;

use crate::JError;

// like IntoIterator<Item = T> + ExactSizeIterator
pub trait Arrayable<T> {
    fn len(&self) -> usize;
    fn into_vec(self) -> Result<Vec<T>>;

    fn into_array(self) -> Result<ArrayD<T>>
    where
        Self: Sized,
    {
        let len = self.len();
        let vec = self.into_vec()?;
        Array::from_shape_vec(IxDyn(&[len]), vec)
            .map_err(JError::ShapeError)
            .context("into_array")
    }
}

impl<T> Arrayable<T> for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn into_vec(self) -> Result<Vec<T>> {
        Ok(self)
    }
}

// This is designed for use with shape(), sorry if it caught something else.
impl Arrayable<i64> for &[usize] {
    fn len(&self) -> usize {
        <[usize]>::len(self)
    }

    fn into_vec(self) -> Result<Vec<i64>> {
        self.iter()
            .map(|&v| {
                i64::try_from(v)
                    .map_err(|_| JError::LimitError)
                    .with_context(|| anyhow!("{} doesn't fit in an i64", v))
            })
            .collect()
    }
}

impl<T: Clone, const N: usize> Arrayable<T> for [T; N] {
    fn len(&self) -> usize {
        N
    }

    fn into_vec(self) -> Result<Vec<T>> {
        Ok(self.to_vec())
    }
}

impl<T> Arrayable<T> for ArrayD<T> {
    fn len(&self) -> usize {
        self.len()
    }

    fn into_vec(self) -> Result<Vec<T>> {
        Ok(self.into_raw_vec())
    }

    fn into_array(self) -> Result<ArrayD<T>> {
        Ok(self)
    }
}
