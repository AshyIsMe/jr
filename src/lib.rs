mod arrays;
pub mod cells;
mod empty;
mod error;
mod eval;
mod modifiers;
mod number;
mod scan;
mod verbs;

#[cfg(feature = "ui")]
mod plot;

// laziness
pub use arrays::{Elem, IntoJArray, JArray, Word};
pub use cells::flatten;
pub use empty::HasEmpty;

// public API
pub use crate::error::JError;
pub use crate::eval::eval;

// e.g. cli syntax highlighting
pub use scan::{scan, scan_with_locations};

use modifiers::ModifierImpl;
use verbs::VerbImpl;

// TODO: helper function for tests, not really public
pub use crate::eval::resolve_names;
pub use crate::modifiers::collect_nouns;
pub use crate::scan::char_array;
pub use crate::verbs::Rank;

fn primitive_verbs(sentence: &str) -> Option<VerbImpl> {
    use verbs::*;
    let inf = u32::MAX;
    let primitive = |op, monad, dyad, ranks: (u32, u32, u32)| {
        VerbImpl::Primitive(PrimitiveImpl::new(
            op,
            monad,
            dyad,
            (
                Rank::new(ranks.0).unwrap(),
                Rank::new(ranks.1).unwrap(),
                Rank::new(ranks.2).unwrap(),
            ),
        ))
    };
    let not_impl = |op| {
        primitive(
            op,
            v_not_implemented_monad,
            v_not_implemented_dyad,
            (inf, inf, inf),
        )
    };

    Some(match sentence {
        // (echo '<table>'; <~/Downloads/Vocabulary.html fgrep '&#149;' | sed 's/<td nowrap>/<tr><td>/g') > a.html; links -dump a.html | perl -ne 's/\s*$/\n/; my ($a,$b,$c) = $_ =~ /\s+([^\s]+) (.*?) \xc2\x95 (.+?)$/; $b =~ tr/A-Z/a-z/; $c =~ tr/A-Z/a-z/; $b =~ s/[^a-z ]//g; $c =~ s/[^a-z -]//g; $b =~ s/ +|-/_/g; $c =~ s/ +|-/_/g; print "simple(\"$a\", v_$b, v_$c);\n"'
        "=" => primitive("=", v_self_classify, v_equal, (inf, 0, 0)),
        "<" => primitive("<", v_box, v_less_than, (inf, 0, 0)),
        "<." => primitive("<.", v_floor, v_lesser_of_min, (0, 0, 0)),
        "<:" => primitive("<:", v_decrement, v_less_or_equal, (0, 0, 0)),
        ">" => primitive(">", v_open, v_larger_than, (0, 0, 0)),
        ">." => primitive(">.", v_ceiling, v_larger_of_max, (0, 0, 0)),
        ">:" => primitive(">:", v_increment, v_larger_or_equal, (0, 0, 0)),

        "+" => primitive("+", v_conjugate, v_plus, (0, 0, 0)),
        "+." => primitive("+.", v_real_imaginary, v_gcd_or, (0, 0, 0)),
        "+:" => primitive("+:", v_double, v_not_or, (0, 0, 0)),
        "*" => primitive("*", v_signum, v_times, (0, 0, 0)),
        "*." => primitive("*.", v_length_angle, v_lcm_and, (0, 0, 0)),
        "*:" => primitive("*:", v_square, v_not_and, (0, 0, 0)),
        "-" => primitive("-", v_negate, v_minus, (0, 0, 0)),
        "-." => primitive("-.", v_not, v_less, (0, inf, inf)),
        "-:" => primitive("-:", v_halve, v_match, (inf, inf, 0)),
        "%" => primitive("%", v_reciprocal, v_divide, (0, 0, 0)),
        "%." => primitive("%.", v_matrix_inverse, v_matrix_divide, (2, inf, 2)),
        "%:" => primitive("%:", v_square_root, v_root, (0, 0, 0)),

        "^" => primitive("^", v_exponential, v_power, (0, 0, 0)),
        "^." => primitive("^.", v_natural_log, v_logarithm, (0, 0, 0)),
        "$" => primitive("$", v_shape_of, v_shape, (inf, 1, inf)),
        "~:" => primitive("~:", v_nub_sieve, v_not_equal, (inf, 0, 0)),
        "|" => primitive("|", v_magnitude, v_residue, (0, 0, 0)),
        "|." => primitive("|.", v_reverse, v_rotate_shift, (inf, inf, inf)),

        "," => primitive(",", v_ravel, v_append, (inf, inf, inf)),
        ",." => primitive(",.", v_ravel_items, v_stitch, (inf, inf, inf)),
        ",:" => primitive(",:", v_itemize, v_laminate, (inf, inf, inf)),
        ";" => primitive(";", v_raze, v_link, (inf, inf, inf)),
        ";:" => primitive(";:", v_words, v_sequential_machine, (1, inf, inf)),

        "#" => primitive("#", v_tally, v_copy, (inf, 1, inf)),
        "#." => primitive("#.", v_base_, v_base, (1, 1, 1)),
        "#:" => primitive("#:", v_antibase_, v_antibase, (inf, 1, 0)),
        "!" => primitive("!", v_factorial, v_out_of, (0, 0, 0)),
        "/:" => primitive("/:", v_grade_up, v_sort, (inf, inf, inf)),
        "\\:" => primitive("\\:", v_grade_down, v_sort, (inf, inf, inf)),

        "[" => primitive("[", v_same, v_left, (inf, inf, inf)),
        "]" => primitive("]", v_same, v_right, (inf, inf, inf)),
        "{" => primitive("{", v_catalogue, v_from, (1, 0, inf)),
        "{." => primitive("{.", v_head, v_take, (inf, 1, inf)),
        "{:" => primitive("{:", v_tail, v_not_implemented_dyad, (inf, inf, inf)),
        "{::" => primitive("{:", v_map, v_fetch, (inf, 1, inf)),
        "}." => primitive("}.", v_behead, v_drop, (inf, 1, inf)),

        "\"." => primitive("\".", v_do, v_numbers, (1, inf, inf)),
        "\":" => primitive("\":", v_default_format, v_format, (inf, 1, inf)),
        "?" => primitive("?", v_roll, v_deal, (0, 0, 0)),
        "?." => primitive("?.", v_roll, v_deal_fixed_seed, (inf, 0, 0)),

        "A." => primitive("A.", v_anagram_index, v_anagram, (1, 0, inf)),
        "C." => primitive("C.", v_cycledirect, v_permute, (1, 1, inf)),
        "e." => primitive("e.", v_raze_in, v_member_in, (inf, inf, inf)),

        "i." => primitive("i.", v_integers, v_index_of, (1, inf, inf)),
        "i:" => primitive("i:", v_steps, v_index_of_last, (0, inf, inf)),
        "I." => primitive("I.", v_indices, v_interval_index, (1, inf, inf)),
        "j." => primitive("j.", v_imaginary, v_complex, (0, 0, 0)),
        "o." => primitive("o.", v_pi_times, v_circle_function, (0, 0, 0)),
        "p." => primitive("p.", v_roots, v_polynomial, (1, 1, 0)),
        "p.." => primitive("p..", v_poly_deriv, v_poly_integral, (1, 0, 1)),

        "q:" => primitive("q:", v_prime_factors, v_prime_exponents, (0, 0, 0)),
        "r." => primitive("r.", v_angle, v_polar, (0, 0, 0)),
        "x:" => primitive("x:", v_extend_precision, v_num_denom, (inf, inf, inf)),

        //"=." => not_impl("=."), IsLocal
        //"=:" => not_impl("=:"), IsGlobal
        // ("<", VerbImpl::Primitive(PrimitiveImpl::new("<", verbs::v_box, verbs::v_lt))),
        "_:" => not_impl("_:"),
        "^!." => not_impl("^!."),
        "$." => not_impl("$."),
        "$:" => not_impl("$:"),
        "~." => not_impl("~."),
        "|:" => not_impl("|:"),
        ".:" => not_impl(".:"),
        ".." => not_impl(".."),
        "[:" => not_impl("[:"),
        "}:" => not_impl("}:"),
        "C.!.2" => not_impl("C.!.2"),
        "E." => not_impl("E."),
        "L." => not_impl("L."),
        "p:" => not_impl("p:"),
        "s:" => not_impl("s:"),
        "T." => not_impl("T."),
        "u:" => not_impl("u:"),
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
        "plot." => VerbImpl::Primitive(PrimitiveImpl::monad("plot.", v_plot)),
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

fn primitive_adverbs(sentence: &str) -> Option<ModifierImpl> {
    Some(match sentence {
        "~" => ModifierImpl::NotImplemented,
        "/" => ModifierImpl::Slash,
        "/." => ModifierImpl::NotImplemented,
        "\\" => ModifierImpl::NotImplemented,
        "\\." => ModifierImpl::NotImplemented,
        "]:" => ModifierImpl::NotImplemented,
        "}" => ModifierImpl::CurlyRt,
        "b." => ModifierImpl::NotImplemented,
        "f." => ModifierImpl::NotImplemented,
        "M." => ModifierImpl::NotImplemented,
        _ => return None,
    })
}

pub fn primitive_nouns(sentence: &str) -> Option<Word> {
    // https://code.jsoftware.com/wiki/NuVoc
    Some(match sentence {
        //https://code.jsoftware.com/wiki/Vocabulary/adot
        "a." => {
            // TODO Fix this:
            // A chunk of alphabet is jumbled around (sorta, it's complicated...)
            //    |:(16+i.11) ([ ; {)"0 _ a.
            // ┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
            // │16│17│18│19│20│21│22│23│24│25│26│
            // ├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤
            // │┌ │┬ │┐ │├ │┼ │┤ │└ │┴ │┘ ││ │─ │
            // └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
            // Extended ascii codes from here: https://www.asciitable.com/
            // 218 194 191 195 197 180 192 193 217 179 196
            // This doesn't do what I hoped:
            // let ascii_ints = [
            //     (0..=15u8).collect(),
            //     vec![218u8, 194, 191, 195, 197, 180, 192, 193, 217, 179, 196],
            //     (27..=255u8).collect(),
            // ]
            // .concat();
            let ascii_ints: Vec<u8> = (0..=255u8).collect();
            char_array(ascii_ints.iter().map(|i| *i as char).collect::<String>()).unwrap()
        }
        //"a:" => Word::Noun(JArray::BoxArray(arr0d([]))),
        // TODO declare a: properly instead of the scan hack
        "a:" => scan("<0$0").unwrap()[0].clone(),
        _ => return None,
    })
}

fn primitive_conjunctions(sentence: &str) -> Option<ModifierImpl> {
    // https://code.jsoftware.com/wiki/NuVoc
    Some(match sentence {
        "^:" => ModifierImpl::HatCo,
        "." => ModifierImpl::NotImplemented,
        ":" => ModifierImpl::NotImplemented,
        ":." => ModifierImpl::NotImplemented,
        "::" => ModifierImpl::NotImplemented,
        ";." => ModifierImpl::NotImplemented,
        "!." => ModifierImpl::NotImplemented,
        "!:" => ModifierImpl::NotImplemented,
        "[." => ModifierImpl::NotImplemented,
        "]." => ModifierImpl::NotImplemented,
        "\"" => ModifierImpl::Quote,
        "`" => ModifierImpl::NotImplemented,
        "`:" => ModifierImpl::NotImplemented,
        "@" => ModifierImpl::NotImplemented,
        "@." => ModifierImpl::NotImplemented,
        "@:" => ModifierImpl::NotImplemented,
        "&" => ModifierImpl::NotImplemented,
        "&." => ModifierImpl::NotImplemented,
        "&:" => ModifierImpl::NotImplemented,
        "&.:" => ModifierImpl::NotImplemented,
        "d." => ModifierImpl::NotImplemented,
        "D." => ModifierImpl::NotImplemented,
        "D:" => ModifierImpl::NotImplemented,
        "F." => ModifierImpl::NotImplemented,
        "F.." => ModifierImpl::NotImplemented,
        "F.:" => ModifierImpl::NotImplemented,
        "F:" => ModifierImpl::NotImplemented,
        "F:." => ModifierImpl::NotImplemented,
        "F::" => ModifierImpl::NotImplemented,
        "H." => ModifierImpl::NotImplemented,
        "L:" => ModifierImpl::NotImplemented,
        "S:" => ModifierImpl::NotImplemented,
        "t." => ModifierImpl::NotImplemented,
        _ => return None,
    })
}

pub fn arr0d<T>(x: T) -> ndarray::ArrayD<T> {
    ndarray::arr0(x).into_dyn()
}
