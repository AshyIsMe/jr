use anyhow::{anyhow, Context, Result};
use jr::test_impls::run_j;
use std::fs;

fn main() -> Result<()> {
    let arg = std::env::args_os().nth(1).expect("usage: path to load");
    let arg = fs::canonicalize(&arg).with_context(|| anyhow!("lookup up {arg:?}"))?;
    let arg = arg
        .to_str()
        .ok_or_else(|| anyhow!("path must be valid utf-8"))?;
    let arg = arg.replace('\'', "''");
    let j = run_j(format!("(0!:0) <'{arg}'\n10 (6!:2) '10 comb 20'"))?;
    let j: f64 = j.parse().with_context(|| anyhow!("j said: {j}"))?;

    Ok(())
}
