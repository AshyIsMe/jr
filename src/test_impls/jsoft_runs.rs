use crate::test_impls::jsoft_binary;
use crate::test_impls::jsoft_binary::parse_hex;
use crate::JArray;
use anyhow::{anyhow, Context, Result};
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
    let debug_expr = insert_debug(expr)?;
    let output = run_j(expr).with_context(|| anyhow!("{expr}"))?;
    let grab_binary = run_j(&debug_expr).context(debug_expr)?;

    let help = concat!(
        "Normally this means the program failed to end with a single expression, ",
        "returns an error, or has some unexpected output during execution"
    );

    let encoded = grab_binary
        .lines()
        // most of the values are like "e800000000000000". toml lists are very verbose.
        // strip the trailing zeros and just join them with a comma
        .map(|x| {
            if parse_hex(x).is_ok() {
                Ok(x)
            } else {
                Err(anyhow!("invalid value in j output, {x:?}"))
            }
        })
        .collect::<Result<Vec<_>>>()
        .context(help)
        .with_context(|| anyhow!("J said:\n{output}"))?
        .into_iter()
        .map(|s| s.trim_end_matches('0'))
        .join(",")
        // e3 is the object marker; this doesn't matter, but makes diffs a little nicer
        .replace(",e3", ",\ne3");
    let run = Run {
        expr: expr.to_string(),
        encoded,
        output: output.to_string(),
    };

    let _ = run
        .parse_encoded()
        .context("ensuring that the generated encoded output is parseable")
        .context(help)
        .with_context(|| anyhow!("J said:\n{output}"))
        .with_context(|| anyhow!("J (debug) said:\n{grab_binary}"))?;
    Ok(run)
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
                .replace('\n', "")
                .split(',')
                .map(|s| jsoft_binary::parse_hex(&format!("{s:0<16}")))
                .collect::<Result<Vec<u64>>>()?,
        )
    }
}

// replaces the last \n in a trimmed string with `\n3!:3 ]`, which generates the hex stuff
fn insert_debug(expr: impl AsRef<str>) -> Result<String> {
    let mut content = expr
        .as_ref()
        .trim()
        .lines()
        .map(ToString::to_string)
        .collect_vec();
    let last_line = content
        .last_mut()
        .ok_or(anyhow!("non-empty files please"))?;
    *last_line = format!("3!:3 ] {last_line}");
    Ok(content.join("\n"))
}
