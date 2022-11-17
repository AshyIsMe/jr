use anyhow::Result;

use jr::test_impls::{run_j, scan_eval, to_arr};
use jr::{generate_cells, JArray, Rank, Word};

fn main() -> Result<()> {
    env_logger::init();
    assert_eq!("69".to_string(), run_j("69")?);

    for (x, op, y, (xr, yr)) in [
        ("1 2 3 4", "+", "10 20 30 40", (0, 0)),
        ("10", "+", "1 2 3 4", (0, 0)),
        ("1 2", "+", "2 3 $ 10 20 30 70 80 90", (0, 0)),
        ("2 3", "$", "10 20 30 40 50 60", (1, u32::MAX)),
        ("(3 1 $ 2 3 4)", "$", "0 1 2 3", (1, u32::MAX)),
        ("3", "$", "(2 2 $ 5 6 7 8)", (1, u32::MAX)),
    ] {
        let expr = format!("{x} {op} {y}");
        println!("e: {expr}    NB. {op} is {xr} {yr}");
        println!("j: {}", run_j(&expr)?.replace('\n', "\n   "));
        let res = scan_eval(&expr);
        println!("r: {}", format!("{:?}", res).replace('\n', "\n   "));

        let (c, common, spare) =
            generate_cells(to_arr(x)?, to_arr(y)?, (Rank::new(xr)?, Rank::new(yr)?))?;
        println!("g: {c:?} {common:?} {spare:?}");
        println!();
    }
    Ok(())
}
