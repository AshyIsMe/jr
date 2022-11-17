use std::fmt;

use ndarray::prelude::*;
use ndarray::Data;

use crate::{impl_array, JArray};

pub trait JDisplay {
    fn to_display(&self) -> String;
}

impl<A: fmt::Display, S, D: Dimension> JDisplay for ArrayBase<S, D>
where
    S: Data<Elem = A>,
{
    fn to_display(&self) -> String {
        let ops = FormatOptions::default_for_array(self.len(), false);
        format!("{}", FmtArrayBaseOpts(self, ops))
    }
}

impl JDisplay for JArray {
    fn to_display(&self) -> String {
        use JArray::*;
        match self {
            BoxArray(arr) => arr.to_display(),
            _ => impl_array!(self, |arr: &ArrayBase<_, _>| arr.to_display()),
        }
    }
}

/// Default threshold, below this element count, we don't ellipsize
const ARRAY_MANY_ELEMENT_LIMIT: usize = 500;
/// Limit of element count for non-last axes before overflowing with an ellipsis.
const AXIS_LIMIT_STACKED: usize = 6;
/// Limit for next to last axis (printed as column)
/// An odd number because one element uses the same space as the ellipsis.
const AXIS_LIMIT_COL: usize = 11;
/// Limit for last axis (printed as row)
/// An odd number because one element uses approximately the space of the ellipsis.
const AXIS_LIMIT_ROW: usize = 11;

/// The string used as an ellipsis.
const ELLIPSIS: &str = "...";

#[derive(Copy, Clone, Debug)]
struct FormatOptions {
    axis_collapse_limit: usize,
    axis_collapse_limit_next_last: usize,
    axis_collapse_limit_last: usize,
}

impl FormatOptions {
    fn default_for_array(nelem: usize, no_limit: bool) -> Self {
        let default = Self {
            axis_collapse_limit: AXIS_LIMIT_STACKED,
            axis_collapse_limit_next_last: AXIS_LIMIT_COL,
            axis_collapse_limit_last: AXIS_LIMIT_ROW,
        };
        default.set_no_limit(no_limit || nelem < ARRAY_MANY_ELEMENT_LIMIT)
    }

    fn set_no_limit(mut self, no_limit: bool) -> Self {
        if no_limit {
            self.axis_collapse_limit = std::usize::MAX;
            self.axis_collapse_limit_next_last = std::usize::MAX;
            self.axis_collapse_limit_last = std::usize::MAX;
        }
        self
    }

    /// Axis length collapse limit before ellipsizing, where `axis_rindex` is
    /// the index of the axis from the back.
    fn collapse_limit(&self, axis_rindex: usize) -> usize {
        match axis_rindex {
            0 => self.axis_collapse_limit_last,
            1 => self.axis_collapse_limit_next_last,
            _ => self.axis_collapse_limit,
        }
    }
}

/// Formats the contents of a list of items, using an ellipsis to indicate when
/// the `length` of the list is greater than `limit`.
///
/// # Parameters
///
/// * `f`: The formatter.
/// * `length`: The length of the list.
/// * `limit`: The maximum number of items before overflow.
/// * `separator`: Separator to write between items.
/// * `ellipsis`: Ellipsis for indicating overflow.
/// * `fmt_elem`: A function that formats an element in the list, given the
///   formatter and the index of the item in the list.
fn format_with_overflow(
    f: &mut fmt::Formatter<'_>,
    length: usize,
    limit: usize,
    separator: &str,
    ellipsis: &str,
    fmt_elem: &mut dyn FnMut(&mut fmt::Formatter, usize) -> fmt::Result,
) -> fmt::Result {
    if length == 0 {
        // no-op
    } else if length <= limit {
        fmt_elem(f, 0)?;
        for i in 1..length {
            f.write_str(separator)?;
            fmt_elem(f, i)?
        }
    } else {
        let edge = limit / 2;
        fmt_elem(f, 0)?;
        for i in 1..edge {
            f.write_str(separator)?;
            fmt_elem(f, i)?;
        }
        f.write_str(separator)?;
        f.write_str(ellipsis)?;
        for i in length - edge..length {
            f.write_str(separator)?;
            fmt_elem(f, i)?
        }
    }
    Ok(())
}

fn format_array<A, S, D, F>(
    array: &ArrayBase<S, D>,
    f: &mut fmt::Formatter<'_>,
    format: F,
    fmt_opt: &FormatOptions,
) -> fmt::Result
where
    F: FnMut(&A, &mut fmt::Formatter<'_>) -> fmt::Result + Clone,
    D: Dimension,
    S: Data<Elem = A>,
{
    // Cast into a dynamically dimensioned view
    // This is required to be able to use `index_axis` for the recursive case
    format_array_inner(array.view().into_dyn(), f, format, fmt_opt, 0, array.ndim())
}

fn format_array_inner<A, F>(
    view: ArrayView<A, IxDyn>,
    f: &mut fmt::Formatter<'_>,
    mut format: F,
    fmt_opt: &FormatOptions,
    depth: usize,
    full_ndim: usize,
) -> fmt::Result
where
    F: FnMut(&A, &mut fmt::Formatter<'_>) -> fmt::Result + Clone,
{
    // If any of the axes has 0 length, we return the same empty array representation
    // e.g. [[]] for 2-d arrays
    if view.is_empty() {
        write!(f, "{}{}", " ".repeat(view.ndim()), " ".repeat(view.ndim()))?;
        return Ok(());
    }
    match view.shape() {
        // If it's 0 dimensional, we just print out the scalar
        &[] => format(&view[[]], f)?,
        // We handle 1-D arrays as a special case
        &[len] => {
            let view = view.view().into_dimensionality::<Ix1>().unwrap();
            format_with_overflow(
                f,
                len,
                fmt_opt.collapse_limit(0),
                " ",
                ELLIPSIS,
                &mut |f, index| format(&view[index], f),
            )?;
        }
        // For n-dimensional arrays, we proceed recursively
        shape => {
            let blank_lines = "\n".repeat(shape.len() - 2);
            let separator = format!("\n{blank_lines}");

            let limit = fmt_opt.collapse_limit(full_ndim - depth - 1);
            format_with_overflow(f, shape[0], limit, &separator, ELLIPSIS, &mut |f, index| {
                format_array_inner(
                    view.index_axis(Axis(0), index),
                    f,
                    format.clone(),
                    fmt_opt,
                    depth + 1,
                    full_ndim,
                )
            })?;
        }
    }
    Ok(())
}

// this is just a hack to get hold of a fmt::Formatter, for the rest of the code
struct FmtArrayBaseOpts<'s, S, A: fmt::Display, D>(&'s ArrayBase<S, D>, FormatOptions)
where
    S: Data<Elem = A>;

impl<A: fmt::Display, S, D: Dimension> fmt::Display for FmtArrayBaseOpts<'_, S, A, D>
where
    S: Data<Elem = A>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fmt_opt = FormatOptions::default_for_array(self.0.len(), f.alternate());
        format_array(&self.0, f, <_>::fmt, &fmt_opt)
    }
}

#[cfg(test)]
mod tests {
    use crate::{arr0d, IntoJArray};
    use ndarray::prelude::*;
    use super::JDisplay;

    #[test]
    fn short() {
        assert_eq!("1", arr0d(1u8).into_jarray().to_display());
        assert_eq!("2 4 8", array![2i64, 4, 8].into_dyn().into_jarray().to_display());
        assert_eq!("2 4\n6 8", array![[2i64, 4], [6, 8]].into_dyn().into_jarray().to_display());
    }
}
