pub mod arrays;
pub mod eval;
pub mod modifiers;
pub mod scan;
pub mod verbs;

#[cfg(feature = "ui")]
mod plot;

use std::collections::HashMap;

pub use crate::arrays::*;
pub use crate::eval::*;
pub use crate::modifiers::*;
pub use crate::scan::*;
pub use crate::verbs::*;

macro_rules! not_impl {
    ($s:expr) => {
        ($s, VerbImpl::NotImplemented($s.to_string()))
    };
}

fn primitive_verbs() -> HashMap<&'static str, VerbImpl> {
    HashMap::from([
        not_impl!("="),
        //not_impl!("=."), IsLocal
        //not_impl!("=:"), IsGlobal
        ("<", VerbImpl::LT),
        not_impl!("<."),
        not_impl!("<:"),
        (">", VerbImpl::GT),
        not_impl!(">."),
        not_impl!(">:"),
        not_impl!("_:"),
        ("+", VerbImpl::Plus),
        not_impl!("+."),
        not_impl!("+:"),
        ("*", VerbImpl::Star),
        not_impl!("*."),
        ("*:", VerbImpl::StarCo),
        ("-", VerbImpl::Minus),
        not_impl!("-."),
        not_impl!("-:"),
        ("%", VerbImpl::Percent),
        not_impl!("%."),
        not_impl!("%:"),
        not_impl!("^"),
        not_impl!("^."),
        not_impl!("^!."),
        ("$", VerbImpl::Dollar),
        not_impl!("$."),
        not_impl!("$:"),
        not_impl!("~."),
        not_impl!("~:"),
        not_impl!("|"),
        not_impl!("|."),
        not_impl!("|:"),
        not_impl!(".:"),
        not_impl!(".."),
        not_impl!(",."),
        not_impl!(","),
        not_impl!(",:"),
        (";", VerbImpl::Semi),
        not_impl!(";:"),
        ("#", VerbImpl::Number),
        not_impl!("#."),
        not_impl!("#:"),
        not_impl!("!"),
        not_impl!("/:"),
        not_impl!("\\:"),
        not_impl!("["),
        not_impl!("[:"),
        not_impl!("]"),
        not_impl!("{"),
        not_impl!("{."),
        not_impl!("{:"),
        not_impl!("{::"),
        not_impl!("}."),
        not_impl!("}:"),
        not_impl!("\"."),
        not_impl!("\":"),
        not_impl!("?"),
        not_impl!("?."),
        not_impl!("A."),
        not_impl!("C."),
        not_impl!("C.!.2"),
        not_impl!("e."),
        not_impl!("E."),
        ("i.", VerbImpl::IDot),
        not_impl!("i:"),
        not_impl!("I."),
        not_impl!("j."),
        not_impl!("L."),
        not_impl!("o."),
        not_impl!("p."),
        not_impl!("p.."),
        not_impl!("p:"),
        not_impl!("q:"),
        not_impl!("r."),
        not_impl!("s:"),
        not_impl!("T."),
        not_impl!("u:"),
        not_impl!("x:"),
        not_impl!("Z:"),
        not_impl!("_9:"),
        not_impl!("_8:"),
        not_impl!("_7:"),
        not_impl!("_6:"),
        not_impl!("_5:"),
        not_impl!("_4:"),
        not_impl!("_3:"),
        not_impl!("_2:"),
        not_impl!("_1:"),
        not_impl!("0:"),
        not_impl!("1:"),
        not_impl!("2:"),
        not_impl!("3:"),
        not_impl!("4:"),
        not_impl!("5:"),
        not_impl!("6:"),
        not_impl!("7:"),
        not_impl!("8:"),
        not_impl!("9"),
        not_impl!("u."),
        not_impl!("v."),
        // TODO Controls need to be handled differently
        not_impl!("NB."),
        not_impl!("{{"),
        not_impl!("}}"),
        ("plot.", VerbImpl::Plot),
        not_impl!("assert."),
        not_impl!("break."),
        not_impl!("continue."),
        not_impl!("else."),
        not_impl!("elseif."),
        not_impl!("for."),
        not_impl!("for_ijk."),   // TODO handle ijk label properly
        not_impl!("goto_lbl."),  // TODO handle lbl properly
        not_impl!("label_lbl."), // TODO handle lbl properly
        not_impl!("if."),
        not_impl!("return."),
        not_impl!("select."),
        not_impl!("case."),
        not_impl!("fcase."),
        not_impl!("throw."),
        not_impl!("try."),
        not_impl!("catch."),
        not_impl!("catchd."),
        not_impl!("catcht."),
        not_impl!("while."),
        not_impl!("whilst."),
    ])
}

fn primitive_adverbs() -> HashMap<&'static str, ModifierImpl> {
    HashMap::from([
        ("~", ModifierImpl::NotImplemented),
        ("/", ModifierImpl::Slash),
        ("/.", ModifierImpl::NotImplemented),
        ("\\", ModifierImpl::NotImplemented),
        ("\\.", ModifierImpl::NotImplemented),
        ("]:", ModifierImpl::NotImplemented),
        ("}", ModifierImpl::CurlyRt),
        ("b.", ModifierImpl::NotImplemented),
        ("f.", ModifierImpl::NotImplemented),
        ("M.", ModifierImpl::NotImplemented),
    ])
}

fn primitive_nouns() -> &'static [&'static str] {
    // TODO
    // https://code.jsoftware.com/wiki/NuVoc
    &["_", "_.", "a.", "a:"]
}

fn primitive_conjunctions() -> HashMap<&'static str, ModifierImpl> {
    // https://code.jsoftware.com/wiki/NuVoc
    HashMap::from([
        ("^:", ModifierImpl::NotImplemented),
        (".", ModifierImpl::NotImplemented),
        (":", ModifierImpl::NotImplemented),
        (":.", ModifierImpl::NotImplemented),
        ("::", ModifierImpl::NotImplemented),
        (";.", ModifierImpl::NotImplemented),
        ("!.", ModifierImpl::NotImplemented),
        ("!:", ModifierImpl::NotImplemented),
        ("[.", ModifierImpl::NotImplemented),
        ("].", ModifierImpl::NotImplemented),
        ("\"", ModifierImpl::NotImplemented),
        ("`", ModifierImpl::NotImplemented),
        ("`:", ModifierImpl::NotImplemented),
        ("@", ModifierImpl::NotImplemented),
        ("@.", ModifierImpl::NotImplemented),
        ("@:", ModifierImpl::NotImplemented),
        ("&", ModifierImpl::NotImplemented),
        ("&.", ModifierImpl::NotImplemented),
        ("&:", ModifierImpl::NotImplemented),
        ("&.:", ModifierImpl::NotImplemented),
        ("d.", ModifierImpl::NotImplemented),
        ("D.", ModifierImpl::NotImplemented),
        ("D:", ModifierImpl::NotImplemented),
        ("F.", ModifierImpl::NotImplemented),
        ("F..", ModifierImpl::NotImplemented),
        ("F.:", ModifierImpl::NotImplemented),
        ("F:", ModifierImpl::NotImplemented),
        ("F:.", ModifierImpl::NotImplemented),
        ("F::", ModifierImpl::NotImplemented),
        ("H.", ModifierImpl::NotImplemented),
        ("L:", ModifierImpl::NotImplemented),
        ("S:", ModifierImpl::NotImplemented),
        ("t.", ModifierImpl::NotImplemented),
    ])
}
