use anyhow::{Context, Result};

use crate::{arr0d, JArray, JError, Num};

pub fn f_type(y: &JArray) -> Result<JArray> {
    use JArray::*;
    match y {
        BoolArray(_) => Ok(JArray::from(arr0d(1i64))),
        IntArray(_) => Ok(JArray::from(arr0d(4i64))),
        ExtIntArray(_) => Ok(JArray::from(arr0d(64i64))),
        RationalArray(_) => Ok(JArray::from(arr0d(128i64))),
        FloatArray(_) => Ok(JArray::from(arr0d(8i64))),
        ComplexArray(_) => Ok(JArray::from(arr0d(16i64))),
        CharArray(_) => Ok(JArray::from(arr0d(2i64))),
        BoxArray(_) => Ok(JArray::from(arr0d(32i64))),
        // UnicodeCharArray(_) => Ok(JArray::from(arr0d(131072i64))),
        // Unicode4CharArray(_) => Ok(JArray::from(arr0d(262144i64))),
        // SymbolArray(_) => Ok(JArray::from(arr0d(65536i64))),
        // SparseBoolArray(_) => Ok(JArray::from(arr0d(1024i64))),
        // SparseCharArray(_) => Ok(JArray::from(arr0d(2048i64))),
        // SparseIntArray(_) => Ok(JArray::from(arr0d(4096i64))),
        // SparseFloatArray(_) => Ok(JArray::from(arr0d(8192i64))),
        // SparseComplexArray(_) => Ok(JArray::from(arr0d(16384i64))),
        // SparseBoxArray(_) => Ok(JArray::from(arr0d(32768i64))),
    }
}

pub fn f_dump_hex(x: Option<&JArray>, y: &JArray) -> Result<JArray> {
    if cfg!(not(target_pointer_width = "64")) {
        return Err(JError::NonceError).context("only support 64-bit (laziness)");
    }

    if cfg!(not(target_endian = "little")) {
        return Err(JError::NonceError).context("haha, very funny");
    }

    match x {
        Some(x) => match x.single_math_num() {
            Some(x) if x == Num::Int(3) || x == Num::Int(11) => (),
            _ => return Err(JError::NonceError).context("unsupported mode"),
        },
        None => (),
    }

    let mut result = Vec::with_capacity(8);
    result.push(0xe3); // 64-bit, reversed

    match y {
        JArray::IntArray(arr) => {
            result.push(4);
            // note: not JArray.len()
            result.push(i64::try_from(arr.len())?);
            result.push(i64::try_from(arr.shape().len())?);
            for shape in arr.shape() {
                result.push(i64::try_from(*shape)?);
            }
            result.extend(arr.iter().copied());
        }

        _ => return Err(JError::NonceError).context("only int arrays (don't ask)"),
    }

    JArray::from_fill_promote(
        result
            .into_iter()
            .map(|x| JArray::from_string(format!("{:016x}", x.to_be()))),
    )
}

pub fn f_int_bytes(x: Option<&JArray>, y: &JArray) -> Result<JArray> {
    let Some(x) = x else { return Err(JError::DomainError).context("invalid mode type"); };
    match x.single_math_num() {
        Some(x) if x == Num::Int(2) => (),
        _ => return Err(JError::NonceError).context("unsupported mode"),
    }
    let Some(y) = y.single_math_num() else { return Err(JError::NonceError).context("only single numbers"); };
    let Some(y) = y.value_i64() else  { return Err(JError::NonceError).context("only integers"); };
    if y < 32 && y > 128 {
        return Err(JError::NonceError).context("ascii only");
    }

    Ok(JArray::from_list(
        // thanks, I hate it
        vec![y as u8 as char, '\0', '\0', '\0'],
    ))
}
