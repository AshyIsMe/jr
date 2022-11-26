use std::fs;

use anyhow::Result;
use itertools::Itertools;
use jr::test_impls::run_j;

fn main() -> Result<()> {
    let contents = fs::read_to_string(std::env::args_os().nth(1).expect("usage: file.ijs"))
        .expect("reading input file");
    let tests = contents.lines().enumerate().collect_vec();

    println!("use jr::test_impls::run_to_string as run;");
    println!();

    for (line, expr) in tests {
        eprintln!("line {}, running {expr:?}", line + 1);
        let j = run_j(&expr)?;
        println!("#[test]");
        println!("fn test_{line}() {{");
        println!("    assert_eq!(");
        println!("        run({expr:?}).unwrap(),");
        println!("        {j:?},");
        println!("        {expr:?},");
        println!("    );");
        println!("}}");
        println!();
    }

    Ok(())
}
