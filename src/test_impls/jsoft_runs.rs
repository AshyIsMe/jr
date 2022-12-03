use std::str::FromStr;

use crate::number::Num;
use crate::scan::scan_num_token;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::run_j;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunList {
    pub runs: Vec<Run>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub expr: String,
    pub output: String,
    pub encoded: String,
    pub datatype: String,
    pub shape: String,
}

fn capture(expr: impl AsRef<str>) -> Result<Run> {
    let expr = expr.as_ref();
    Ok(Run {
        expr: expr.to_string(),
        output: run_j(expr)?,
        encoded: run_j(format!("3!:3 ] {expr}"))?,
        datatype: run_j(format!("datatype {expr}"))?,
        shape: run_j(format!("$ {expr}"))?,
    })
}

impl RunList {
    pub fn empty() -> Self {
        RunList { runs: Vec::new() }
    }

    pub fn open(content: impl AsRef<str>) -> Result<Self> {
        Ok(toml::from_str(content.as_ref())?)
    }

    pub fn save(&mut self) -> Result<String> {
        self.sort();
        Ok(toml::to_string_pretty(self)?)
    }

    fn sort(&mut self) {
        self.runs.sort_by_key(|r| r.expr.to_string())
    }

    pub fn add(&mut self, expr: impl AsRef<str>) -> Result<Run> {
        let expr = expr.as_ref();
        if let Some(run) = self.runs.iter().find(|r| r.expr == expr) {
            return Ok(run.clone());
        }
        let run = capture(expr)?;
        self.runs.push(run.clone());
        Ok(run)
    }
}

impl Run {
    pub fn parse_shape(&self) -> Result<Vec<usize>> {
        self.shape
            .split_whitespace()
            .map(|x| Ok(usize::from_str(x)?))
            .collect()
    }

    pub fn parse_data(&self) -> Result<Vec<Num>> {
        self.output.split_whitespace().map(scan_num_token).collect()
    }
}
