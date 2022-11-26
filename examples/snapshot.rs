use std::str::FromStr;

use anyhow::{bail, Result};
use jr::test_impls::{run_j, scan_eval};

#[derive(Debug, Copy, Clone, PartialEq)]
enum Expectation {
    Identical,
    Success,
    DomainError,
    LengthError,
    Failure,
    Comment,
}

fn main() -> Result<()> {
    let tests = include_str!("snapshot.txt")
        .lines()
        .enumerate()
        .map(|(line, x)| {
            Ok(match x.split_once(' ') {
                Some((x, y)) => (line, x.parse()?, y.to_string()),
                None if x.is_empty() => (line, Expectation::Comment, String::new()),
                None => bail!("lines must contain a space, not {x:?}"),
            })
        })
        .collect::<Result<Vec<(usize, Expectation, String)>>>()?;

    let mut errors = Vec::new();

    for (line, expect, expr) in tests {
        if expect == Expectation::Comment {
            continue;
        }
        println!("line {}, running {expr:?}", line + 1);
        let j = run_j(&expr)?;
        let jr = scan_eval(&expr)
            .map(|w| format!("{}", w))
            .map_err(|e| format!("{:?}", e));
        let good = match expect {
            Expectation::Identical => match &jr {
                Ok(s) => s == &j,
                _ => false,
            },
            Expectation::Success => jr.is_ok(),
            Expectation::DomainError => todo!(),
            Expectation::LengthError => todo!(),
            Expectation::Failure => todo!(),
            Expectation::Comment => true,
        };
        if good {
            continue;
        }
        println!("Bad!");
        println!("j:");
        println!("{j}");
        println!("jr:");
        match &jr {
            Ok(s) => println!("{s}"),
            Err(e) => println!("failed: {e}"),
        }
        errors.push((line, expect, expr, j, jr));
    }

    Ok(())
}

impl FromStr for Expectation {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "i" | "identical" => Expectation::Identical,
            "s" | "success" => Expectation::Success,
            "d" | "domainerror" => Expectation::DomainError,
            "l" | "lengtherror" => Expectation::LengthError,
            "f" | "failure" => Expectation::Failure,
            "#" => Expectation::Comment,
            _ => bail!("unsupported prefix: {s:?}"),
        })
    }
}
