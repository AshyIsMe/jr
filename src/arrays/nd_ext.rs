use crate::JError;
use anyhow::{Context, Result};
use ndarray::prelude::*;
use ndarray::Data;

#[allow(unused)]
pub fn map_result<T, U>(arr: ArrayD<T>, f: impl FnMut(T) -> Result<U>) -> Result<ArrayD<U>> {
    let shape = arr.shape().to_vec();
    let data = arr.into_iter().map(f).collect::<Result<Vec<U>>>()?;
    Ok(ArrayD::from_shape_vec(shape, data).expect("just unpacked it"))
}

pub fn len_of_0(arr: &ArrayBase<impl Data, impl Dimension>) -> usize {
    match arr.shape() {
        [] => 1,
        s => s[0],
    }
}

// copied from ndarray: https://github.com/rust-ndarray/ndarray/blob/07406955868dd98985d7e2f1de1f643be4d8888f/src/dimension/mod.rs#L78-L99
// adapted for our error system. License: Apache 2.0 OR MIT.
/// Returns the `size` of the `dim`, checking that the product of non-zero axis
/// lengths does not exceed `isize::MAX`.
///
/// If `size_of_checked_shape(dim)` returns `Ok(size)`, the data buffer is a
/// slice or `Vec` of length `size`, and `strides` are created with
/// `self.default_strides()` or `self.fortran_strides()`, then the invariants
/// are met to construct an array from the data buffer, `dim`, and `strides`.
/// (The data buffer being a slice or `Vec` guarantees that it contains no more
/// than `isize::MAX` bytes.)
pub fn size_of_shape_checked<D: Dimension>(dim: &D) -> Result<usize> {
    let help = "dimensionality is too big (by a lot)";
    let size_nonzero = dim
        .slice()
        .iter()
        .filter(|&&d| d != 0)
        .try_fold(1usize, |acc, &d| acc.checked_mul(d))
        .ok_or(JError::LimitError)
        .context(help)?;
    if size_nonzero > isize::MAX as usize {
        Err(JError::LimitError).context(help)
    } else {
        Ok(dim.size())
    }
}
