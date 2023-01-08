mod conversion;
mod files;
mod global_param;
mod host;
mod locales;
mod names;
mod scripts;

use anyhow::{anyhow, Context, Result};

use crate::{JError, Rank};

use crate::verbs::BivalentOwned;
use conversion::*;
use files::*;
use global_param::*;
use host::*;
use locales::*;
use names::*;
use scripts::*;

/// https://www.jsoftware.com/help/dictionary/xmain.htm
pub fn foreign(l: i64, r: i64) -> Result<BivalentOwned> {
    let unsupported = |name: &'static str| -> Result<BivalentOwned> {
        Err(JError::NonceError).with_context(|| anyhow!("unsupported {name:?} foreign: {l}!:{r}"))
    };

    let iii = Rank::inf_inf_inf();
    let zii = (Rank::zero(), Rank::infinite_infinite());

    let unimplemented = |name: &'static str| {
        (
            iii,
            BivalentOwned::from_bivalent(move |_, _, _| {
                Err(JError::NonceError)
                    .with_context(|| anyhow!("unimplemented {name:?} foreign ({l}!:{r})"))
            }),
        )
    };
    let security_violation = |name: &'static str| {
        (
            iii,
            BivalentOwned::from_bivalent(move |_, _, _| {
                Err(JError::SecurityViolation).with_context(|| anyhow!("disallowed: {name}"))
            }),
        )
    };

    let (ranks, biv) = match (l, r) {
        (0, k) => (
            iii,
            BivalentOwned::from_monad(move |ctx, y| f_load_script(ctx, k, y)),
        ),
        (1, 0) => unimplemented("list dir"),
        (1, 1) => (zii, BivalentOwned::from_monad(|_, y| f_read_file(y))),
        (1, 2) => unimplemented("write file"),
        (1, 3) => unimplemented("append file"),
        (1, 4) => (zii, BivalentOwned::from_monad(|_, y| f_file_size(y))),
        (1, 5) => security_violation("mkdir"),
        (1, 6) => unimplemented("file attributes"),
        (1, 7) => unimplemented("file permissions"),
        (1, 11) => unimplemented("offset read"),
        (1, 12) => unimplemented("offset write"),
        (1, 20) => unimplemented("list file handles"),
        (1, 21) => unimplemented("open file handle"),
        (1, 22) => unimplemented("close file handle"),
        (1, 30) => unimplemented("list file locks"),
        (1, 31) => unimplemented("lock file region"),
        (1, 32) => unimplemented("unlock file region"),
        (1, 43) => (iii, BivalentOwned::from_monad(|_, _| f_file_cwd())),
        (1, 44) => unimplemented("change dir"),
        (1, 46) => unimplemented("path of j.dll"),
        (1, 55) => unimplemented("erase directory"),
        (1, _) => return unsupported("file"),
        (2, 0) => (iii, BivalentOwned::from_monad(|_, y| f_shell_out(y))),
        (2, 1) => security_violation("shell out (forked)"),
        (2, 3) => unimplemented("terminate and wait"),
        (2, 5) => (zii, BivalentOwned::from_monad(|_, y| f_getenv(y))),
        (2, 6) => (iii, BivalentOwned::from_monad(|_, _| f_getpid())),
        (2, 7) => unimplemented("get cpu features"),
        (2, 8) => unimplemented("errno"),
        (2, 55) => unimplemented("terminate session"),
        (2, _) => return unsupported("host"),
        (3, 0) => unimplemented("type"),
        (3, 1) => unimplemented("byte representation"),
        (3, 2) => unimplemented("byte to hex representation"),
        (3, 3) => (
            iii,
            BivalentOwned::from_bivalent(|_ctx, x, y| f_dump_hex(x, y)),
        ),
        (3, 4) => (
            iii,
            BivalentOwned::from_bivalent(|_ctx, x, y| f_int_bytes(x, y)),
        ),
        (3, 5) => unimplemented("floating point conversions"),
        (3, 6) => unimplemented("lock script"),
        (3, 10) => unimplemented("to base64"),
        (3, 11) => unimplemented("from base64"),
        (3, 12) => unimplemented("to/from uppercase"),
        (3, _) => return unsupported("conversion"),
        (4, 0) => (
            zii,
            BivalentOwned::from_monad(|ctx, y| f_name_status(ctx, y)),
        ),
        (4, 1) => (
            iii,
            BivalentOwned::from_bivalent(|ctx, x, y| f_name_namelist(ctx, x, y)),
        ),
        (4, 3) => unimplemented("list loaded scripts"),
        (4, 4) => unimplemented("find loaded script"),
        (4, 5) => unimplemented("name change tracing"),
        (4, 6) => unimplemented("set current script name"),
        (4, 7) => unimplemented("set current script number"),
        (4, 8) => unimplemented("create cached reference"),
        (4, 55) => (
            zii,
            BivalentOwned::from_monad(|ctx, y| f_name_erase(ctx, y)),
        ),
        (5, 0) => unimplemented("create from ar"),
        (5, 1) => unimplemented("create ar"),
        (5, 2) => unimplemented("ar boxed"),
        (5, 4) => unimplemented("ar tree"),
        (5, 5) => unimplemented("ar linear"),
        (5, 6) => unimplemented("ar linear (paren)"),
        (5, 7) => unimplemented("something to do with valences"),
        (5, _) => return unsupported("representation"),
        (6, 0) => unimplemented("time current"),
        (6, 1) => unimplemented("time session"),
        (6, 2) => unimplemented("time sentence"),
        (6, 3) => unimplemented("time delay"),
        (6, 4) => unimplemented("time pm count calls"),
        (6, 8) => unimplemented("time pm clock frequency"),
        (6, 9) => unimplemented("time pm clock counter"),
        (6, 10) => unimplemented("time pm data"),
        (6, 11) => unimplemented("time pm unpack"),
        (6, 12) => unimplemented("time pm add"),
        (6, 13) => unimplemented("time pm stats"),
        (6, 14) => unimplemented("time to nano"),
        (6, 15) => unimplemented("time to expanded"),
        (6, 16) => unimplemented("time to 8601"),
        (6, 17) => unimplemented("time to 8601 nano"),
        (6, 18) => unimplemented("time string to nano"),
        (6, _) => return unsupported("time"),
        (7, 0) => unimplemented("space use"),
        (7, 1) => unimplemented("space memory use"),
        (7, 2) => unimplemented("space measure"),
        (7, 3) => unimplemented("space free"),
        (7, 5) => unimplemented("space for object"),
        (7, 6) => unimplemented("space for locale"),
        (7, 7) => unimplemented("space by os"),
        (7, 8) => unimplemented("space for locale all"),
        (8, _) => return unsupported("format"),
        (9, 12) => (iii, BivalentOwned::from_monad(|_, _| f_os_type())),
        (9, _) => return unsupported("global param"),
        (13, 8) => unimplemented("signal error"),
        (13, _) => return unsupported("debug"),
        (15, 0) => unimplemented("call dll"),
        (15, 10) => unimplemented("dll error code"),
        (15, _) => return unsupported("dll"),
        (18, 3) => unimplemented("create locale"),
        (18, 4) => (
            iii,
            BivalentOwned::from_monad(|ctx, y| f_locales_set(ctx, y)),
        ),
        (18, _) => return unsupported("locales"),
        (128, 2) => unimplemented("eval and apply"),
        (128, _) => return unsupported("misc"),
        _ => return unsupported("major"),
    };

    Ok(BivalentOwned { biv, ranks })
}
