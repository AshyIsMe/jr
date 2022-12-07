use std::collections::{HashMap, VecDeque};

use anyhow::{Context, Result};

use crate::{arr0d, JArray, JError, Word};

#[derive(Clone, Debug)]
pub struct Ctx {
    names: HashMap<String, Word>,
    suspension: Vec<Suspense>,
}

#[derive(Clone, Debug)]
pub struct Suspense {
    pub queue: VecDeque<Word>,
    pub stack: VecDeque<Word>,
    done: bool,
    pub data: String,
}

impl Ctx {
    pub fn empty() -> Self {
        let mut ctx = Ctx {
            names: Default::default(),
            suspension: Vec::new(),
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
        !self.suspension.is_empty()
    }

    pub fn suspend(&mut self, queue: VecDeque<Word>, stack: VecDeque<Word>) -> Result<()> {
        self.suspension.push(Suspense {
            queue,
            stack,
            done: false,
            data: String::new(),
        });
        Ok(())
    }

    pub fn pop_suspension(&mut self) -> Result<Suspense> {
        self.suspension
            .pop()
            .ok_or(JError::DomainError)
            .context("no suspensions")
    }

    pub fn input_wanted(&self) -> bool {
        self.suspension.last().map(|s| !s.done).unwrap_or_default()
    }

    pub fn input_push(&mut self, data: &str) -> Result<()> {
        let suspension = self
            .suspension
            .last_mut()
            .ok_or(JError::DomainError)
            .context("not suspended")?;
        if suspension.done {
            return Err(JError::DomainError).context("data complete");
        }

        suspension.data.push_str(data);
        suspension.data.push('\n');
        Ok(())
    }

    pub fn input_done(&mut self) -> Result<()> {
        self.suspension
            .last_mut()
            .ok_or(JError::DomainError)
            .context("not suspended")?
            .done = true;
        Ok(())
    }
}
