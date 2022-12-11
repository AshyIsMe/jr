use std::collections::HashMap;

use anyhow::{ensure, Context, Result};

use crate::eval::Qs;
use crate::{arr0d, JArray, JError, Word};

#[derive(Clone, Debug)]
pub struct Ctx {
    names: HashMap<String, Word>,
    suspension: Option<Suspense>,
    pub other_input_buffer: String,
}

#[derive(Clone, Debug)]
pub struct Suspense {
    pub qs: Qs,
    pub data: String,
}

impl Ctx {
    pub fn empty() -> Self {
        let mut ctx = Ctx {
            names: Default::default(),
            suspension: None,
            other_input_buffer: String::new(),
        };
        ctx.alias("LF", Word::Noun(JArray::CharArray(arr0d('\n'))));
        ctx
    }

    pub fn alias(&mut self, n: impl ToString, v: Word) {
        self.names.insert(n.to_string(), v);
    }

    pub fn resolve(&self, n: impl AsRef<str>) -> Option<&Word> {
        let n = n.as_ref();
        self.names.get(n)
    }

    pub fn is_suspended(&self) -> bool {
        self.suspension.is_some()
    }

    pub fn suspend(&mut self, qs: Qs) -> Result<()> {
        ensure!(self.suspension.is_none());
        self.suspension = Some(Suspense {
            qs,
            data: String::new(),
        });
        Ok(())
    }

    pub fn take_suspension(&mut self) -> Option<Suspense> {
        self.suspension.take()
    }

    pub fn input_push(&mut self, data: &str) -> Result<()> {
        let suspension = self
            .suspension
            .as_mut()
            .ok_or(JError::DomainError)
            .context("not suspended")?;

        suspension.data.push_str(data);
        suspension.data.push('\n');
        Ok(())
    }
}
