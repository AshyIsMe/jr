use std::collections::HashMap;
use std::mem::transmute;

use crate::JArray;
use anyhow::{anyhow, bail, ensure, Context, Result};
use itertools::Itertools;
use ndarray::{ArrayD, IxDyn};
use num::complex::Complex64;
use num::{BigInt, BigRational};

// https://www.jsoftware.com/help/dictionary/dx003.htm
#[derive(Copy, Clone, Debug)]
enum JKind {
    Bool,
    Lit,
    Int,
    Float,
    Complex,
    Boxed,
    ExtInt,
    Rational,
    Symbol,
    Lit2,
    Lit4,
}

impl TryFrom<u64> for JKind {
    type Error = anyhow::Error;
    fn try_from(value: u64) -> Result<Self> {
        use JKind::*;
        Ok(match value {
            1 => Bool,
            2 => Lit,
            4 => Int,
            8 => Float,
            16 => Complex,
            32 => Boxed,
            64 => ExtInt,
            128 => Rational,
            1024 | 2048 | 4096 | 8192 | 16384 | 32768 => bail!("unsupported: sparse"),
            65536 => Symbol,
            131072 => Lit2,
            262144 => Lit4,
            other => bail!("unknown: {other}"),
        })
    }
}

impl JKind {
    fn element_slots(&self, elements: usize) -> Result<usize> {
        use JKind::*;
        Ok(match self {
            Bool => elements / 8 + 1,
            Lit => elements / 8 + 1,
            Complex => elements * 2,
            Int | Float => elements * 1,
            Boxed => elements * 1,
            ExtInt => elements * 1,
            Rational => elements * 2,
            other => bail!("unknown size: {other:?}"),
        })
    }

    fn stored_by_ref(&self) -> Result<bool> {
        use JKind::*;
        Ok(match self {
            Bool | Int | Float | Complex | Lit => false,
            Boxed | ExtInt | Rational => true,
            other => bail!("unknown ref: {other:?}"),
        })
    }
}

#[derive(Debug)]
struct Block {
    kind: JKind,
    shape: Vec<usize>,
    elements: usize,
    data: Vec<u64>,
}

pub fn parse_hex(x: &str) -> Result<u64> {
    u64::from_str_radix(x, 16)
        .with_context(|| anyhow!("{x:?}"))
        .map(u64::swap_bytes)
}

// https://www.jsoftware.com/help/dictionary/dx003.htm#:~:text=Convert%20to%20Binary%20Representation
pub fn decode(lines: &[u64]) -> Result<JArray> {
    let mut lines = lines.into_iter().copied().enumerate();
    let mut blocks = HashMap::new();

    while let Some((pos, marker)) = lines.next() {
        match marker {
            0xe3 => (),
            0xe0 | 0xe1 | 0xe2 => bail!("unsupported machine type (or parser error) at {pos}"),
            other if pos == 0 => {
                bail!("invalid record header {other} at the start; not a valid document")
            }
            other => {
                bail!("invalid record header {other} at {pos}, successfully parsed so far: {blocks:?}")
            }
        }
        blocks.insert(
            pos,
            parse_block(&mut lines).with_context(|| anyhow!("block at {pos}"))?,
        );
    }

    // println!("{blocks:?}");

    let zero = blocks
        .get(&0)
        .ok_or_else(|| anyhow!("empty output isn't allowed"))?;

    reconstitute(zero, 0, &blocks)
}

fn reconstitute(block: &Block, our_off: usize, blocks: &HashMap<usize, Block>) -> Result<JArray> {
    if !(block.kind.stored_by_ref()?) {
        return Ok(match block.kind {
            JKind::Bool => JArray::BoolArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                block
                    .data
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .take(block.elements)
                    .collect(),
            )?),
            JKind::Lit => JArray::CharArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                block
                    .data
                    .iter()
                    .flat_map(|v| v.to_le_bytes())
                    .map(|c| char::from(c))
                    .take(block.elements)
                    .collect(),
            )?),
            JKind::Int => JArray::IntArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                block
                    .data
                    .iter()
                    .map(|&v| unsafe { transmute(v) })
                    .collect(),
            )?),
            JKind::Float => JArray::FloatArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                block
                    .data
                    .iter()
                    .map(|&v| unsafe { transmute(v) })
                    .collect(),
            )?),
            JKind::Complex => JArray::ComplexArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                block
                    .data
                    .iter()
                    .tuples()
                    .map(|(&r, &i)| unsafe { Complex64::new(transmute(r), transmute(i)) })
                    .collect(),
            )?),
            other => bail!("unsupported plain type: {other:?}"),
        });
    }
    let mut parts = Vec::new();
    for sub in &block.data {
        let off = usize::try_from(sub / 8)? + our_off;
        let sub = blocks
            .get(&off)
            .ok_or_else(|| anyhow!("invalid block references: {sub}"))?;
        parts.push(reconstitute(sub, off, blocks).context("pulling sub block")?);
    }
    Ok(match block.kind {
        JKind::Boxed => JArray::BoxArray(ArrayD::from_shape_vec(IxDyn(&block.shape), parts)?),
        JKind::ExtInt => {
            let parts = parts
                .into_iter()
                .map(|p| match p {
                    JArray::IntArray(x) if x.shape().len() == 1 => Ok(x),
                    other => bail!("unexpected non-linear part in extint: {other:?}"),
                })
                .collect::<Result<Vec<_>>>()?;
            JArray::ExtIntArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                parts.into_iter().map(big).collect::<Result<Vec<_>>>()?,
            )?)
        }
        JKind::Rational => {
            let parts = parts
                .into_iter()
                .map(|p| match p {
                    JArray::IntArray(x) if x.shape().len() == 1 => Ok(x),
                    other => bail!("unexpected non-linear part in extint: {other:?}"),
                })
                .collect::<Result<Vec<_>>>()?;
            JArray::RationalArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                parts
                    .into_iter()
                    .tuples()
                    .map(|(up, down)| Ok(BigRational::new(big(up)?, big(down)?)))
                    .collect::<Result<Vec<_>>>()?,
            )?)
        }
        other => bail!("unsupported ref type: {other:?}"),
    })
}

fn big(words: ArrayD<i64>) -> Result<BigInt> {
    ensure!(words.shape().len() == 1);
    if words.len() == 1 {
        Ok(words[0].into())
    } else {
        bail!("don't know how to unpack {words:?} into bigint")
    }
}

fn parse_block(lines: &mut impl Iterator<Item = (usize, u64)>) -> Result<Block> {
    let mut lines = lines.map(|(_, x)| x);
    let kind = JKind::try_from(lines.next().ok_or_else(|| anyhow!("expecting kind"))?)?;
    let elements = lines.next().ok_or_else(|| anyhow!("expecting elements"))?;
    let elements = usize::try_from(elements).context("elements didn't fit in a usize")?;
    let rank = lines.next().ok_or_else(|| anyhow!("expecting rank"))?;
    let mut shape = Vec::new();
    for _ in 0..rank {
        let part = lines
            .next()
            .ok_or_else(|| anyhow!("expecting shape part"))?;
        shape.push(usize::try_from(part)?);
    }

    Ok(Block {
        kind,
        shape,
        elements,
        data: lines.take(kind.element_slots(elements)?).collect(),
    })
}

#[test]
fn floats() {
    let i = parse_hex("000000000000f8bf").unwrap();
    assert_eq!(-1.5, unsafe { std::mem::transmute(i) });

    let i = parse_hex("333333333333c33f").unwrap();
    assert_eq!(i, 4594572339843380019);
    assert_eq!(0.15, unsafe { std::mem::transmute(i) });
}
