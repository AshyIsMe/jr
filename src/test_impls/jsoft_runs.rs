use crate::test_impls::jsoft_binary;
use crate::JArray;
use anyhow::{anyhow, Result};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
}

fn capture(expr: impl AsRef<str>) -> Result<Run> {
    let expr = expr.as_ref();
    Ok(Run {
        expr: expr.to_string(),
        output: run_j(expr)?,
        encoded: run_j(format!("3!:3 ] {expr}"))?
            .lines()
            // most of the values are like "e800000000000000". toml lists are very verbose.
            // strip the trailing zeros and just join them with a comma
            .map(|s| s.trim_end_matches('0'))
            .join(","),
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

    pub fn into_lookup(self) -> Result<HashMap<String, (JArray, String)>> {
        self.runs
            .into_iter()
            .map(|run| {
                let arr = run.parse_encoded()?;
                Ok((run.expr, (arr, run.output)))
            })
            .collect()
    }
}

pub trait Lookup {
    fn get_cached(&self, val: &str) -> Result<&(JArray, String)>;
}

impl Lookup for HashMap<String, (JArray, String)> {
    fn get_cached(&self, expr: &str) -> Result<&(JArray, String)> {
        self.get(expr)
            .ok_or_else(|| anyhow!("no cached result for {expr:?}, re-generate the toml?"))
    }
}

impl Run {
    pub fn parse_encoded(&self) -> Result<JArray> {
        jsoft_binary::decode(
            &self
                .encoded
                .split(',')
                .map(|s| jsoft_binary::parse_hex(&format!("{s:0<16}")))
                .collect::<Result<Vec<u64>>>()?,
        )
    }
}
