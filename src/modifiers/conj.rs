use std::iter;
use std::{fmt, fs};

use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use ndarray::prelude::*;

use crate::arrays::{map_result, Arrayable, BoxArray, JArrays};
use crate::verbs::{exec_dyad, exec_monad, Rank};
use crate::{arr0d, eval, Ctx, IntoJArray};
use crate::{flatten, reduce_arrays, HasEmpty, JArray, JError, Word};

pub type ConjunctionFn = fn(Option<&Word>, &Word, &Word, &Word) -> Result<Word>;

#[derive(Clone)]
pub struct SimpleConjunction {
    pub name: &'static str,
    pub f: ConjunctionFn,
}

impl PartialEq for SimpleConjunction {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(other.name)
    }
}

impl fmt::Debug for SimpleConjunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SimpleAdverb({:?})", self.name)
    }
}

pub fn c_not_implemented(_x: Option<&Word>, _u: &Word, _v: &Word, _y: &Word) -> Result<Word> {
    Err(JError::NonceError).context("blanket conjunction implementation")
}

pub fn c_hatco(x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
    // TODO: inverse, converge and Dynamic Power (verb argument)
    // https://code.jsoftware.com/wiki/Vocabulary/hatco
    match (u, v) {
        (Word::Verb(_, u), Word::Noun(ja)) => {
            let n = ja.to_i64().ok_or(JError::DomainError)?;
            Ok(collect_nouns(
                n.iter()
                    .map(|i| -> Result<_> {
                        let mut t = y.clone();
                        for _ in 0..*i {
                            t = u.exec(x, &t).map(Word::Noun)?;
                        }
                        Ok(t)
                    })
                    .collect::<Result<_, _>>()?,
            )?)
        }
        (Word::Verb(_, _), Word::Verb(_, _)) => todo!("power conjunction verb right argument"),
        _ => Err(JError::DomainError).with_context(|| anyhow!("{u:?} {v:?}")),
    }
}

pub fn collect_nouns(n: Vec<Word>) -> Result<Word> {
    // Collect a Vec<Word::Noun> into a single Word::Noun.
    // Must all be the same JArray type. ie. IntArray, etc

    let arr = n
        .iter()
        .map(|w| match w {
            Word::Noun(arr) => Ok(arr),
            _ => Err(JError::DomainError).with_context(|| anyhow!("{w:?}")),
        })
        .collect::<Result<Vec<_>>>()?;

    let arrs = JArrays::from_homo(&arr)?;

    Ok(Word::Noun(reduce_arrays!(arrs, collect)))
}

fn collect<T: Clone + HasEmpty>(arr: &[ArrayViewD<T>]) -> Result<ArrayD<T>> {
    // TODO: this special cases the atom/scalar case, as the reshape algorithm mangles it
    if arr.len() == 1 && arr[0].shape().is_empty() {
        return Ok(arr[0].to_owned());
    }
    let cell_shape = arr
        .iter()
        .map(|arr| arr.shape())
        .max()
        .ok_or(JError::DomainError)?;
    let empty_shape = iter::once(0)
        .chain(cell_shape.iter().copied())
        .collect::<Vec<_>>();

    let mut result = Array::from_elem(empty_shape, T::empty());
    for item in arr {
        result
            .push(Axis(0), item.view())
            .map_err(JError::ShapeError)?;
    }
    Ok(result)
}

pub fn c_quote(x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
    match (u, v) {
        (Word::Verb(_, u), Word::Noun(n)) => {
            let n = n
                .approx()
                .ok_or(JError::DomainError)
                .context("rank expects integer arguments")?;

            let ranks = match (n.shape().len(), n.len()) {
                (0, 1) => {
                    let only = n.iter().next().copied().expect("checked the length");
                    [only, only, only]
                }
                (1, 1) => [n[0], n[0], n[0]],
                (1, 2) => [n[1], n[0], n[1]],
                (1, 3) => [n[0], n[1], n[2]],
                _ => {
                    return Err(JError::LengthError).with_context(|| {
                        anyhow!("rank operator requires a list of 1-3 elements, not: {n:?}")
                    })
                }
            };

            let ranks = (
                Rank::from_approx(ranks[0])?,
                Rank::from_approx(ranks[1])?,
                Rank::from_approx(ranks[2])?,
            );

            match (x, y) {
                (None, Word::Noun(y)) => {
                    exec_monad(|y| u.exec(None, &Word::Noun(y.clone())), ranks.0, y)
                        .map(Word::Noun)
                        .context("monadic rank drifting")
                }
                (Some(Word::Noun(x)), Word::Noun(y)) => exec_dyad(
                    |x, y| u.exec(Some(&Word::Noun(x.clone())), &Word::Noun(y.clone())),
                    (ranks.1, ranks.2),
                    x,
                    y,
                )
                .map(Word::Noun)
                .context("dyadic rank drifting"),
                _ => Err(JError::NonceError)
                    .with_context(|| anyhow!("can't rank non-nouns, {x:?} {y:?}")),
            }
        }
        _ => bail!("rank conjunction - other options? {x:?}, {u:?}, {v:?}, {y:?}"),
    }
}

pub fn c_at(x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
    match (u, v) {
        (Word::Verb(_, u), Word::Verb(_, v)) => {
            let r = v.partial_exec(x, y).context("right half of c_at")?;
            let r = map_result(r, |a| u.exec(None, &Word::Noun(a.clone())))
                .context("left half of c_at")?;
            Ok(Word::Noun(flatten(&r).context("expanding result of c_at")?))
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("expected to verb @ verb, not {u:?} @ {v:?}")),
    }
}

pub fn c_cor(x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    use crate::arrays::JArray::*;
    match (n, m) {
        (Word::Noun(IntArray(n)), Word::Noun(CharArray(jcode))) => {
            if n == Array::from_elem(IxDyn(&[]), 4) {
                match x {
                    None => Err(JError::DomainError).with_context(|| anyhow!("dyad")),
                    Some(x) => {
                        let mut ctx = Ctx::empty();
                        ctx.alias("x", x.clone());
                        ctx.alias("y", y.clone());
                        eval(
                            crate::scan(&jcode.clone().into_raw_vec().iter().collect::<String>())?,
                            &mut ctx,
                        )
                        .with_context(|| anyhow!("evaluating {:?}", jcode))
                    }
                }
            } else if n == Array::from_elem(IxDyn(&[]), 3) {
                match x {
                    None => {
                        let mut ctx = Ctx::empty();
                        ctx.alias("y", y.clone());
                        eval(
                            crate::scan(&jcode.clone().into_raw_vec().iter().collect::<String>())?,
                            &mut ctx,
                        )
                        .with_context(|| anyhow!("evaluating {:?}", jcode))
                    }
                    _ => Err(JError::DomainError).with_context(|| anyhow!("monad")),
                }
            } else {
                Err(JError::DomainError).with_context(|| anyhow!("{n:?} {m:?}"))
            }
        }
        _ => Err(JError::DomainError).with_context(|| anyhow!("{n:?} {m:?}")),
    }
}

fn empty_box_array() -> BoxArray {
    ArrayD::from_shape_vec(IxDyn(&[0]), Vec::new()).expect("static shape")
}

pub fn c_cut(x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    use Word::*;
    match (x, n, m, y) {
        (None, Verb(_, v), Noun(m), Noun(y)) => {
            let m = m
                .single_math_num()
                .ok_or(JError::DomainError)
                .context("mathematical modes")?
                .value_i64()
                .ok_or(JError::DomainError)
                .context("integer modes")?;
            let parts = y.outer_iter();
            ensure!(!parts.is_empty());
            match m {
                -2 => (),
                _ => return Err(JError::NonceError).context("only mode -2 is supported"),
            }

            let key = &parts[parts.len() - 1];
            let mut stack = empty_box_array();
            let mut out = empty_box_array();
            for part in &parts {
                if part == key {
                    let arg = if stack.is_empty() {
                        JArray::BoxArray(empty_box_array())
                    } else {
                        flatten(&stack).context("flattening intermediate")?
                    };
                    out.push(
                        Axis(0),
                        arr0d(
                            v.exec(None, &Noun(arg))
                                .context("evaluating intermediate")?,
                        )
                        .view(),
                    )?;
                    stack = empty_box_array();
                } else {
                    stack
                        .push(Axis(0), arr0d(JArray::from(part.clone())).view())
                        .context("push")?;
                }
            }

            flatten(&out).map(Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {n:?} {m:?} {y:?}")),
    }
}

pub fn c_foreign(_x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    match (n, m) {
        (Word::Noun(n), Word::Noun(m)) => {
            let n = n
                .single_math_num()
                .and_then(|n| n.value_len())
                .ok_or(JError::DomainError)
                .context("left foreign takes numerics")?;
            let m = m
                .single_math_num()
                .and_then(|m| m.value_len())
                .ok_or(JError::DomainError)
                .context("right foreign takes numerics")?;
            match (m, n) {
                (1, 1) => f_read_file(y).context("reading file"),
                _ => Err(JError::NonceError)
                    .with_context(|| anyhow!("unsupported foreign: {m}!:{n}")),
            }
        }
        _ => Err(JError::NonceError).context("unsupported foreign syntax"),
    }
}

fn f_read_file(y: &Word) -> Result<Word> {
    match y {
        Word::Noun(JArray::BoxArray(arr)) if arr.len() == 1 => {
            let arr = arr
                .iter()
                .next()
                .ok_or(JError::DomainError)
                .context("empty box?")?;
            let arr = arr
                .when_char()
                .ok_or(JError::NonceError)
                .context("can't read boxed non-paths")?;

            if arr.shape().len() > 1 {
                return Err(JError::NonceError).context("multi-dimensional path");
            }

            let path: String = arr.iter().copied().collect();
            match fs::read_to_string(&path) {
                Ok(s) => Ok(s.chars().collect_vec().into_array()?.into_noun()),
                Err(e) => Err(JError::FileNameError)
                    .context(e)
                    .with_context(|| anyhow!("reading {path:?}")),
            }
        }
        _ => Err(JError::NonceError)
            .context("can't read non-paths (hint: pointless box required 1!:1 <'a')"),
    }
}
