pub mod arrays;
pub mod eval;
pub mod modifiers;
pub mod scan;
pub mod verbs;

use std::collections::HashMap;

pub use crate::arrays::*;
pub use crate::eval::*;
pub use crate::modifiers::*;
pub use crate::scan::*;
pub use crate::verbs::*;

fn primitive_verbs() -> HashMap<&'static str, VerbImpl> {
    HashMap::from([
        ("=", VerbImpl::NotImplemented),
        //("=.", VerbImpl::NotImplemented), IsLocal
        //("=:", VerbImpl::NotImplemented), IsGlobal
        ("<", VerbImpl::LT),
        ("<.", VerbImpl::NotImplemented),
        ("<:", VerbImpl::NotImplemented),
        (">", VerbImpl::GT),
        (">.", VerbImpl::NotImplemented),
        (">:", VerbImpl::NotImplemented),
        ("_:", VerbImpl::NotImplemented),
        ("+", VerbImpl::Plus),
        ("+.", VerbImpl::NotImplemented),
        ("+:", VerbImpl::NotImplemented),
        ("*", VerbImpl::Star),
        ("*.", VerbImpl::NotImplemented),
        ("*:", VerbImpl::StarCo),
        ("-", VerbImpl::Minus),
        ("-.", VerbImpl::NotImplemented),
        ("-:", VerbImpl::NotImplemented),
        ("%", VerbImpl::Percent),
        ("%.", VerbImpl::NotImplemented),
        ("%:", VerbImpl::NotImplemented),
        ("^", VerbImpl::NotImplemented),
        ("^.", VerbImpl::NotImplemented),
        ("^!.", VerbImpl::NotImplemented),
        ("$", VerbImpl::Dollar),
        ("$.", VerbImpl::NotImplemented),
        ("$:", VerbImpl::NotImplemented),
        ("~.", VerbImpl::NotImplemented),
        ("~:", VerbImpl::NotImplemented),
        ("|", VerbImpl::NotImplemented),
        ("|.", VerbImpl::NotImplemented),
        ("|:", VerbImpl::NotImplemented),
        (".:", VerbImpl::NotImplemented),
        ("..", VerbImpl::NotImplemented),
        (",.", VerbImpl::NotImplemented),
        (",", VerbImpl::NotImplemented),
        (",:", VerbImpl::NotImplemented),
        (";", VerbImpl::Semi),
        (";:", VerbImpl::NotImplemented),
        ("#", VerbImpl::Number),
        ("#.", VerbImpl::NotImplemented),
        ("#:", VerbImpl::NotImplemented),
        ("!", VerbImpl::NotImplemented),
        ("/:", VerbImpl::NotImplemented),
        ("\\:", VerbImpl::NotImplemented),
        ("[", VerbImpl::NotImplemented),
        ("[:", VerbImpl::NotImplemented),
        ("]", VerbImpl::NotImplemented),
        ("{", VerbImpl::NotImplemented),
        ("{.", VerbImpl::NotImplemented),
        ("{:", VerbImpl::NotImplemented),
        ("{::", VerbImpl::NotImplemented),
        ("}.", VerbImpl::NotImplemented),
        ("}:", VerbImpl::NotImplemented),
        ("\".", VerbImpl::NotImplemented),
        ("\":", VerbImpl::NotImplemented),
        ("?", VerbImpl::NotImplemented),
        ("?.", VerbImpl::NotImplemented),
        ("A.", VerbImpl::NotImplemented),
        ("C.", VerbImpl::NotImplemented),
        ("C.!.2", VerbImpl::NotImplemented),
        ("e.", VerbImpl::NotImplemented),
        ("E.", VerbImpl::NotImplemented),
        ("i.", VerbImpl::IDot),
        ("i:", VerbImpl::NotImplemented),
        ("I.", VerbImpl::NotImplemented),
        ("j.", VerbImpl::NotImplemented),
        ("L.", VerbImpl::NotImplemented),
        ("o.", VerbImpl::NotImplemented),
        ("p.", VerbImpl::NotImplemented),
        ("p..", VerbImpl::NotImplemented),
        ("p:", VerbImpl::NotImplemented),
        ("q:", VerbImpl::NotImplemented),
        ("r.", VerbImpl::NotImplemented),
        ("s:", VerbImpl::NotImplemented),
        ("T.", VerbImpl::NotImplemented),
        ("u:", VerbImpl::NotImplemented),
        ("x:", VerbImpl::NotImplemented),
        ("Z:", VerbImpl::NotImplemented),
        ("_9:", VerbImpl::NotImplemented),
        ("_8:", VerbImpl::NotImplemented),
        ("_7:", VerbImpl::NotImplemented),
        ("_6:", VerbImpl::NotImplemented),
        ("_5:", VerbImpl::NotImplemented),
        ("_4:", VerbImpl::NotImplemented),
        ("_3:", VerbImpl::NotImplemented),
        ("_2:", VerbImpl::NotImplemented),
        ("_1:", VerbImpl::NotImplemented),
        ("0:", VerbImpl::NotImplemented),
        ("1:", VerbImpl::NotImplemented),
        ("2:", VerbImpl::NotImplemented),
        ("3:", VerbImpl::NotImplemented),
        ("4:", VerbImpl::NotImplemented),
        ("5:", VerbImpl::NotImplemented),
        ("6:", VerbImpl::NotImplemented),
        ("7:", VerbImpl::NotImplemented),
        ("8:", VerbImpl::NotImplemented),
        ("9", VerbImpl::NotImplemented),
        ("u.", VerbImpl::NotImplemented),
        ("v.", VerbImpl::NotImplemented),
        // TODO Controls need to be handled differently
        ("NB.", VerbImpl::NotImplemented),
        ("{{", VerbImpl::NotImplemented),
        ("}}", VerbImpl::NotImplemented),
        ("assert.", VerbImpl::NotImplemented),
        ("break.", VerbImpl::NotImplemented),
        ("continue.", VerbImpl::NotImplemented),
        ("else.", VerbImpl::NotImplemented),
        ("elseif.", VerbImpl::NotImplemented),
        ("for.", VerbImpl::NotImplemented),
        ("for_ijk.", VerbImpl::NotImplemented), // TODO handle ijk label properly
        ("goto_lbl.", VerbImpl::NotImplemented), // TODO handle lbl properly
        ("label_lbl.", VerbImpl::NotImplemented), // TODO handle lbl properly
        ("if.", VerbImpl::NotImplemented),
        ("return.", VerbImpl::NotImplemented),
        ("select.", VerbImpl::NotImplemented),
        ("case.", VerbImpl::NotImplemented),
        ("fcase.", VerbImpl::NotImplemented),
        ("throw.", VerbImpl::NotImplemented),
        ("try.", VerbImpl::NotImplemented),
        ("catch.", VerbImpl::NotImplemented),
        ("catchd.", VerbImpl::NotImplemented),
        ("catcht.", VerbImpl::NotImplemented),
        ("while.", VerbImpl::NotImplemented),
        ("whilst.", VerbImpl::NotImplemented),
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
