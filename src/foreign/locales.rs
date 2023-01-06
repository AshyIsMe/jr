use anyhow::{Context, Result};

use crate::foreign::files::arg_to_string;
use crate::{Ctx, HasEmpty, JArray, JError};

pub fn f_locales_set(ctx: &mut Ctx, y: &JArray) -> Result<JArray> {
    let JArray::BoxArray(y) = y else { return Err(JError::DomainError).context("boxed name please"); };
    let y = arg_to_string(y)?;
    ctx.eval_mut().locales.current = y;
    Ok(JArray::empty())
}
