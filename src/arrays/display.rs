use itertools::Itertools;
use log::debug;
use std::cmp::max;
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
        //JArray::BoxArray(arr) => br_box(f, arr.view()),
        JArray::BoxArray(arr) => br_box_aa_wrong(f, arr.view()),
        JArray::CharArray(arr) => br_char(f, arr.view()),
        _ => impl_array!(arr, |arr: &ArrayBase<_, _>| br(&mut f, arr.view())),
    }
}

trait JFormat {
    fn j_format(&self) -> String;
}

fn width(s: impl AsRef<str>) -> usize {
    s.as_ref().width()
}

fn short_array_cases<T: JFormat>(arr: &ArrayViewD<T>) -> Option<String> {
    if arr.is_empty() {
        // what on earth is even going on
        Some(if arr.shape().len() == 1 {
            "\n".to_string()
        } else {
            String::new()
        })
    } else if arr.shape().is_empty() {
        Some(format!(
            "{}\n",
            arr.first().expect("atom has an element").j_format()
        ))
    } else {
        None
    }
}

fn br<T: JFormat>(mut f: impl fmt::Write, arr: ArrayViewD<T>) -> fmt::Result {
    if let Some(s) = short_array_cases(&arr) {
        return write!(f, "{s}");
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
                .map(|(l, r)| max(l, r))
                .collect_vec()
        })
        .expect("non-empty rows");

    let multiples = compute_dimension_spacing(&arr);

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
        print_dimension_markings(&mut f, rn, &multiples)?;
    }

    Ok(())
}

// fn br_box(mut f: impl fmt::Write, arr: ArrayViewD<JArray>) -> fmt::Result {
//     if let Some(s) = short_array_cases(&arr) {
//         return write!(f, "{s}");
//     }
//     let corners = ['┌', '┐', '└', '┘'];
//     let walls = ['─', '│', '┼'];
//     let limit = 128usize;
//     // TODO: jsoft takes from the end, not the start, for some reason
//     let iter = arr.rows().into_iter().enumerate().take(limit);
//     let table = iter
//         .map(|(p, x)| {
//             (
//                 p,
//                 x.into_iter()
//                     .take(limit)
//                     .map(|x| x.j_format())
//                     .collect_vec(),
//             )
//         })
//         .collect_vec();

//     let widths: Vec<usize> = table
//         .iter()
//         .map(|(_, row)| {
//             row.iter()
//                 .map(|item| item.split('\n').map(width).max().unwrap_or_default())
//                 .collect_vec()
//         })
//         .reduce(|l, r| {
//             l.into_iter()
//                 .zip(r.into_iter())
//                 .map(|(l, r)| max(l, r))
//                 .collect_vec()
//         })
//         .expect("non-empty rows");

//     let widths: Vec<usize> = table
//         .iter()
//         .map(|(_, row)| {
//             row.iter()
//                 .map(|item| item.split('\n').map(width).max().unwrap_or_default())
//                 .collect_vec()
//         })
//         .reduce(|l, r| {
//             l.into_iter()
//                 .zip(r.into_iter())
//                 .map(|(l, r)| max(l, r))
//                 .collect_vec()
//         })
//         .expect("non-empty rows");

//     // let row_sizes = table
//     //     .iter()
//     //     .map(|(_, row)| {
//     //         row.iter()
//     //             .map(|item| {
//     //                 let lines = item.split('\n').count();
//     //                 let max_width = item
//     //                     .split('\n')
//     //                     .map(|line| width(line))
//     //                     .max()
//     //                     .unwrap_or_default();
//     //                 (lines, max_width)
//     //             })
//     //             .collect_vec()
//     //     }).collect_vec();
//     // let widths = row_sizes.iter().map(|(_lines, width)| width)
//     //     .reduce(|l, r| {
//     //         l.into_iter()
//     //             .zip(r.into_iter())
//     //             .map(|(l, r)| *l.max(r))
//     //             .collect_vec()
//     //     })
//     //     .expect("non-empty rows");

//     let multiples = compute_dimension_spacing(&arr);

//     let last = table.last().expect("non-empty").0;

//     for (rn, row) in table {
//         for (col, (target, item)) in row_sizes.iter().zip(row.into_iter()).enumerate() {
//             let len = width(&item);
//             write!(
//                 f,
//                 "{}",
//                 (0..(target - len)).map(|_| ' ').collect::<String>()
//             )?;
//             if col != 0 {
//                 write!(f, " ")?;
//             }
//             write!(f, "{item}")?;
//         }
//         write!(f, "\n")?;
//         if rn == last {
//             break;
//         }
//         print_dimension_markings(&mut f, rn, &multiples)?;
//     }

//     Ok(())
// }

fn br_box_aa_wrong(mut f: impl fmt::Write, arr: ArrayViewD<JArray>) -> fmt::Result {
    if let Some(s) = short_array_cases(&arr) {
        return write!(f, "{s}");
    }
    let corners = ['┌', '┐', '└', '┘'];
    let walls = ['─', '│', '┼'];
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

    let row_max_heights: Vec<usize> = table
        .iter()
        .map(|(_, row)| {
            row.iter()
                .map(|item| item.split('\n').count())
                .max()
                .unwrap_or_default()
        })
        .collect_vec();

    let col_max_widths: Vec<usize> = table
        .iter()
        .map(|(_, row)| {
            row.iter()
                .map(|item| item.split('\n').map(width).max().unwrap_or_default())
                .collect_vec()
        })
        .reduce(|l, r| {
            l.into_iter()
                .zip(r.into_iter())
                .map(|(l, r)| max(l, r))
                .collect_vec()
        })
        .expect("non-empty rows");

    debug!("arr.shape(): {:?}", arr.shape());
    debug!("table: {:?}", table);
    debug!("row_max_heights: {:?}", row_max_heights);
    debug!("col_max_widths: {:?}", col_max_widths);

    // pad each item according to row_max_heights, col_max_widths
    // AA TODO each box should be correct in isolation but the inputs seem wrong already
    let boxes = table
        .iter()
        .map(|(rn, row)| {
            row.iter()
                .enumerate()
                .map(|(cn, item)| {
                    let lines = item.split('\n').collect_vec();
                    let target_height = row_max_heights[*rn];
                    let target_width = col_max_widths[cn];
                    let missing_height = target_height - lines.len();
                    let lines = lines
                        .iter()
                        .chain(std::iter::repeat(&" \n").take(missing_height))
                        .collect_vec();
                    let lines = lines
                        .iter()
                        .map(|l| {
                            debug!("l: {:?}", l);
                            let missing_width = target_width - l.len();
                            let tail: String = std::iter::repeat(" ").take(missing_width).collect();
                            vec![l.to_string(), tail].join("")
                        })
                        .collect_vec();
                    lines.join("\n")
                })
                .collect_vec()
        })
        .collect_vec();

    debug!("boxes: {:?}", boxes);
    // AA TODO Now join each box together into rows/cols with box chars surrounding them

    for (i, row) in table.iter() {
        for item in row.iter() {
            write!(f, "{item}");
        }
    }

    Ok(())
}

fn br_char(mut f: impl fmt::Write, arr: ArrayViewD<char>) -> fmt::Result {
    if let Some(s) = short_array_cases(&arr) {
        return write!(f, "{s}");
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
                    // look, just don't ask
                    .filter(|c| **c != '\0')
                    .collect::<String>(),
            )
        })
        .collect_vec();

    let multiples = compute_dimension_spacing(&arr);

    let last = table.last().expect("non-empty").0;

    for (rn, row) in table {
        write!(f, "{row}\n")?;
        if rn == last {
            break;
        }
        print_dimension_markings(&mut f, rn, &multiples)?;
    }

    Ok(())
}

fn compute_dimension_spacing<T>(arr: &ArrayViewD<T>) -> Vec<usize> {
    arr.shape()
        .iter()
        .rev()
        .skip(1)
        .fold(Vec::<usize>::new(), |mut acc, &x| {
            let t: usize = acc.iter().product();
            acc.push(x * t);
            acc
        })
}

fn print_dimension_markings(mut f: impl fmt::Write, rn: usize, multiples: &[usize]) -> fmt::Result {
    for &m in multiples {
        if (rn + 1) % m == 0 {
            write!(f, "\n")?;
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
        let mut ret = String::with_capacity(self.tally() * 2);
        jsoft(&mut ret, self).expect("TODO: nested array panic?");
        ret
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
