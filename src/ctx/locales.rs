use std::collections::HashMap;

use anyhow::{anyhow, Context, Result};
use itertools::Itertools;

use crate::{JError, Word};

#[derive(Debug)]
pub struct Eval {
    pub locales: Locales,
}

#[derive(Debug)]
pub struct Locales {
    pub inner: HashMap<String, Names>,
    pub search_paths: HashMap<String, Vec<String>>,
    pub current: String,

    // TODO: this should probably be as part of `nest`'s return value,
    // TODO: instead of an explicitly managed stack?
    pub anon: Vec<Names>,
}

#[derive(Debug, Default)]
pub struct Names(pub HashMap<String, Word>);

impl Eval {
    pub fn new() -> Eval {
        Eval {
            locales: Locales::new(),
        }
    }
}

impl Locales {
    pub fn new() -> Self {
        Self {
            inner: HashMap::with_capacity(8),
            search_paths: HashMap::with_capacity(8),
            current: "base".to_string(),
            anon: vec![Names::default()],
        }
    }

    pub fn assign_global(&mut self, n: impl ToString, v: Word) -> Result<()> {
        let n = n.to_string();
        let (n, ns) = parse_name(&n)?;
        let ns = ns.unwrap_or(self.current.as_str());

        self.inner
            .entry(ns.to_string())
            .or_insert_with(|| Default::default())
            .0
            .insert(n.to_string(), v);
        Ok(())
    }

    pub fn assign_local(&mut self, n: impl AsRef<str>, v: Word) -> Result<()> {
        let (n, ns) = parse_name(n.as_ref())?;
        if ns.is_some() {
            return Err(JError::NonceError).context("namespaced names");
        }

        self.anon
            .last_mut()
            .ok_or(JError::DomainError)
            .context("local assignment with no local scope")?
            .0
            .insert(n.to_string(), v);
        Ok(())
    }

    pub fn erase(&mut self, n: impl AsRef<str>) -> Result<()> {
        let (n, ns) = parse_name(n.as_ref())?;
        let ns = ns.unwrap_or(self.current.as_str());
        let Some(names) = self.inner.get_mut(ns) else { return Ok(()); };
        names.0.remove(n);
        Ok(())
    }

    pub fn lookup(&self, n: impl AsRef<str>) -> Result<Option<&Word>> {
        let (n, ns) = parse_name(n.as_ref())?;

        if let Some(ns) = ns {
            return Ok(self.inner.get(ns).and_then(|ns| ns.0.get(n)));
        }

        for local in self.anon.iter().rev() {
            if let Some(v) = local.0.get(n) {
                return Ok(Some(v));
            }
        }

        for ns in self
            .search_paths
            .get(self.current.as_str())
            .unwrap_or(&vec!["z".to_string(), self.current.to_string()])
            .iter()
            .rev()
        {
            if let Some(v) = self.inner.get(ns).and_then(|ns| ns.0.get(n)) {
                return Ok(Some(v));
            }
        }

        Ok(None)
    }
}

pub fn parse_name(name: &str) -> Result<(&str, Option<&str>)> {
    if name.contains("__") {
        return Err(JError::NonceError).context("resolved names");
    }
    if !name.ends_with('_') {
        return Ok((name, None));
    }
    let parts = name.split('_').collect_vec();
    Ok(match parts.len() {
        3 if parts[2].is_empty() => (parts[0], Some(parts[1])),
        _ => return Err(JError::IllFormedName).with_context(|| anyhow!("{name:?}")),
    })
}
