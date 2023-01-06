use anyhow::Result;

use crate::{arr0d, JArray};

pub fn f_os_type() -> Result<JArray> {
    Ok(JArray::IntArray(arr0d(match std::env::consts::OS {
        "linux" => 5i64,
        "windows" => 6,
        _ => -1,
    })))
}
