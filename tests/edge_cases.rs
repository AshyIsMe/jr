#[test]
fn invalid_one_letter() {
    // TODO: error matcher / diagnostics
    assert!(jr::scan("q").is_err());
}

#[test]
fn invalid_comma() {
    // TODO: error matcher / diagnostics
    assert!(jr::scan(",").is_err());
}

#[test]
fn invalid_prime() {
    // TODO: error matcher / diagnostics
    assert!(jr::scan("'").is_err());
}
