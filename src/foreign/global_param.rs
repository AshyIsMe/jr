use anyhow::Result;

use crate::{arr0ad, JArray};

pub fn f_os_type() -> Result<JArray> {
    Ok(JArray::IntArray(arr0ad(match std::env::consts::OS {
        "linux" => 5i64,
        "windows" => 6,
        _ => -1,
    })))
}

pub fn f_is_secure() -> Result<JArray> {
    // it's rust, of course it's secure
    Ok(JArray::IntArray(arr0ad(1)))
}
