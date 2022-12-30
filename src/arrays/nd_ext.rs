use anyhow::Result;
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
