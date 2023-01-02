use std::fmt;

use ndarray::ArrayBase;

use crate::{impl_array, Elem, JArray};

pub fn nd(mut f: impl fmt::Write, arr: &JArray) -> fmt::Result {
    match arr {
        JArray::BoxArray(_) => impl_array!(arr, |a: &ArrayBase<_, _>| write!(f, "|{}|", a)),
        _ => impl_array!(arr, |a: &ArrayBase<_, _>| write!(f, "{}", a)),
    }
}

pub fn jsoft(mut f: impl fmt::Write, arr: &JArray) -> fmt::Result {
    match arr {
        JArray::BoxArray(_) => nd(f, arr),
        _ => js(&mut f, arr.shape(), &arr.clone().into_elems()),
    }
}

fn js(f: &mut impl fmt::Write, shape: &[usize], items: &[Elem]) -> fmt::Result {
    if shape.len() == 1 {
        return write!(f, "{:?}", &items[..shape[0]]);
    }
    let (last, shape) = shape.split_last().expect("<= 1");
    for x in 0..*last {
        js(f, shape, &items[x * shape.iter().product::<usize>()..])?;
        write!(f, "\n")?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{Elem, Num};

    fn idot(v: usize) -> Vec<Elem> {
        (0..v).map(|i| Elem::Num(Num::Int(i as i64))).collect()
    }

    #[test]
    fn js_2_2() {
        let mut s = String::new();
        super::js(&mut s, &[2, 3, 2], &idot(12)).unwrap();
        assert_eq!("", s)
    }
}
