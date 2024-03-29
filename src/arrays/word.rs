use num::complex::Complex64;
use num::{BigInt, BigRational};
use std::convert::From;
use std::fmt;

use anyhow::{anyhow, Context, Result};
use ndarray::prelude::*;

use crate::eval::quote_arr;
use crate::modifiers::ModifierImpl;
use crate::verbs::{stringify, VerbImpl};
use crate::{primitive_verbs, JArray, JError};

// A Word is a part of speech.
#[derive(Clone, PartialEq, Debug)]
pub enum Word {
    LP,
    RP,
    StartOfLine, // used as placeholder when parsing
    Nothing,     // used as placeholder when parsing
    Name(String),

    NewLine,

    IsLocal,
    IsGlobal,
    Noun(JArray),
    Verb(VerbImpl),
    Adverb(ModifierImpl),
    Conjunction(ModifierImpl),
    IfBlock(Vec<Word>),
    SelectBlock(Vec<Word>),
    TryBlock(Vec<Word>),
    ForBlock(Option<String>, Vec<Word>),
    // bool: false: while (check at start), true: whilst (check at end)
    WhileBlock(bool, Vec<Word>),
    AssertLine(Vec<Word>),

    Comment,
    DirectDefUnknown, // {{
    DirectDef(char),  // {{)n
    DirectDefEnd,     // }}

    If,
    Do,
    Else,
    ElseIf,
    End,

    Try,
    Catch,
    CatchD,
    CatchT,
    Throw,
    Return,

    For(Option<String>),
    While,
    Whilst,

    Select,
    Case,

    Assert,
}

impl Word {
    /// primarily intended for asserts, hence the "static", and the PANIC on invalid input
    pub fn static_verb(v: &'static str) -> Word {
        Word::Verb(primitive_verbs(v).expect("static verbs should be valid"))
    }

    pub fn is_control_symbol(&self) -> bool {
        use Word::*;
        match self {
            DirectDef(_) | DirectDefUnknown | DirectDefEnd => true,
            If | Do | Else | ElseIf | End => true,
            For(_) | While | Whilst => true,
            Try | Catch | CatchD | CatchT => true,
            Select | Case => true,
            Assert => true,
            LP | RP | Name(_) | IsLocal | IsGlobal => false,
            Verb(_) | Noun(_) | Adverb(_) | Conjunction(_) => false,
            IfBlock(_)
            | ForBlock(_, _)
            | WhileBlock(_, _)
            | AssertLine(_)
            | TryBlock(_)
            | SelectBlock(_) => false,
            Throw | Return => false,
            NewLine => false,
            StartOfLine | Nothing => false,
            Comment => unreachable!("should have been removed from the stream by now"),
        }
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Word::Noun(a) => write!(f, "{}", a),
            Word::Verb(v) => write!(f, "{}", v.name()),
            Word::Adverb(a) => write!(f, "{}", a.name()),
            Word::Conjunction(c) => write!(f, "{}", c.name()),
            Word::Nothing => Ok(()),
            //_ => write!(f, "{:+}", self),
            _ => write!(f, "XXX TODO: unable to Display Word::{:?}", self),
        }
    }
}

macro_rules! impl_from_atom {
    ($t:ty, $j:path) => {
        impl From<$t> for Word {
            fn from(value: $t) -> Word {
                Word::Noun($j(ArcArray::from_elem(IxDyn(&[]), value.into())))
            }
        }
    };
}
impl_from_atom!(char, JArray::CharArray);
impl_from_atom!(u8, JArray::BoolArray);
impl_from_atom!(i32, JArray::IntArray);
impl_from_atom!(i64, JArray::IntArray);
impl_from_atom!(BigInt, JArray::ExtIntArray);
impl_from_atom!(BigRational, JArray::RationalArray);
impl_from_atom!(f64, JArray::FloatArray);
impl_from_atom!(Complex64, JArray::ComplexArray);

impl Word {
    pub fn name(&self) -> String {
        use Word::*;
        match self {
            Name(s) => s.to_string(),
            Verb(v) => v.name(),
            Noun(arr) => quote_arr(arr),
            Adverb(v) => v.name(),
            Conjunction(v) => v.name(),
            WhileBlock(b, block) => {
                if *b {
                    format!("whilst. {} end.", stringify(block))
                } else {
                    format!("while. {} end.", stringify(block))
                }
            }
            ForBlock(c, block) => format!(
                "for_{}. {} end.",
                c.as_ref().map(|s| s.as_str()).unwrap_or(""),
                stringify(block)
            ),
            IfBlock(block) => format!("if. {} end.", stringify(block)),
            SelectBlock(block) => format!("select. {} end.", stringify(block)),
            Case => "case.".to_string(),
            Do => "do.".to_string(),
            Else => "else.".to_string(),
            ElseIf => "elseif.".to_string(),
            End => "end.".to_string(),
            Return => "return.".to_string(),
            NewLine => "\n".to_string(),
            IsLocal => "=.".to_string(),
            IsGlobal => "=:".to_string(),
            LP => "(".to_string(),
            RP => ")".to_string(),
            _ => format!("TODO: can't Word::name {self:?}"),
        }
    }

    pub fn boxed_ar(&self) -> Result<JArray> {
        use Word::*;
        // TODO: not quite a copy-paste of tie_top
        match self {
            Noun(a) => Ok(JArray::from_list([JArray::from_string("0"), a.clone()])),
            Verb(v) => v.boxed_ar(),
            Name(s) => Ok(JArray::from_string(s)),
            Adverb(m) | Conjunction(m) => m.boxed_ar(),
            _ => Err(JError::NonceError).with_context(|| anyhow!("can't Word::boxed_ar {self:?}")),
        }
    }
}
