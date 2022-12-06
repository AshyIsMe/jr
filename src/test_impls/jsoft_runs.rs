use crate::test_impls::jsoft_binary;
use crate::JArray;
use anyhow::Result;
use itertools::Itertools;
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
    pub encoded: Vec<i64>,
}

fn capture(expr: impl AsRef<str>) -> Result<Run> {
    let expr = expr.as_ref();
    Ok(Run {
        expr: expr.to_string(),
        output: run_j(expr)?,
        encoded: run_j(format!("3!:3 ] {expr}"))?
            .lines()
            .map(jsoft_binary::parse_hex)
            .map_ok(|i| i as i64)
            .collect::<Result<_>>()?,
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
    pub fn parse_encoded(&self) -> Result<JArray> {
        jsoft_binary::decode(
            &self
                .encoded
                .iter()
                .copied()
                .map(|i: i64| i as u64)
                .collect_vec(),
        )
    }
}
