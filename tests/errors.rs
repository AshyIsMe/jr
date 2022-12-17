use anyhow::Result;
use jr::test_impls::scan_eval;
use jr::JError;

#[test]
fn test_not_impl() -> Result<()> {
    let err = scan_eval("'abc';:'def'").unwrap_err();
    let root = JError::extract(&err).expect("caused by jerror");
    assert!(matches!(root, JError::NonceError));
    assert_eq!("feature not supported yet", format!("{}", err.root_cause()));
    Ok(())
}
