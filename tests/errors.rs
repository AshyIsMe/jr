use jr::JError;

#[test]
fn test_not_impl() -> Result<(), JError> {
    let err = jr::eval(jr::scan("'abc','def'")?).unwrap_err();
    assert_eq!(JError::NonceError, err);
    assert_eq!("feature not supported yet", format!("{}", err));
    Ok(())
}
