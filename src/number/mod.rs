mod number;
mod promote;

pub use number::{float_is_int, Num};
pub use promote::{infer_kind_from_boxes, Promote};
