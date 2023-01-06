mod conversion;
mod files;
mod global_param;
mod host;
mod locales;
mod names;
mod scripts;

use anyhow::{anyhow, Context, Result};

use crate::{JError, Rank};

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
        Err(JError::NonceError).with_context(|| anyhow!("unsupported {name:?} foreign: {l}!:{r}"))
    };

    let unimplemented = |name: &'static str| -> Result<PartialImpl> {
        let biv = PartialImpl::from_legacy_inf(move |_ctx, _x, _y| {
            Err(JError::NonceError)
                .with_context(|| anyhow!("unimplemented {name:?} foreign ({l}!:{r})"))
        });
        Ok(PartialImpl {
            name: name.to_string(),
            monad: None,
            dyad: None,
            biv,
            ranks: Rank::inf_inf_inf(),
        })
    };

    let name = format!("{l}!:{r}");

    let iii = Rank::inf_inf_inf();
    let zii = (Rank::zero(), Rank::infinite_infinite());

    let (ranks, biv) = match (l, r) {
        (0, k) => (
            iii,
            PartialImpl::from_monad(move |ctx, y| f_load_script(ctx, k, y)),
        ),
        (1, 1) => (zii, PartialImpl::from_monad(|_, y| f_read_file(y))),
        (1, _) => return unsupported("file"),
        (2, 0) => (iii, PartialImpl::from_monad(|_, y| f_shell_out(y))),
        (2, 5) => (zii, PartialImpl::from_monad(|_, y| f_getenv(y))),
        (2, 6) => (iii, PartialImpl::from_monad(|_, _| f_getpid())),
        (2, _) => return unsupported("host"),
        (3, 3) => (
            iii,
            PartialImpl::from_legacy_inf(|_ctx, x, y| f_dump_hex(x, y)),
        ),
        (3, 4) => (
            iii,
            PartialImpl::from_legacy_inf(|_ctx, x, y| f_int_bytes(x, y)),
        ),
        (3, _) => return unsupported("conversion"),
        (4, 0) => (zii, PartialImpl::from_monad(|ctx, y| f_name_status(ctx, y))),
        (4, 1) => (
            iii,
            PartialImpl::from_legacy_inf(|ctx, x, y| f_name_namelist(ctx, x, y)),
        ),
        (4, 55) => (zii, PartialImpl::from_monad(|ctx, y| f_name_erase(ctx, y))),
        (5, _) => return unsupported("representation"),
        (6, _) => return unsupported("time"),
        (7, _) => return unsupported("space"),
        (8, _) => return unsupported("format"),
        (9, 12) => (iii, PartialImpl::from_monad(|_, _| f_os_type())),
        (9, _) => return unsupported("global param"),
        (13, _) => return unsupported("debug"),
        (15, 0) => return unimplemented("call dll"),
        (15, 10) => return unimplemented("dll error code"),
        (15, _) => return unsupported("dll"),
        (18, 4) => (iii, PartialImpl::from_monad(|ctx, y| f_locales_set(ctx, y))),
        (18, _) => return unsupported("locales"),
        (128, _) => return unsupported("misc"),
        _ => return unsupported("major"),
    };

    Ok(PartialImpl {
        name,
        monad: None,
        dyad: None,
        biv,
        ranks,
    })
}
