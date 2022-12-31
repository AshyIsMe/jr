#![allow(clippy::enum_variant_names)]

mod cow;
mod elem;
mod into_vec;
mod nd_ext;
mod owned;
mod plural;
mod word;

pub use cow::{CowArrayD, JArrayCow};
pub use elem::Elem;
pub use into_vec::IntoVec;
pub use nd_ext::*;
pub use owned::{BoxArray, JArray, JArrayKind};
pub use plural::JArrays;
pub use word::Word;

// All terminology should match J terminology:
// Glossary: https://code.jsoftware.com/wiki/Vocabulary/Glossary

#[macro_export]
macro_rules! reduce_arrays {
    ($arr:expr, $func:expr) => {
        match $arr {
            JArrays::BoolArrays(ref a) => JArray::BoolArray($func(a)?),
            JArrays::CharArrays(ref a) => JArray::CharArray($func(a)?),
            JArrays::IntArrays(ref a) => JArray::IntArray($func(a)?),
            JArrays::ExtIntArrays(ref a) => JArray::ExtIntArray($func(a)?),
            JArrays::RationalArrays(ref a) => JArray::RationalArray($func(a)?),
            JArrays::FloatArrays(ref a) => JArray::FloatArray($func(a)?),
            JArrays::ComplexArrays(ref a) => JArray::ComplexArray($func(a)?),
            JArrays::BoxArrays(ref a) => JArray::BoxArray($func(a)?),
        }
    };
}
