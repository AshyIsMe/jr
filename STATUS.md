# Implementation Status
This file auto-generated: just get-impl-status


## Implemented Verbs
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
        "-:" => primitive("-:", v_halve, v_match, (0, inf, inf)),
        "%" => primitive("%", v_reciprocal, v_divide, (0, 0, 0)),
        "%." => primitive("%.", v_matrix_inverse, v_matrix_divide, (2, inf, 2)),
        "%:" => primitive("%:", v_square_root, v_root, (0, 0, 0)),
        "^" => primitive("^", v_exponential, v_power, (0, 0, 0)),
        "^." => primitive("^.", v_natural_log, v_logarithm, (0, 0, 0)),
        "$" => primitive("$", v_shape_of, v_shape, (inf, 1, inf)),
        "~:" => primitive("~:", v_nub_sieve, v_not_equal, (inf, 0, 0)),
        "|" => primitive("|", v_magnitude, v_residue, (0, 0, 0)),
        "|." => primitive("|.", v_reverse, v_rotate_shift, (inf, inf, inf)),
        "|:" => primitive("|:", v_transpose, v_transpose_dyad, (inf, inf, inf)),
        "," => primitive(",", v_ravel, v_append, (inf, inf, inf)),
        ",." => primitive(",.", v_ravel_items, v_stitch, (inf, inf, inf)),
        ",:" => primitive(",:", v_itemize, v_laminate, (inf, inf, inf)),
        ";" => primitive(";", v_raze, v_link, (inf, inf, inf)),
        ";:" => primitive(";:", v_words, v_sequential_machine, (1, inf, inf)),
        "#" => primitive("#", v_tally, v_copy, (inf, 1, inf)),
        "#." => primitive("#.", v_base_, v_base, (1, 1, 1)),
        "#:" => primitive("#:", v_antibase_, v_antibase, (inf, 1, 0)),
        "!" => primitive("!", v_factorial, v_out_of, (0, 0, 0)),
        "/:" => primitive("/:", v_grade_up, v_sort_up, (inf, inf, inf)),
        "\\:" => primitive("\\:", v_grade_down, v_sort_down, (inf, inf, inf)),
        "[" => primitive("[", v_same, v_left, (inf, inf, inf)),
        "]" => primitive("]", v_same, v_right, (inf, inf, inf)),
        "{" => primitive("{", v_catalogue, v_from, (1, 0, inf)),
        "{." => primitive("{.", v_head, v_take, (inf, 1, inf)),
        "{:" => primitive("{:", v_tail, v_not_implemented_dyad, (inf, inf, inf)),
        "}:" => primitive("}:", v_curtail, v_not_implemented_dyad, (inf, inf, inf)),
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

## Implemented Adverbs
        "~" => adverb("~", a_tilde),
        "/" => adverb("/", a_slash),
        "/." => adverb("/.", a_slash_dot),

## Implemented Conjunctions
        "^:" => conj("^:", c_hatco),
        ";." => conj(";.", c_cut),
        "!:" => conj("!:", c_foreign),
        "\"" => conj("\"", c_quote),
        "@" => conj("@", c_at),
        "&" => conj("&", c_bondo),

## Not Implemented Yet
        //"=." => not_impl("=."), IsLocal
        //"=:" => not_impl("=:"), IsGlobal
        "_:" => not_impl("_:"),
        "^!." => not_impl("^!."),
        "$." => not_impl("$."),
        "$:" => not_impl("$:"),
        ".:" => not_impl(".:"),
        ".." => not_impl(".."),
        "[:" => not_impl("[:"),
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
        "NB." => not_impl("NB."),
        "{{" => not_impl("{{"),
        "}}" => not_impl("}}"),
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
        "\\" => adverb("\\", a_not_implemented),
        "\\." => adverb("\\.", a_not_implemented),
        "]:" => adverb("]:", a_not_implemented),
        "}" => adverb("}", a_not_implemented),
        "b." => adverb("b.", a_not_implemented),
        "f." => adverb("f.", a_not_implemented),
        "M." => adverb("M.", a_not_implemented),
        "." => conj(".", c_not_implemented),
        ":." => conj(":.", c_not_implemented),
        "::" => conj("::", c_not_implemented),
        "!." => conj("!.", c_not_implemented),
        "[." => conj("[.", c_not_implemented),
        "]." => conj("].", c_not_implemented),
        "`" => conj("`", c_not_implemented),
        "`:" => conj("`:", c_not_implemented),
        "@." => conj("@.", c_not_implemented),
        "@:" => conj("@:", c_not_implemented),
        "&." => conj("&.", c_not_implemented),
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

```
pub fn v_less_or_equal(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_larger_or_equal(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_gcd_or(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_lcm_and(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_matrix_inverse(_y: &JArray) -> Result<JArray> {
pub fn v_matrix_divide(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_root(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_natural_log(_y: &JArray) -> Result<JArray> {
pub fn v_logarithm(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_magnitude(_y: &JArray) -> Result<JArray> {
pub fn v_residue(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_factorial(_y: &JArray) -> Result<JArray> {
pub fn v_out_of(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_deal_fixed_seed(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_prime_factors(_y: &JArray) -> Result<JArray> {
pub fn v_prime_exponents(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_angle(_y: &JArray) -> Result<JArray> {
pub fn v_polar(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_transpose_dyad(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_append(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_stitch(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_copy(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_fetch(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_not_implemented_monad(_y: &JArray) -> Result<JArray> {
pub fn v_not_implemented_dyad(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_self_classify(y: &JArray) -> Result<JArray> {
pub fn v_less(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_match(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_nub_sieve(_y: &JArray) -> Result<JArray> {
pub fn v_reverse(_y: &JArray) -> Result<JArray> {
pub fn v_rotate_shift(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_ravel_items(_y: &JArray) -> Result<JArray> {
pub fn v_itemize(_y: &JArray) -> Result<JArray> {
pub fn v_laminate(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_raze(_y: &JArray) -> Result<JArray> {
pub fn v_words(_y: &JArray) -> Result<JArray> {
pub fn v_sequential_machine(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_base_(_y: &JArray) -> Result<JArray> {
pub fn v_base(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_antibase_(_y: &JArray) -> Result<JArray> {
pub fn v_antibase(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_grade_up(_y: &JArray) -> Result<JArray> {
pub fn v_sort_up(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_grade_down(_y: &JArray) -> Result<JArray> {
pub fn v_sort_down(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_catalogue(_y: &JArray) -> Result<JArray> {
pub fn v_from(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_map(_y: &JArray) -> Result<JArray> {
pub fn v_numbers(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_default_format(_y: &JArray) -> Result<JArray> {
pub fn v_format(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_anagram_index(_y: &JArray) -> Result<JArray> {
pub fn v_anagram(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_cycledirect(_y: &JArray) -> Result<JArray> {
pub fn v_permute(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_raze_in(_y: &JArray) -> Result<JArray> {
pub fn v_member_in(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_index_of(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_steps(_y: &JArray) -> Result<JArray> {
pub fn v_index_of_last(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_indices(_y: &JArray) -> Result<JArray> {
pub fn v_interval_index(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_circle_function(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_roots(_y: &JArray) -> Result<JArray> {
pub fn v_polynomial(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_poly_deriv(_y: &JArray) -> Result<JArray> {
pub fn v_poly_integral(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_extend_precision(_y: &JArray) -> Result<JArray> {
pub fn v_num_denom(x: &JArray, y: &JArray) -> Result<JArray> {
```
