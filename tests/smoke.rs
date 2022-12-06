use anyhow::{anyhow, bail, Context, Result};
use jr::test_impls::{scan_eval, Run, RunList};
use jr::Word;

#[test]
fn smoke() -> Result<()> {
    for run in RunList::open(include_str!("smoke.toml"))
        .expect("static asset")
        .runs
    {
        exec(&run).with_context(|| anyhow!("testing {:?}", run.expr))?;
    }
    Ok(())
}

fn exec(run: &Run) -> Result<()> {
    let them = run.parse_encoded().context("rehydrating j output")?;
    let us = scan_eval(&run.expr).context("running expression in smoke test")?;
    let Word::Noun(arr) = us else { bail!("unexpected non-array from eval: {us:?}") };

    if arr == them {
        return Ok(());
    }

    Err(anyhow!("incorrect data, we got:\n{arr:?}\n\nThey expect:\n{them:?}\n\njsoft would render this like this:\n{}", run.output))
}
