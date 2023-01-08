mod locales;

use anyhow::{ensure, Context, Result};
use std::ops::{Deref, DerefMut};

use crate::eval::Qs;
use crate::JError;

// :)
pub use locales::Eval;
// :(
pub use locales::Names;

#[derive(Debug)]
pub struct Ctx {
    eval: Eval,
    pub input_buffers: Option<InputBuffers>,
}

#[derive(Debug)]
pub struct InputBuffers {
    suspension: Option<Suspense>,
    pub other_input_buffer: String,
}

#[derive(Clone, Debug)]
pub struct Suspense {
    pub qs: Qs,
    pub data: String,
}

impl Ctx {
    pub fn root() -> Self {
        Ctx {
            eval: Eval::new(),
            input_buffers: Some(InputBuffers {
                suspension: None,
                other_input_buffer: String::new(),
            }),
        }
    }

    pub fn eval(&self) -> &Eval {
        &self.eval
    }

    pub fn eval_mut(&mut self) -> &mut Eval {
        &mut self.eval
    }

    pub fn nest(&mut self) -> Guard {
        self.eval.locales.anon.push(Names::default());
        Guard { inner: self }
    }
}

pub struct Guard<'i> {
    inner: &'i mut Ctx,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        self.inner.eval.locales.anon.pop().expect("I pushed it");
    }
}

impl Deref for Guard<'_> {
    type Target = Ctx;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl DerefMut for Guard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl InputBuffers {
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
