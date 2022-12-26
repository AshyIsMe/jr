use ndarray::prelude::*;

// like IntoIterator<Item = T> + ExactSizeIterator
pub trait Arrayable<T> {
    fn into_vec(self) -> Vec<T>;

    fn into_array(self) -> ArrayD<T>
    where
        Self: Sized,
    {
        let vec = self.into_vec();
        Array::from_shape_vec(IxDyn(&[vec.len()]), vec).expect("len is correct, as we just read it")
    }
}

impl<T> Arrayable<T> for Vec<T> {
    fn into_vec(self) -> Vec<T> {
        self
    }
}

impl<T: Clone, const N: usize> Arrayable<T> for [T; N] {
    fn into_vec(self) -> Vec<T> {
        self.to_vec()
    }
}
