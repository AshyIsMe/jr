use anyhow::Result;
use jr::{Ctx, JError};

#[test]
fn test_not_impl() -> Result<()> {
    let err = jr::eval(jr::scan("'abc';:'def'")?, &mut Ctx::empty()).unwrap_err();
    let root = JError::extract(&err).expect("caused by jerror");
    assert!(matches!(root, JError::NonceError));
    assert_eq!("feature not supported yet", format!("{}", err.root_cause()));
    Ok(())
}
