use itertools::Itertools;
use std::fmt;
use std::fmt::Display;

use ndarray::{ArrayBase, ArrayViewD};

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
        // _ => js(&mut f, arr.shape(), &arr.clone().into_elems()),
        _ => impl_array!(arr, |arr: &ArrayBase<_, _>| br(&mut f, arr.view())),
    }
}

// trait JFormat: Display {}

fn br<T: Display>(mut f: impl fmt::Write, arr: ArrayViewD<T>) -> fmt::Result {
    let limit = 50usize;
    // TODO: jsoft takes from the end, not the start, for some reason
    let iter = arr.rows().into_iter().take(limit);
    let table = iter
        .map(|x| {
            x.into_iter()
                .take(limit)
                .map(|x| format!("{x}"))
                .collect_vec()
        })
        .collect_vec();

    let widths = table
        .iter()
        .map(|row| row.iter().map(|s| s.chars().count()).collect_vec())
        .reduce(|l, r| {
            l.into_iter()
                .zip(r.into_iter())
                .map(|(l, r)| l.max(r))
                .collect_vec()
        })
        .expect("non-empty rows");

    for row in table {
        write!(f, "{widths:?} {row:?}\n")?;
    }

    Ok(())
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
    use itertools::Itertools;
    use ndarray::{ArrayD, IxDyn};

    fn idot(v: i64) -> ArrayD<i64> {
        ArrayD::from_shape_vec(IxDyn(&[v as usize]), (0..v).collect_vec())
            .expect("static shape in tests")
    }

    #[test]
    fn br_2_2() {
        let mut s = String::new();
        super::br(
            &mut s,
            idot(12).into_shape(IxDyn(&[2, 3, 2])).unwrap().view(),
        )
        .unwrap();
        assert_eq!("", s)
    }
}
