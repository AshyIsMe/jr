use anyhow::Result;
use jr::test_impls::{read_ijs_lines, test_against, RunList};

#[test]
fn smoke() -> Result<()> {
    let lookup = RunList::open(include_str!("smoke.toml"))?.into_lookup()?;
    for expr in read_ijs_lines(include_str!("smoke.ijs")) {
        test_against(expr, &lookup)?;
    }
    Ok(())
}
