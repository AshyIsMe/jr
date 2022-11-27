use anyhow::{anyhow, bail, Context, Result};
use jr::test_impls::{scan_eval, Run, RunList};
use jr::{JArray, Word};

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
    let us = scan_eval(&run.expr).context("running expression in smoke test")?;
    let Word::Noun(arr) = us else { bail!("unexpected non-array from eval: {us:?}") };
    let them_shape = run.parse_shape()?;

    let them_data = run
        .parse_data()
        .with_context(|| anyhow!("interpreting the generated jsoft output: {:?}", run.output))?;
    let us_data = arr
        .clone()
        .into_nums()
        .ok_or_else(|| anyhow!("not handling non-num results"))?;
    if us_data != them_data {
        bail!(
            "incorrect data, we got {:?}, they expect {:?}",
            us_data,
            them_data
        );
    }

    if arr.shape() != them_shape {
        bail!(
            "incorrect shape, we got {:?}, they expect {:?}",
            arr.shape(),
            them_shape
        );
    }

    let our_type = match arr {
        JArray::IntArray(_) => "integer",
        JArray::BoolArray(_) => "boolean",
        JArray::CharArray(_) => "character",
        JArray::ExtIntArray(_) => "extended",
        JArray::RationalArray(_) => "rational",
        JArray::FloatArray(_) => "floating",
        JArray::ComplexArray(_) => "complex",
        JArray::BoxArray(_) => "box",
    };

    if our_type != run.datatype {
        bail!(
            "incorrect datatype, we got {:?}, they expect {:?}",
            our_type,
            run.datatype
        );
    }

    Ok(())
}
