use crate::{arr0d, JArray, Word};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Ctx {
    names: HashMap<String, Word>,
}

impl Ctx {
    pub fn empty() -> Self {
        let mut ctx = Ctx {
            names: Default::default(),
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
}
