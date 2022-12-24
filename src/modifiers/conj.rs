use std::fmt;
use std::iter;
use std::ops::Deref;

use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use ndarray::prelude::*;

use crate::arrays::{map_result, Arrayable, BoxArray, JArrays};
use crate::cells::{apply_cells, monad_cells};
use crate::eval::{eval_lines, resolve_controls};
use crate::foreign::foreign;
use crate::verbs::{exec_dyad, exec_monad, Rank, VerbImpl};
use crate::{arr0d, generate_cells, Ctx, Num};
use crate::{flatten, reduce_arrays, HasEmpty, JArray, JError, Word};

pub type ConjunctionFn = fn(&mut Ctx, Option<&Word>, &Word, &Word, &Word) -> Result<Word>;

#[derive(Clone)]
pub struct SimpleConjunction {
    pub name: &'static str,
    pub f: ConjunctionFn,
    pub farcical: fn(&JArray, &JArray) -> Result<bool>,
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

pub fn not_farcical(_n: &JArray, _m: &JArray) -> Result<bool> {
    Ok(false)
}

pub fn c_not_implemented(
    _ctx: &mut Ctx,
    _x: Option<&Word>,
    _u: &Word,
    _v: &Word,
    _y: &Word,
) -> Result<Word> {
    Err(JError::NonceError).context("blanket conjunction implementation")
}

pub fn c_hatco(ctx: &mut Ctx, x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
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
                            t = u.exec(ctx, x, &t).map(Word::Noun)?;
                        }
                        Ok(t)
                    })
                    .collect::<Result<_, _>>()?,
            )?)
        }
        (Word::Verb(_, _), Word::Verb(_, _)) => bail!("power conjunction verb right argument"),
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

pub fn c_quote(ctx: &mut Ctx, x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
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
                (None, Word::Noun(y)) => exec_monad(
                    |y| {
                        u.exec(ctx, None, &Word::Noun(y.clone()))
                            .context("running monadic u inside re-rank")
                    },
                    ranks.0,
                    y,
                )
                .map(Word::Noun)
                .context("monadic rank drifting"),
                (Some(Word::Noun(x)), Word::Noun(y)) => exec_dyad(
                    |x, y| {
                        u.exec(ctx, Some(&Word::Noun(x.clone())), &Word::Noun(y.clone()))
                            .context("running dyadic u inside re-rank")
                    },
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
        (Word::Noun(u), Word::Noun(n)) => {
            let n = n
                .approx()
                .ok_or(JError::DomainError)
                .context("rank expects integer arguments")?;
            if n != arr0d(f32::INFINITY) {
                return Err(JError::NonceError).context("only infinite ranks");
            }
            Ok(Word::Noun(u.clone()))
        }
        _ => bail!("rank conjunction - other options? {x:?}, {u:?}, {v:?}, {y:?}"),
    }
}

pub fn c_agenda(ctx: &mut Ctx, x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
    use VerbImpl::*;
    use Word::*;

    let Noun(v) = v else { return Err(JError::NounResultWasRequired).context("agenda's index type"); };
    let v = v
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("agenda's index must be numeric")?
        .value_len()
        .ok_or(JError::DomainError)
        .context("agenda's index must be a len")?;

    // TODO: complete hack, only handling a tiny case
    if v != 0 && v != 1 {
        return Err(JError::NonceError).context("@. only implemented for 2-gerunds");
    }

    match u {
        Verb(_, DerivedVerb { l, r, m }) => match (l.deref(), r.deref(), m.deref()) {
            // TODO: complete hack, matching on the *name* of the verb
            (Verb(_, l), Verb(_, r), Conjunction(name, _)) if name == "`" => {
                if v == 0 {
                    return l.exec(ctx, x, y).map(Noun).context("agenda l");
                } else if v == 1 {
                    return r.exec(ctx, x, y).map(Noun).context("agenda r");
                } else {
                    unreachable!("checked above")
                }
            }
            _ => (),
        },
        _ => (),
    }
    Err(JError::NonceError).with_context(|| anyhow!("\nx: {x:?}\nu: {u:?}\nv: {v:?}\ny: {y:?}"))
}

// https://code.jsoftware.com/wiki/Vocabulary/at#/media/File:Funcomp.png
pub fn c_atop(ctx: &mut Ctx, x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
    match (u, v) {
        (Word::Verb(_, u), Word::Verb(_, v)) => {
            let r = v.partial_exec(ctx, x, y).context("right half of c_atop")?;
            let r = map_result(r, |a| u.exec(ctx, None, &Word::Noun(a.clone())))
                .context("left half of c_at")?;
            Ok(Word::Noun(
                flatten(&r).context("expanding result of c_atop")?,
            ))
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("expected to verb @ verb, not {u:?} @ {v:?}")),
    }
}

// https://code.jsoftware.com/wiki/Vocabulary/at#/media/File:Funcomp.png
pub fn c_at(ctx: &mut Ctx, x: Option<&Word>, u: &Word, v: &Word, y: &Word) -> Result<Word> {
    match (u, v) {
        (Word::Verb(_, u), Word::Verb(_, v)) => {
            let r = v.partial_exec(ctx, x, y).context("right half of c_at")?;
            let r = flatten(&r).context("expanding result of c_atop")?;
            u.exec(ctx, None, &Word::Noun(r))
                .context("left half of c_at")
                .map(Word::Noun)
        }
        _ => Err(JError::DomainError)
            .with_context(|| anyhow!("expected to verb @: verb, not {u:?} @: {v:?}")),
    }
}

pub fn c_cor_farcical(n: &JArray, m: &JArray) -> Result<bool> {
    Ok(match (n.single_math_num(), m.single_math_num()) {
        (Some(_n), Some(m)) => m == Num::Bool(0),
        _ => false,
    })
}

// TODO: this is typically called from partial_exec which has a panic
// TODO: attack about Nothing; this is a big lie
fn nothing_to_empty(w: Word) -> Word {
    match w {
        Word::Nothing => Word::Noun(JArray::empty()),
        other => other,
    }
}

pub fn c_cor(ctx: &mut Ctx, x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    use crate::arrays::JArray::*;
    match (n, m) {
        (Word::Noun(IntArray(n)), Word::Noun(CharArray(jcode))) => {
            if n == Array::from_elem(IxDyn(&[]), 4) {
                match x {
                    None => Err(JError::DomainError).with_context(|| anyhow!("dyad")),
                    Some(x) => {
                        // TODO: wrong, this should be a sub-context
                        let mut ctx = ctx.clone();
                        ctx.alias("x", x.clone());
                        ctx.alias("y", y.clone());
                        let mut words =
                            crate::scan(&jcode.clone().into_raw_vec().iter().collect::<String>())?;
                        if !resolve_controls(&mut words)? {
                            return Err(JError::SyntaxError).context("unable to resolve controls");
                        }
                        eval_lines(&words, &mut ctx)
                            .with_context(|| anyhow!("evaluating {:?}", jcode))
                            .map(nothing_to_empty)
                    }
                }
            } else if n == Array::from_elem(IxDyn(&[]), 3) {
                match x {
                    None => {
                        // TODO: wrong, this should be a sub-context
                        let mut ctx = ctx.clone();
                        ctx.alias("y", y.clone());
                        let mut words =
                            crate::scan(&jcode.clone().into_raw_vec().iter().collect::<String>())?;
                        if !resolve_controls(&mut words)? {
                            return Err(JError::SyntaxError).context("unable to resolve controls");
                        }
                        eval_lines(&words, &mut ctx)
                            .with_context(|| anyhow!("evaluating {:?}", jcode))
                            .map(nothing_to_empty)
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

pub fn c_assign_adverse(
    ctx: &mut Ctx,
    x: Option<&Word>,
    n: &Word,
    m: &Word,
    y: &Word,
) -> Result<Word> {
    match (n, m) {
        (Word::Verb(_, n), Word::Verb(_, m)) => n
            .exec(ctx, x, y)
            .or_else(|_| m.exec(ctx, x, y))
            .map(Word::Noun),
        _ => Err(JError::NonceError)
            .with_context(|| anyhow!("\nx: {x:?}\nn: {n:?}\nm: {m:?}\ny: {y:?}")),
    }
}

fn empty_box_array() -> BoxArray {
    ArrayD::from_shape_vec(IxDyn(&[0]), Vec::new()).expect("static shape")
}

pub fn c_cut(ctx: &mut Ctx, x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    use Word::*;
    let Noun(m) = m else { return Err(JError::DomainError).context("cut's mode arg"); };
    let m = m
        .single_math_num()
        .ok_or(JError::DomainError)
        .context("mathematical modes")?
        .value_i64()
        .ok_or(JError::DomainError)
        .context("integer modes")?;

    match (x, n, y) {
        (None, Verb(_, v), Noun(y)) => {
            let parts = y.outer_iter().collect_vec();
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
                    // copy-paste of below
                    let arg = if stack.is_empty() {
                        JArray::BoxArray(empty_box_array())
                    } else {
                        flatten(&stack).context("flattening intermediate")?
                    };
                    out.push(
                        Axis(0),
                        arr0d(
                            v.exec(ctx, None, &Noun(arg))
                                .context("evaluating intermediate")?,
                        )
                        .view(),
                    )?;
                    stack = empty_box_array();
                } else {
                    stack
                        .push(Axis(0), arr0d(part.to_owned()).view())
                        .context("push")?;
                }
            }

            flatten(&out).map(Noun)
        }
        (Some(Noun(JArray::BoolArray(x))), Verb(_, v), Noun(y)) if x.shape().len() == 1 => {
            let is_end = match m {
                2 | -2 => true,
                1 | -1 => false,
                _ => return Err(JError::DomainError).context("invalid mode for dyadic cut"),
            };

            let is_inclusive = match m {
                -2 | -1 => false,
                2 | 1 => true,
                _ => return Err(JError::DomainError).context("invalid mode for dyadic cut"),
            };

            if !is_end {
                return Err(JError::NonceError).context("only end mode supported right now");
            }
            let mut stack = empty_box_array();
            let mut out = empty_box_array();
            for (&x, part) in x.iter().zip(y.outer_iter()) {
                if is_inclusive || x == 0 {
                    stack.push(Axis(0), arr0d(part.into_owned()).view())?;
                }
                if x != 1 {
                    continue;
                }
                // copy-paste of above
                let arg = if stack.is_empty() {
                    JArray::BoxArray(empty_box_array())
                } else {
                    flatten(&stack).context("flattening intermediate")?
                };
                out.push(
                    Axis(0),
                    arr0d(
                        v.exec(ctx, None, &Noun(arg))
                            .context("evaluating intermediate")?,
                    )
                    .view(),
                )?;
                stack = empty_box_array();
            }
            flatten(&out).map(Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("{x:?} {n:?} {m:?} {y:?}")),
    }
}

pub fn c_foreign(ctx: &mut Ctx, x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
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
            foreign(ctx, n, m, x, y)
        }
        _ => Err(JError::NonceError).context("unsupported foreign syntax"),
    }
}

pub fn c_bondo(ctx: &mut Ctx, x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    match (x, n, m) {
        (None, Word::Verb(_, n), Word::Noun(m)) => n
            .exec(ctx, Some(&Word::Noun(m.clone())), y)
            .context("monad bondo VN")
            .map(Word::Noun),
        (None, Word::Noun(n), Word::Verb(_, m)) => m
            .exec(ctx, Some(&Word::Noun(n.clone())), y)
            .context("monad bondo NV")
            .map(Word::Noun),
        (None, n @ Word::Verb(_, _), m @ Word::Verb(_, _)) => c_atop(ctx, x, n, m, y),
        (Some(x), Word::Verb(_, u), Word::Verb(_, v)) => {
            let l = v.exec(ctx, None, x).context("left bondo NVV")?;
            let r = v.exec(ctx, None, y).context("right bondo NVV")?;
            u.exec(ctx, Some(&Word::Noun(l)), &Word::Noun(r))
                .map(Word::Noun)
                .context("central bondo NVV")
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("bondo x:{x:?} n:{n:?} m:{m:?}")),
    }
}

pub fn c_under(ctx: &mut Ctx, x: Option<&Word>, n: &Word, m: &Word, y: &Word) -> Result<Word> {
    match (x, n, m, y) {
        (None, Word::Verb(_, u), Word::Verb(_, v), Word::Noun(y)) => {
            let (cells, frame) = monad_cells(y, Rank::zero())?;
            let vi = v
                .obverse()
                .ok_or(JError::NonceError)
                .context("lacking obverse")?;
            let mut parts = Vec::new();
            for y in cells {
                let v = v.exec(ctx, None, &Word::Noun(y)).context("under dual v")?;
                let u = u.exec(ctx, None, &Word::Noun(v)).context("under dual u")?;
                let vi = vi
                    .exec(ctx, None, &Word::Noun(u))
                    .context("under dual vi")?;
                parts.push(vi);
            }
            flatten(&parts.into_array()?)?
                .to_shape(frame)
                .map(|cow| cow.to_owned())
                .map(Word::Noun)
        }
        (Some(Word::Noun(x)), Word::Verb(_, u), Word::Verb(_, v), Word::Noun(y)) => {
            let vi = v
                .obverse()
                .ok_or(JError::NonceError)
                .context("lacking obverse")?;
            let vr = v
                .dyad_rank()
                .ok_or(JError::NonceError)
                .context("missing rank for dyad")?;
            let (frame, cells) = generate_cells(x.clone(), y.clone(), vr)?;
            let parts = apply_cells(
                &cells,
                |x, y| {
                    let l = v
                        .exec(ctx, None, &Word::Noun(x.clone()))
                        .context("under dual l")?;
                    let r = v
                        .exec(ctx, None, &Word::Noun(y.clone()))
                        .context("under dual r")?;
                    let u = u
                        .exec(ctx, Some(&Word::Noun(l)), &Word::Noun(r))
                        .context("under dual u")?;
                    vi.exec(ctx, None, &Word::Noun(u)).context("under dual vi")
                },
                vr,
            )?;
            flatten(&parts.into_array()?)?
                .to_shape(frame)
                .map(|cow| cow.to_owned())
                .map(Word::Noun)
        }
        _ => Err(JError::NonceError).with_context(|| anyhow!("under dual x:{x:?} n:{n:?} m:{m:?}")),
    }
}
