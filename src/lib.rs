mod arrays;
mod cells;
mod ctx;
mod empty;
mod error;
mod eval;
mod foreign;
mod modifiers;
mod number;
mod scan;
mod verbs;

#[cfg(feature = "ui")]
mod plot;

pub mod test_impls;
use crate::test_impls::scan_eval;

// laziness
pub use arrays::{Elem, JArray, Word};
pub use empty::HasEmpty;

// public API
pub use crate::ctx::Ctx;
pub use crate::error::JError;
pub use crate::eval::{eval, feed, EvalOutput};

// e.g. cli syntax highlighting
pub use scan::{scan, scan_with_locations};

// TODO: helper function for tests, not really public
pub use crate::arrays::IntoVec;
pub use crate::cells::generate_cells;
pub use crate::eval::resolve_names;
pub use crate::scan::char_array;
pub use crate::verbs::Rank;

// TODO: is this too much? it's necessary to construct atoms atm
pub use crate::number::Num;

// TODO: maybe as helper methods on JArray?
pub use crate::arrays::display;

use crate::arrays::ArcArrayD;
use anyhow::Result;
use modifiers::ModifierImpl;
use verbs::VerbImpl;

fn primitive_verbs(sentence: &str) -> Option<VerbImpl> {
    use verbs::*;
    fn p(
        name: &'static str,
        monad: fn(&JArray) -> Result<JArray>,
        dyad: fn(&JArray, &JArray) -> Result<JArray>,
        ranks: (Rank, DyadRank),
        inverse: impl Into<Option<&'static str>>,
    ) -> VerbImpl {
        VerbImpl::Primitive(PrimitiveImpl {
            name,
            monad: Monad {
                f: monad,
                rank: ranks.0,
            },
            dyad: Some(Dyad {
                f: dyad,
                rank: ranks.1,
            }),
            inverse: inverse.into(),
        })
    }

    let not_impl = |op| {
        p(
            op,
            v_not_implemented_monad,
            v_not_implemented_dyad,
            rank!(_ _ _),
            None,
        )
    };

    Some(match sentence {
        "=" => p("=", v_self_classify, v_equal, rank!(_ 0 0), None),
        "<" => p("<", v_box, v_less_than, rank!(_ 0 0), ">"),
        "<." => p("<.", v_floor, v_lesser_of_min, rank!(0 0 0), None),
        "<:" => p("<:", v_decrement, v_less_or_equal, rank!(0 0 0), None),
        ">" => p(">", v_open, v_larger_than, rank!(0 0 0), "<"),
        ">." => p(">.", v_ceiling, v_larger_of_max, rank!(0 0 0), None),
        ">:" => p(">:", v_increment, v_larger_or_equal, rank!(0 0 0), None),

        "+" => p("+", v_conjugate, v_plus, rank!(0 0 0), "-"),
        "+." => p("+.", v_real_imaginary, v_gcd_or, rank!(0 0 0), None),
        "+:" => p("+:", v_double, v_not_or, rank!(0 0 0), None),
        "*" => p("*", v_signum, v_times, rank!(0 0 0), None),
        "*." => p("*.", v_length_angle, v_lcm_and, rank!(0 0 0), None),
        "*:" => p("*:", v_square, v_not_and, rank!(0 0 0), "%:"),
        "-" => p("-", v_negate, v_minus, rank!(0 0 0), None),
        "-." => p("-.", v_not, v_less, rank!(0 _ _), None),
        "-:" => p("-:", v_halve, v_match, rank!(0 _ _), None),
        "%" => p("%", v_reciprocal, v_divide, rank!(0 0 0), None),
        "%." => p("%.", v_matrix_inverse, v_matrix_divide, rank!(2 _ 2), None),
        "%:" => p("%:", v_square_root, v_root, rank!(0 0 0), "*:"),

        "^" => p("^", v_exponential, v_power, rank!(0 0 0), None),
        "^." => p("^.", v_natural_log, v_logarithm, rank!(0 0 0), "^"),
        "$" => p("$", v_shape_of, v_shape, rank!(_ 1 _), None),
        "~." => VerbImpl::Primitive(PrimitiveImpl::monad("~.", v_nub)), // i, Nonenf
        "~:" => p("~:", v_nub_sieve, v_not_equal, rank!(_ 0 0), None),
        "|" => p("|", v_magnitude, v_residue, rank!(0 0 0), None),
        "|." => p("|.", v_reverse, v_rotate_shift, rank!(_ _ _), None),
        "|:" => p("|:", v_transpose, v_transpose_dyad, rank!(_ _ _), None),

        "," => p(",", v_ravel, v_append, rank!(_ _ _), None),
        ",." => p(",.", v_ravel_items, v_stitch, rank!(_ _ _), None),
        ",:" => p(",:", v_itemize, v_laminate, rank!(_ _ _), None),
        ";" => p(";", v_raze, v_link, rank!(_ _ _), None),
        ";:" => p(";:", v_words, v_sequential_machine, rank!(1 _ _), None),

        "#" => p("#", v_tally, v_copy, rank!(_ 1 _), None),
        "#." => p("#.", v_base_, v_base, rank!(1 1 1), None),
        "#:" => p("#:", v_antibase_, v_antibase, rank!(_ 1 0), None),
        "!" => p("!", v_factorial, v_out_of, rank!(0 0 0), None),
        "/:" => p("/:", v_grade_up, v_sort_up, rank!(_ _ _), None),
        "\\:" => p("\\:", v_grade_down, v_sort_down, rank!(_ _ _), None),

        "[" => p("[", v_same, v_left, rank!(_ _ _), None),
        "]" => p("]", v_same, v_right, rank!(_ _ _), None),
        "{" => p("{", v_catalogue, v_from, rank!(1 0 _), None),
        "{." => p("{.", v_head, v_take, rank!(_ 1 _), None),
        "{:" => p("{:", v_tail, v_not_implemented_dyad, rank!(_ _ _), None),
        "}:" => p("}:", v_curtail, v_not_implemented_dyad, rank!(_ _ _), None),
        "{::" => p("{::", v_map, v_fetch, rank!(_ 1 _), None),
        "}." => p("}.", v_behead, v_drop, rank!(_ 1 _), None),

        "\"." => p("\".", v_do, v_numbers, rank!(1 _ _), None),
        "\":" => p("\":", v_default_format, v_format, rank!(_ 1 _), None),
        "?" => p("?", v_roll, v_deal, rank!(0 0 0), None),
        "?." => p("?.", v_roll, v_deal_fixed_seed, rank!(_ 0 0), None),

        "A." => p("A.", v_anagram_index, v_anagram, rank!(1 0 _), None),
        "C." => p("C.", v_cycledirect, v_permute, rank!(1 1 _), None),
        "e." => p("e.", v_raze_in, v_member_in, rank!(_ _ _), None),

        "i." => p("i.", v_integers, v_index_of, rank!(1 _ _), None),
        "i:" => p("i:", v_steps, v_index_of_last, rank!(0 _ _), None),
        "I." => p("I.", v_indices, v_interval_index, rank!(1 _ _), None),
        "j." => p("j.", v_imaginary, v_complex, rank!(0 0 0), None),
        "o." => p("o.", v_pi_times, v_circle_function, rank!(0 0 0), None),
        "p." => p("p.", v_roots, v_polynomial, rank!(1 1 0), None),
        "p.." => p("p..", v_poly_deriv, v_poly_integral, rank!(1 0 1), None),

        "q:" => p("q:", v_prime_factors, v_prime_exponents, rank!(0 0 0), None),
        "r." => p("r.", v_angle, v_polar, rank!(0 0 0), None),
        "x:" => p("x:", v_extend_precision, v_num_denom, rank!(_ _ _), None),

        "$." => not_impl("$."),
        "$:" => not_impl("$:"),
        ".:" => not_impl(".:"),
        ".." => not_impl(".."),
        "[:" => VerbImpl::Cap,
        "C.!.2" => not_impl("C.!.2"),
        "E." => p(
            "E.",
            v_not_exist_monad,
            v_member_interval,
            rank!(_ _ _),
            None,
        ),
        "L." => VerbImpl::Primitive(PrimitiveImpl::monad("L.", v_levels)), // _
        "p:" => not_impl("p:"),
        "s:" => not_impl("s:"),
        "T." => not_impl("T."),
        "u:" => not_impl("u:"),
        "Z:" => not_impl("Z:"),
        "u." => not_impl("u."),
        "v." => not_impl("v."),

        // this is spelt "plot", with no ".", in jsoft's documentation
        "plot." => VerbImpl::Primitive(PrimitiveImpl::monad("plot.", v_plot)),
        _ => return None,
    })
}

fn primitive_adverbs(sentence: &str) -> Option<ModifierImpl> {
    use modifiers::*;
    let adverb = |name, f| ModifierImpl::Adverb(SimpleAdverb { name, f });
    Some(match sentence {
        "~" => adverb("~", a_tilde),
        "/" => adverb("/", a_slash),
        "/." => adverb("/.", a_slash_dot),
        "\\" => adverb("\\", a_backslash),
        "\\." => adverb("\\.", a_suffix_outfix),
        "]:" => adverb("]:", a_not_implemented),
        "}" => adverb("}", a_curlyrt),
        "b." => adverb("b.", a_bdot),
        "f." => adverb("f.", a_not_implemented),
        "M." => adverb("M.", a_not_implemented),
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
            char_array(ascii_ints.iter().map(|i| *i as char).collect::<String>())
        }
        // TODO declare a: properly instead of the scan hack
        "a:" => scan_eval("<0$0").unwrap(),
        _ => return None,
    })
}

fn primitive_conjunctions(sentence: &str) -> Option<ModifierImpl> {
    use modifiers::*;
    let conj = |name, f| ModifierImpl::Conjunction(SimpleConjunction { name, f });
    // https://code.jsoftware.com/wiki/NuVoc
    Some(match sentence {
        "^:" => conj("^:", c_hatco),
        "." => conj(".", c_not_implemented),
        ":" => ModifierImpl::Cor,
        ":." => conj(":.", c_not_implemented),
        "::" => conj("::", c_assign_adverse),
        ";." => conj(";.", c_cut),
        "!." => conj("!.", c_not_implemented),
        "!:" => conj("!:", c_foreign),
        "[." => conj("[.", c_not_implemented),
        "]." => conj("].", c_not_implemented),
        "\"" => conj("\"", c_quote),
        "`" => ModifierImpl::WordyConjunction(WordyConjunction {
            name: "`",
            f: c_tie,
        }),
        "`:" => conj("`:", c_not_implemented),
        "@" => conj("@", c_atop),
        "@." => ModifierImpl::WordyConjunction(WordyConjunction {
            name: "@.",
            f: c_agenda,
        }),
        "@:" => conj("@:", c_at),
        "&" => conj("&", c_bondo),
        "&." => conj("&.", c_under),
        "&:" => conj("&:", c_not_implemented),
        "&.:" => conj("&.:", c_not_implemented),
        "d." => conj("d.", c_not_implemented),
        "D." => conj("D.", c_not_implemented),
        "D:" => conj("D:", c_not_implemented),
        "F." => conj("F.", c_not_implemented),
        "F.." => conj("F..", c_not_implemented),
        "F.:" => conj("F.:", c_not_implemented),
        "F:" => conj("F:", c_not_implemented),
        "F:." => conj("F:.", c_not_implemented),
        "F::" => conj("F::", c_not_implemented),
        "H." => conj("H.", c_not_implemented),
        "L:" => conj("L:", c_not_implemented),
        "S:" => conj("S:", c_not_implemented),
        "t." => conj("t.", c_not_implemented),
        _ => return None,
    })
}

pub fn arr0d<T>(x: T) -> ndarray::ArrayD<T> {
    ndarray::arr0(x).into_dyn()
}

pub fn arr0ad<T>(x: T) -> ArcArrayD<T> {
    ndarray::arr0(x).into_dyn().into_shared()
}

// #[macro_export] is incompatible with #[rustfmt::skip], believe it or not
// https://github.com/rust-lang/rust/issues/74087
#[rustfmt::skip]
macro_rules! rank {
    (_         ) => (crate::Rank::infinite());
    ($m:literal) => (crate::Rank::new($m));

    (_           _          ) => ((crate::Rank::infinite(), crate::Rank::infinite()));
    (_           $dr:literal) => ((crate::Rank::infinite(), crate::Rank::new($dr)  ));
    ($dl:literal _          ) => ((crate::Rank::new($dl),   crate::Rank::infinite()));
    ($dl:literal $dr:literal) => ((crate::Rank::new($dl),   crate::Rank::new($dr)  ));


    (_          _           _          ) => ((crate::Rank::infinite(), (crate::Rank::infinite(), crate::Rank::infinite())));
    ($m:literal _           _          ) => ((crate::Rank::new($m),    (crate::Rank::infinite(), crate::Rank::infinite())));
    (_          $dl:literal _          ) => ((crate::Rank::infinite(), (crate::Rank::new($dl),   crate::Rank::infinite())));
    ($m:literal $dl:literal _          ) => ((crate::Rank::new($m),    (crate::Rank::new($dl),   crate::Rank::infinite())));
    (_          _           $dr:literal) => ((crate::Rank::infinite(), (crate::Rank::infinite(), crate::Rank::new($dr))  ));
    ($m:literal _           $dr:literal) => ((crate::Rank::new($m),    (crate::Rank::infinite(), crate::Rank::new($dr))  ));
    (_          $dl:literal $dr:literal) => ((crate::Rank::infinite(), (crate::Rank::new($dl),   crate::Rank::new($dr))  ));
    ($m:literal $dl:literal $dr:literal) => ((crate::Rank::new($m),    (crate::Rank::new($dl),   crate::Rank::new($dr))  ));
}

pub(crate) use rank;
