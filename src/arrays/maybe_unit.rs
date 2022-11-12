pub enum MaybeUnit<T> {
    Atom([T; 1]),
    List(Vec<T>),
}

impl<T> MaybeUnit<T> {
    pub fn from_parse_rules(list: Vec<T>) -> Self {
        if list.len() == 1 {
            Self::Atom([list.into_iter().next().expect("checked len")])
        } else {
            Self::List(list)
        }
    }

    pub fn from_parse_iter(it: impl IntoIterator<Item = T>) -> Self {
        Self::from_parse_rules(it.into_iter().collect())
    }

    pub fn as_list(&self) -> &[T] {
        match self {
            Self::Atom(x) => x.as_slice(),
            Self::List(x) => &x,
        }
    }

    pub fn into_list(self) -> Vec<T> {
        match self {
            Self::Atom(x) => vec![x.into_iter().next().expect("fixed size array")],
            Self::List(x) => x,
        }
    }
}
