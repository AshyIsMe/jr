use anyhow::{ensure, Context, Result};

use crate::{arr0d, JArray, JError};

pub fn f_shell_out(y: &JArray) -> Result<JArray> {
    let JArray::CharArray(y) = y else { return Err(JError::NonceError).context("string required") };
    // TODO: new lines
    let y = y.iter().collect::<String>();
    let result = match y.as_str() {
        "uname" if cfg!(target_os = "linux") => "Linux",
        _ => return Err(JError::NonceError).context("shelling out is disabled"),
    };

    Ok(JArray::from_string(result))
}

pub fn f_getenv(y: &JArray) -> Result<JArray> {
    ensure!(y.shape().len() <= 1, "rank 0");
    let JArray::CharArray(y) = y else { return Err(JError::NonceError).context("string required") };
    let y = y.iter().collect::<String>();
    use std::env::VarError;
    match std::env::var(y) {
        Ok(result) => Ok(JArray::from_string(result)),
        Err(VarError::NotPresent) => Ok(JArray::from(arr0d(0u8))),
        Err(VarError::NotUnicode(_)) => {
            Err(JError::DomainError).context("environment variable not representable")
        }
    }
}

pub fn f_getpid() -> Result<JArray> {
    Ok(JArray::from(arr0d(i64::from(std::process::id()))))
}
