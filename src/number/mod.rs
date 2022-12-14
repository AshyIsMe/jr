mod number;
mod promote;

pub use number::{float_is_int, Num};
pub use promote::{
    elems_to_jarray, infer_kind_from_boxes, infer_kind_from_elems, promote_to_array,
};
