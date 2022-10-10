use std::collections::HashMap;

use anyhow::Result;
use jr::JError;

#[test]
fn test_not_impl() -> Result<()> {
    let err = jr::eval(jr::scan("'abc','def'")?, &mut HashMap::new()).unwrap_err();
    let root = dbg!(err.root_cause())
        .downcast_ref::<JError>()
        .expect("caused by jerror");
    assert!(matches!(root, JError::NonceError));
    assert_eq!(
        "NonceError: feature not supported yet",
        format!("{}", err.root_cause())
    );
    Ok(())
}
