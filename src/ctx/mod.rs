use crate::Word;
use std::collections::HashMap;

#[derive(Clone, Default, Debug)]
pub struct Ctx {
    names: HashMap<String, Word>,
}

impl Ctx {
    pub fn empty() -> Self {
        Ctx {
            names: Default::default(),
        }
    }

    pub fn alias(&mut self, n: impl ToString, v: Word) {
        self.names.insert(n.to_string(), v);
    }

    pub fn resolve(&self, n: impl AsRef<str>) -> Option<&Word> {
        let n = n.as_ref();
        self.names.get(n)
    }
}
