use std::collections::HashMap;

use anyhow::{Context, Result};

use crate::{JError, Word};

#[derive(Debug)]
pub struct Eval {
    pub locales: Locales,
}

#[derive(Debug)]
pub struct Locales {
    inner: HashMap<String, Names>,
    search_path: Vec<String>,
    pub anon: Vec<Names>,
}

#[derive(Debug, Default)]
pub struct Names(HashMap<String, Word>);

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
            search_path: vec!["z".to_string(), "base".to_string()],
            anon: Vec::new(),
        }
    }

    pub fn assign_global(&mut self, n: impl ToString, v: Word) -> Result<()> {
        let n = n.to_string();
        if n.contains('_') {
            return Err(JError::NonceError).context("namespaced names");
        }
        self.inner
            .entry(
                self.search_path
                    .last()
                    .expect("non-empty search path")
                    .to_string(),
            )
            .or_insert_with(|| Default::default())
            .0
            .insert(n.to_string(), v);
        Ok(())
    }

    pub fn assign_local(&mut self, n: impl ToString, v: Word) -> Result<()> {
        let n = n.to_string();
        if n.contains('_') {
            return Err(JError::NonceError).context("namespaced names");
        }

        self.anon
            .last_mut()
            .ok_or(JError::DomainError)
            .context("local assignment with no local scope")?
            .0
            .insert(n, v);
        Ok(())
    }

    pub fn lookup(&self, n: impl AsRef<str>) -> Result<Option<&Word>> {
        let n = n.as_ref();
        for local in self.anon.iter().rev() {
            if let Some(v) = local.0.get(n) {
                return Ok(Some(v));
            }
        }

        for ns in self.search_path.iter().rev() {
            if let Some(v) = self.inner.get(ns).and_then(|ns| ns.0.get(n)) {
                return Ok(Some(v));
            }
        }

        Ok(None)
    }
}
