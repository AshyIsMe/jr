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
    let them = run.parse_encoded().context("rehydrating j output")?;
    let us = scan_eval(&run.expr).context("running expression in smoke test")?;
    let Word::Noun(arr) = us else { bail!("unexpected non-array from eval: {us:?}") };
    let them_shape = run.parse_shape()?;

    if arr != them {
        bail!("incorrect data, we got {arr:?}, they expect {them:?}",);
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
        JArray::BoxArray(_) => "boxed",
        JArray::LiteralArray(_) => "literal",
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
