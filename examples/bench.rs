use anyhow::{anyhow, Context, Result};
use jr::test_impls::{run_j, scan_eval};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let arg = std::env::args_os().nth(1).expect("usage: path to load");
    let arg = fs::canonicalize(&arg).with_context(|| anyhow!("lookup up {arg:?}"))?;
    let content = fs::read_to_string(&arg).context("loading")?;
    scan_eval(&content)?;

    if false {
        test_j(&arg)?;
    }

    Ok(())
}

fn test_j(arg: &Path) -> Result<()> {
    let arg = arg
        .to_str()
        .ok_or_else(|| anyhow!("path must be valid utf-8"))?;
    let arg = arg.replace('\'', "''");
    let j = run_j(format!("(0!:0) <'{arg}'\n10 (6!:2) '10 comb 20'"))?;
    let _j: f64 = j.parse().with_context(|| anyhow!("j said: {j}"))?;
    Ok(())
}
