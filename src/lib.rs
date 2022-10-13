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

macro_rules! simple {
    ($s:expr, $monad:expr, $dyad: expr) => {
        ($s, VerbImpl::Simple(SimpleImpl::new($s, $monad, $dyad)))
    };
}

fn primitive_verbs() -> HashMap<&'static str, VerbImpl> {
    use verbs::*;
    HashMap::from([
        // (echo '<table>'; <~/Downloads/Vocabulary.html fgrep '&#149;' | sed 's/<td nowrap>/<tr><td>/g') > a.html; links -dump a.html | perl -ne 's/\s*$/\n/; my ($a,$b,$c) = $_ =~ /\s+([^\s]+) (.*?) \xc2\x95 (.+?)$/; $b =~ tr/A-Z/a-z/; $c =~ tr/A-Z/a-z/; $b =~ s/[^a-z ]//g; $c =~ s/[^a-z -]//g; $b =~ s/ +|-/_/g; $c =~ s/ +|-/_/g; print "simple!(\"$a\", v_$b, v_$c);\n"'
        simple!("=", v_self_classify, v_equal),
        simple!("<", v_box, v_less_than),
        simple!("<.", v_floor, v_lesser_of_min),
        simple!("<:", v_decrement, v_less_or_equal),
        simple!(">", v_open, v_larger_than),
        simple!(">.", v_ceiling, v_larger_of_max),
        simple!(">:", v_increment, v_larger_or_equal),
        simple!("+", v_conjugate, v_plus),
        simple!("+.", v_real_imaginary, v_gcd_or),
        simple!("+:", v_double, v_not_or),
        simple!("*", v_signum, v_times),
        simple!("*.", v_lengthangle, v_lcm_and),
        simple!("*:", v_square, v_not_and),
        simple!("-", v_negate, v_minus),
        simple!("-.", v_not, v_less),
        simple!("-:", v_halve, v_match),
        simple!("%", v_reciprocal, v_divide),
        simple!("%.", v_matrix_inverse, v_matrix_divide),
        simple!("%:", v_square_root, v_root),
        simple!("^", v_exponential, v_power),
        simple!("^.", v_natural_log, v_logarithm),
        simple!("$", v_shape_of, v_shape),
        simple!("~", v_reflex, v_passive_evoke),
        simple!("~:", v_nub_sieve, v_not_equal),
        simple!("|", v_magnitude, v_residue),
        simple!("|.", v_reverse, v_rotate_shift),
        simple!(".", v_determinant, v_dot_product),
        simple!(",", v_ravel, v_append),
        simple!(",.", v_ravel_items, v_stitch),
        simple!(",:", v_itemize, v_laminate),
        simple!(";", v_raze, v_link),
        simple!(";:", v_words, v_sequential_machine),
        simple!("#", v_tally, v_copy),
        simple!("#.", v_base_, v_base),
        simple!("#:", v_antibase_, v_antibase),
        simple!("!", v_factorial, v_out_of),
        // conflicts with the adverb implementation
        // simple!("/", v_insert, v_table),
        simple!("/.", v_oblique, v_key),
        simple!("/:", v_grade_up, v_sort),
        simple!("\\", v_prefix, v_infix),
        simple!("\\.", v_suffix, v_outfix),
        simple!("\\:", v_grade_down, v_sort),
        simple!("[", v_same, v_left),
        simple!("]", v_same, v_right),
        simple!("{", v_catalogue, v_from),
        simple!("{.", v_head, v_take),
        simple!("{:", v_tail, v_map_fetch),
        simple!("}", v_item_amend, v_amend_m_u),
        simple!("}.", v_behead, v_drop),
        simple!("\".", v_do, v_numbers),
        simple!("\":", v_default_format, v_format),
        simple!("?", v_roll, v_deal),
        simple!("?.", v_roll, v_deal_fixed_seed),
        simple!("A.", v_anagram_index, v_anagram),
        simple!("C.", v_cycledirect, v_permute),
        simple!("e.", v_raze_in, v_member_in),
        simple!("i.", v_integers, v_index_of),
        simple!("i:", v_steps, v_index_of_last),
        simple!("I.", v_indices, v_interval_index),
        simple!("j.", v_imaginary, v_complex),
        simple!("o.", v_pi_times, v_circle_function),
        simple!("p.", v_roots, v_polynomial),
        simple!("p..", v_poly_deriv, v_poly_integral),
        simple!("q:", v_prime_factors, v_prime_exponents),
        simple!("r.", v_angle, v_polar),
        not_impl!("="),
        //not_impl!("=."), IsLocal
        //not_impl!("=:"), IsGlobal
        // ("<", VerbImpl::Simple(SimpleImpl::new("<", verbs::v_box, verbs::v_lt))),
        not_impl!("<:"),
        not_impl!(">."),
        not_impl!(">:"),
        not_impl!("_:"),
        not_impl!("+."),
        not_impl!("+:"),
        not_impl!("*."),
        not_impl!("-."),
        not_impl!("-:"),
        not_impl!("%."),
        not_impl!("%:"),
        not_impl!("^"),
        not_impl!("^."),
        not_impl!("^!."),
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
        not_impl!(";:"),
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
