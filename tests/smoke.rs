use std::collections::HashMap;
use std::mem::transmute;

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use jr::test_impls::{scan_eval, Run, RunList};
use jr::{Arrayable, IntoJArray, JArray, Word};
use ndarray::{ArrayD, IxDyn};
use num::complex::Complex64;

#[test]
fn smoke() -> Result<()> {
    for run in RunList::open(include_str!("smoke.toml"))
        .expect("static asset")
        .runs
    {
        exec(&run).with_context(|| anyhow!("testing {:?}", run.expr))?;
    }
    Ok(())
}

fn exec(run: &Run) -> Result<()> {
    let them = decode(&run.encoded).context("decode")?;
    let us = scan_eval(&run.expr).context("running expression in smoke test")?;
    let Word::Noun(arr) = us else { bail!("unexpected non-array from eval: {us:?}") };
    let them_shape = run.parse_shape()?;

    if arr != them {
        bail!("incorrect data, we got {arr:?}, they expect {them:?}",);
    }

    if arr.shape() != them_shape {
        bail!(
            "incorrect shape, we got {:?}, they expect {:?}",
            arr.shape(),
            them_shape
        );
    }

    let our_type = match arr {
        JArray::IntArray(_) => "integer",
        JArray::BoolArray(_) => "boolean",
        JArray::CharArray(_) => "character",
        JArray::ExtIntArray(_) => "extended",
        JArray::RationalArray(_) => "rational",
        JArray::FloatArray(_) => "floating",
        JArray::ComplexArray(_) => "complex",
        JArray::BoxArray(_) => "box",
        JArray::LiteralArray(_) => "literal",
    };

    if our_type != run.datatype {
        bail!(
            "incorrect datatype, we got {:?}, they expect {:?}",
            our_type,
            run.datatype
        );
    }

    Ok(())
}

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
            Bool => elements.saturating_sub(1) / 8 + 1,
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
            Bool | Int | Float | Complex => false,
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

fn parse_hex(x: &str) -> Result<u64> {
    u64::from_str_radix(x, 16)
        .with_context(|| anyhow!("{x:?}"))
        .map(u64::swap_bytes)
}

// https://www.jsoftware.com/help/dictionary/dx003.htm#:~:text=Convert%20to%20Binary%20Representation
fn decode(j: &str) -> Result<JArray> {
    let lines = j.lines().map(parse_hex).collect::<Result<Vec<_>>>()?;
    println!("{lines:#?}");
    let mut lines = lines.into_iter().enumerate();
    let mut blocks = HashMap::new();

    while let Some((pos, marker)) = lines.next() {
        match marker {
            0xe3 => (),
            0xe0 | 0xe1 | 0xe2 => bail!("unsupported machine type (or parser error) at {pos}"),
            other => {
                bail!("invalid record header {other} at {pos}, probably a parser bug: {blocks:?}")
            }
        }
        blocks.insert(
            pos,
            parse_block(&mut lines).with_context(|| anyhow!("block at {pos}"))?,
        );
    }

    println!("{blocks:?}");

    let zero = blocks
        .get(&0)
        .ok_or_else(|| anyhow!("empty output isn't allowed"))?;

    reconstitute(zero, &blocks)
}

fn reconstitute(block: &Block, blocks: &HashMap<usize, Block>) -> Result<JArray> {
    Ok(if block.kind.stored_by_ref()? {
        let mut parts = Vec::new();
        for sub in &block.data {
            let off = usize::try_from(sub / 8)?;
            let sub = blocks
                .get(&off)
                .ok_or_else(|| anyhow!("invalid block references: {sub}"))?;
            parts.push(reconstitute(sub, blocks).context("pulling sub block")?);
        }
        parts.into_array()?.into_jarray()
    } else {
        match block.kind {
            JKind::Bool => JArray::BoolArray(ArrayD::from_shape_vec(
                IxDyn(&block.shape),
                block
                    .data
                    .iter()
                    .flat_map(|v| {
                        println!("{v}");
                        let v = u8::try_from(*v).unwrap_or(0);
                        [
                            (((v & 0b1000_0000) != 0) as u8),
                            (((v & 0b0100_0000) != 0) as u8),
                            (((v & 0b0010_0000) != 0) as u8),
                            (((v & 0b0001_0000) != 0) as u8),
                            (((v & 0b0000_1000) != 0) as u8),
                            (((v & 0b0000_0100) != 0) as u8),
                            (((v & 0b0000_0010) != 0) as u8),
                            (((v & 0b0000_0001) != 0) as u8),
                        ]
                    })
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
            other => bail!("{other:?}"),
        }
    })
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
