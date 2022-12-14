use anyhow::{Context, Result};
use jr::test_impls::{read_ijs_dir, read_ijs_lines, test_against, RunList};

#[test]
fn smoke() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    let lookup = RunList::open(include_str!("smoke.toml"))?.into_lookup()?;
    for (ctx, expr) in read_ijs_lines(include_str!("smoke.ijs")) {
        test_against(expr, &lookup).context(ctx)?;
    }
    Ok(())
}

#[test]
fn snippets() -> Result<()> {
    let _ = env_logger::builder().is_test(true).try_init();
    let lookup = RunList::open(include_str!("snippets.toml"))?.into_lookup()?;
    for (ctx, expr) in read_ijs_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snippets"))? {
        // if !ctx.contains("func-02") { continue; }
        test_against(expr, &lookup).context(ctx)?;
    }
    Ok(())
}
