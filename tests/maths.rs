use jr::test_impls::run_to_string as run;

#[test]
#[ignore]
fn test_0() {
    assert_eq!(
        run("(i. 9) -. 2 3 5 7").unwrap(),
        "0 1 4 6 8",
        "(i. 9) -. 2 3 5 7",
    );
}

#[test]
#[ignore]
fn test_1() {
    assert_eq!(
        run("2 3 4 5 -. 'abcdef'").unwrap(),
        "2 3 4 5",
        "2 3 4 5 -. 'abcdef'",
    );
}
