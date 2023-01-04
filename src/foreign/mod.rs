mod conversion;
mod files;
mod global_param;
mod host;
mod locales;
mod names;
mod scripts;

use anyhow::{anyhow, Context, Result};
use std::sync::Arc;

use crate::JError;

use crate::verbs::PartialImpl;
use conversion::*;
use files::*;
use global_param::*;
use host::*;
use locales::*;
use names::*;
use scripts::*;

/// https://www.jsoftware.com/help/dictionary/xmain.htm
pub fn foreign(l: i64, r: i64) -> Result<PartialImpl> {
    let unsupported = |name: &'static str| -> Result<PartialImpl> {
        Err(JError::NonceError).with_context(|| anyhow!("unsupported {name} foreign: {l}!:{r}"))
    };

    let name = format!("{l}!:{r}");
    let mi = PartialImpl::mi;
    let m0 = PartialImpl::m0;

    let (monad, dyad) = match (l, r) {
        (0, k) => (mi(Arc::new(move |ctx, y| f_load_script(ctx, k, y))), None),
        (1, 1) => (m0(Arc::new(move |_, y| f_read_file(y))), None),
        (1, _) => return unsupported("file"),
        (2, 0) => (mi(Arc::new(|_, y| f_shell_out(y))), None),
        (2, 5) => (m0(Arc::new(|_, y| f_getenv(y))), None),
        (2, 6) => (mi(Arc::new(|_, _| f_getpid())), None),
        (2, _) => return unsupported("host"),
        (3, 3) => PartialImpl::from_legacy_inf(|_ctx, x, y| f_dump_hex(x, y)),
        (3, 4) => PartialImpl::from_legacy_inf(|_ctx, x, y| f_int_bytes(x, y)),
        (3, _) => return unsupported("conversion"),
        (4, 0) => (m0(Arc::new(|ctx, y| f_name_status(ctx, y))), None),
        (4, 1) => PartialImpl::from_legacy_inf(|ctx, x, y| f_name_namelist(ctx, x, y)),
        (4, 55) => (m0(Arc::new(|ctx, y| f_name_erase(ctx, y))), None),
        (5, _) => return unsupported("representation"),
        (6, _) => return unsupported("time"),
        (7, _) => return unsupported("space"),
        (8, _) => return unsupported("format"),
        (9, 12) => (mi(Arc::new(|_, _| f_os_type())), None),
        (9, _) => return unsupported("global param"),
        (13, _) => return unsupported("debug"),
        (15, _) => return unsupported("dll"),
        (18, 4) => (mi(Arc::new(|ctx, y| f_locales_set(ctx, y))), None),
        (18, _) => return unsupported("locales"),
        (128, _) => return unsupported("misc"),
        _ => return unsupported("major"),
    };

    Ok(PartialImpl { name, monad, dyad })
}
