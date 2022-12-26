use ndarray::prelude::*;

pub trait IntoVec<T> {
    fn into_vec(self) -> Vec<T>;

    fn into_array(self) -> ArrayD<T>
    where
        Self: Sized,
    {
        let vec = self.into_vec();
        Array::from_shape_vec(IxDyn(&[vec.len()]), vec).expect("len is correct, as we just read it")
    }
}

impl<T> IntoVec<T> for Vec<T> {
    fn into_vec(self) -> Vec<T> {
        self
    }
}

impl<T: Clone, const N: usize> IntoVec<T> for [T; N] {
    fn into_vec(self) -> Vec<T> {
        self.to_vec()
    }
}
