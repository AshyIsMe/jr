use itertools::Itertools;
use std::fmt;

use ndarray::{ArrayBase, ArrayViewD};
use num::complex::Complex64;
use num::{BigInt, BigRational};
use num_traits::{One, Zero};
use unicode_width::UnicodeWidthStr;

use crate::{impl_array, JArray};

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

trait JFormat {
    fn j_format(&self) -> String;
}

fn width(s: impl AsRef<str>) -> usize {
    s.as_ref().width()
}

fn br<T: JFormat>(mut f: impl fmt::Write, arr: ArrayViewD<T>) -> fmt::Result {
    if arr.shape().is_empty() {
        return write!(
            f,
            "{}\n",
            arr.first().expect("atom has an element").j_format()
        );
    }
    if arr.is_empty() {
        // what on earth is even going on
        return if arr.shape().len() == 1 {
            write!(f, "\n")
        } else {
            write!(f, "")
        };
    }
    let limit = 128usize;
    // TODO: jsoft takes from the end, not the start, for some reason
    let iter = arr.rows().into_iter().enumerate().take(limit);
    let table = iter
        .map(|(p, x)| {
            (
                p,
                x.into_iter()
                    .take(limit)
                    .map(|x| x.j_format())
                    .collect_vec(),
            )
        })
        .collect_vec();

    let widths: Vec<usize> = table
        .iter()
        .map(|(_, row)| row.iter().map(width).collect_vec())
        .reduce(|l, r| {
            l.into_iter()
                .zip(r.into_iter())
                .map(|(l, r)| l.max(r))
                .collect_vec()
        })
        .expect("non-empty rows");

    let multiples = arr
        .shape()
        .iter()
        .rev()
        .skip(1)
        .fold(Vec::<usize>::new(), |mut acc, &x| {
            let t: usize = acc.iter().product();
            acc.push(x * t);
            acc
        });

    let last = table.last().expect("non-empty").0;

    for (rn, row) in table {
        for (col, (target, item)) in widths.iter().zip(row.into_iter()).enumerate() {
            let len = width(&item);
            write!(
                f,
                "{}",
                (0..(target - len)).map(|_| ' ').collect::<String>()
            )?;
            if col != 0 {
                write!(f, " ")?;
            }
            write!(f, "{item}")?;
        }
        write!(f, "\n")?;
        if rn == last {
            break;
        }
        for &m in &multiples {
            if (rn + 1) % m == 0 {
                write!(f, "\n")?;
            }
        }
    }

    Ok(())
}

macro_rules! j_format_is_display {
    ($t:ty) => {
        impl JFormat for $t {
            fn j_format(&self) -> String {
                format!("{self}")
            }
        }
    };
}

j_format_is_display!(u8);
j_format_is_display!(char);

#[inline]
fn sign_lift<T: JFormat + num_traits::sign::Signed>(val: T, f: impl FnOnce(T) -> String) -> String {
    if val.is_negative() {
        format!("_{}", f(-val))
    } else {
        f(val)
    }
}

impl JFormat for i64 {
    fn j_format(&self) -> String {
        sign_lift(*self, |v| format!("{v}"))
    }
}
impl JFormat for BigInt {
    fn j_format(&self) -> String {
        // TODO: incredibly lazy clone
        sign_lift(self.clone(), |v| format!("{v}"))
    }
}

impl JFormat for f64 {
    fn j_format(&self) -> String {
        sign_lift(*self, |v| {
            if v.is_infinite() {
                format!("_")
            } else {
                format!("{v}")
            }
        })
    }
}

impl JFormat for JArray {
    fn j_format(&self) -> String {
        unreachable!()
    }
}

impl JFormat for BigRational {
    fn j_format(&self) -> String {
        if self.denom().is_one() {
            self.numer().j_format()
        } else {
            format!("{}r{}", self.numer().j_format(), self.denom().j_format())
        }
    }
}
impl JFormat for Complex64 {
    fn j_format(&self) -> String {
        if self.im.is_zero() {
            self.re.j_format()
        } else {
            format!("{}j{}", self.re.j_format(), self.im.j_format())
        }
    }
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
        assert_eq!(" 0  1\n 2  3\n 4  5\n\n 6  7\n 8  9\n10 11\n", s)
    }
}
