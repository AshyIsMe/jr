pub mod arrays;
mod empty;
mod error;
pub mod eval;
pub mod modifiers;
pub mod scan;
pub mod verbs;

#[cfg(feature = "ui")]
mod plot;

use std::collections::HashMap;

pub use arrays::*;
pub use empty::HasEmpty;
pub use error::JError;
pub use eval::*;
pub use modifiers::*;
pub use scan::*;
pub use verbs::*;

// TODO: helper function for tests, not really public
pub use crate::scan::char_array;

fn primitive_verbs(sentence: &str) -> Option<VerbImpl> {
    use verbs::*;
    let simple_rank = |op, monad, monad_rank, dyad, dyad_rank| {
        VerbImpl::Simple(SimpleImpl::new(op, monad, monad_rank, dyad, dyad_rank))
    };
    let simple =
        |op, monad, dyad| simple_rank(op, monad, Rank::infinite(), dyad, Rank::infinite_infinite());
    let not_impl = |op| simple(op, v_not_implemented_monad, v_not_implemented_dyad);

    Some(match sentence {
        // (echo '<table>'; <~/Downloads/Vocabulary.html fgrep '&#149;' | sed 's/<td nowrap>/<tr><td>/g') > a.html; links -dump a.html | perl -ne 's/\s*$/\n/; my ($a,$b,$c) = $_ =~ /\s+([^\s]+) (.*?) \xc2\x95 (.+?)$/; $b =~ tr/A-Z/a-z/; $c =~ tr/A-Z/a-z/; $b =~ s/[^a-z ]//g; $c =~ s/[^a-z -]//g; $b =~ s/ +|-/_/g; $c =~ s/ +|-/_/g; print "simple(\"$a\", v_$b, v_$c);\n"'
        "=" => simple("=", v_self_classify, v_equal),
        "<" => simple("<", v_box, v_less_than),
        "<." => simple("<.", v_floor, v_lesser_of_min),
        "<:" => simple("<:", v_decrement, v_less_or_equal),
        ">" => simple(">", v_open, v_larger_than),
        ">." => simple(">.", v_ceiling, v_larger_of_max),
        ">:" => simple(">:", v_increment, v_larger_or_equal),
        "+" => simple_rank("+", v_conjugate, Rank::zero(), v_plus, Rank::zero_zero()),
        "+." => simple("+.", v_real_imaginary, v_gcd_or),
        "+:" => simple("+:", v_double, v_not_or),
        "*" => simple("*", v_signum, v_times),
        "*." => simple("*.", v_lengthangle, v_lcm_and),
        "*:" => simple("*:", v_square, v_not_and),
        "-" => simple("-", v_negate, v_minus),
        "-." => simple("-.", v_not, v_less),
        "-:" => simple("-:", v_halve, v_match),
        "%" => simple("%", v_reciprocal, v_divide),
        "%." => simple("%.", v_matrix_inverse, v_matrix_divide),
        "%:" => simple("%:", v_square_root, v_root),
        "^" => simple("^", v_exponential, v_power),
        "^." => simple("^.", v_natural_log, v_logarithm),
        "$" => simple("$", v_shape_of, v_shape),
        "~" => simple("~", v_reflex, v_passive_evoke),
        "~:" => simple("~:", v_nub_sieve, v_not_equal),
        "|" => simple("|", v_magnitude, v_residue),
        "|." => simple("|.", v_reverse, v_rotate_shift),
        "." => simple(".", v_determinant, v_dot_product),
        "," => simple(",", v_ravel, v_append),
        ",." => simple(",.", v_ravel_items, v_stitch),
        ",:" => simple(",:", v_itemize, v_laminate),
        ";" => simple(";", v_raze, v_link),
        ";:" => simple(";:", v_words, v_sequential_machine),
        "#" => simple("#", v_tally, v_copy),
        "#." => simple("#.", v_base_, v_base),
        "#:" => simple("#:", v_antibase_, v_antibase),
        "!" => simple("!", v_factorial, v_out_of),
        // conflicts with the adverb implementation
        // "/" => simple("/", v_insert, v_table),
        "/." => simple("/.", v_oblique, v_key),
        "/:" => simple("/:", v_grade_up, v_sort),
        "\\" => simple("\\", v_prefix, v_infix),
        "\\." => simple("\\.", v_suffix, v_outfix),
        "\\:" => simple("\\:", v_grade_down, v_sort),
        "[" => simple("[", v_same, v_left),
        "]" => simple("]", v_same, v_right),
        "{" => simple("{", v_catalogue, v_from),
        "{." => simple("{.", v_head, v_take),
        "{:" => simple("{:", v_tail, v_map_fetch),
        "}" => simple("}", v_item_amend, v_amend_m_u),
        "}." => simple("}.", v_behead, v_drop),
        "\"." => simple("\".", v_do, v_numbers),
        "\":" => simple("\":", v_default_format, v_format),
        "?" => simple("?", v_roll, v_deal),
        "?." => simple("?.", v_roll, v_deal_fixed_seed),
        "A." => simple("A.", v_anagram_index, v_anagram),
        "C." => simple("C.", v_cycledirect, v_permute),
        "e." => simple("e.", v_raze_in, v_member_in),
        "i." => simple("i.", v_integers, v_index_of),
        "i:" => simple("i:", v_steps, v_index_of_last),
        "I." => simple("I.", v_indices, v_interval_index),
        "j." => simple("j.", v_imaginary, v_complex),
        "o." => simple("o.", v_pi_times, v_circle_function),
        "p." => simple("p.", v_roots, v_polynomial),
        "p.." => simple("p..", v_poly_deriv, v_poly_integral),
        "q:" => simple("q:", v_prime_factors, v_prime_exponents),
        "r." => simple("r.", v_angle, v_polar),
        //"=." => not_impl("=."), IsLocal
        //"=:" => not_impl("=:"), IsGlobal
        // ("<", VerbImpl::Simple(SimpleImpl::new("<", verbs::v_box, verbs::v_lt))),
        "_:" => not_impl("_:"),
        "^!." => not_impl("^!."),
        "$." => not_impl("$."),
        "$:" => not_impl("$:"),
        "~." => not_impl("~."),
        "|:" => not_impl("|:"),
        ".:" => not_impl(".:"),
        ".." => not_impl(".."),
        "[:" => not_impl("[:"),
        "{::" => not_impl("{::"),
        "}:" => not_impl("}:"),
        "C.!.2" => not_impl("C.!.2"),
        "E." => not_impl("E."),
        "L." => not_impl("L."),
        "p:" => not_impl("p:"),
        "s:" => not_impl("s:"),
        "T." => not_impl("T."),
        "u:" => not_impl("u:"),
        "x:" => simple("x:", v_extend_precision, v_num_denom),
        "Z:" => not_impl("Z:"),
        "_9:" => not_impl("_9:"),
        "_8:" => not_impl("_8:"),
        "_7:" => not_impl("_7:"),
        "_6:" => not_impl("_6:"),
        "_5:" => not_impl("_5:"),
        "_4:" => not_impl("_4:"),
        "_3:" => not_impl("_3:"),
        "_2:" => not_impl("_2:"),
        "_1:" => not_impl("_1:"),
        "0:" => not_impl("0:"),
        "1:" => not_impl("1:"),
        "2:" => not_impl("2:"),
        "3:" => not_impl("3:"),
        "4:" => not_impl("4:"),
        "5:" => not_impl("5:"),
        "6:" => not_impl("6:"),
        "7:" => not_impl("7:"),
        "8:" => not_impl("8:"),
        "9" => not_impl("9"),
        "u." => not_impl("u."),
        "v." => not_impl("v."),
        // TODO Controls need to be handled differently
        "NB." => not_impl("NB."),
        "{{" => not_impl("{{"),
        "}}" => not_impl("}}"),
        "plot." => VerbImpl::Simple(SimpleImpl::monad("plot.", v_plot)),
        "assert." => not_impl("assert."),
        "break." => not_impl("break."),
        "continue." => not_impl("continue."),
        "else." => not_impl("else."),
        "elseif." => not_impl("elseif."),
        "for." => not_impl("for."),
        "for_ijk." => not_impl("for_ijk."), // TODO handle ijk label properly
        "goto_lbl." => not_impl("goto_lbl."), // TODO handle lbl properly
        "label_lbl." => not_impl("label_lbl."), // TODO handle lbl properly
        "if." => not_impl("if."),
        "return." => not_impl("return."),
        "select." => not_impl("select."),
        "case." => not_impl("case."),
        "fcase." => not_impl("fcase."),
        "throw." => not_impl("throw."),
        "try." => not_impl("try."),
        "catch." => not_impl("catch."),
        "catchd." => not_impl("catchd."),
        "catcht." => not_impl("catcht."),
        "while." => not_impl("while."),
        "whilst." => not_impl("whilst."),

        _ => return None,
    })
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
