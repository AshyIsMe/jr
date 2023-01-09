# Implementation Status

## Not Implemented Yet
        "_:" => not_impl("_:"),
        "^!." => not_impl("^!."),
        "$." => not_impl("$."),
        "$:" => not_impl("$:"),
        ".:" => not_impl(".:"),
        ".." => not_impl(".."),
        "C.!.2" => not_impl("C.!.2"),
        "p:" => not_impl("p:"),
        "s:" => not_impl("s:"),
        "T." => not_impl("T."),
        "u:" => not_impl("u:"),
        "Z:" => not_impl("Z:"),
        "u." => not_impl("u."),
        "v." => not_impl("v."),
        "]:" => adverb("]:", a_not_implemented),
        "f." => adverb("f.", a_not_implemented),
        "M." => adverb("M.", a_not_implemented),
        "." => conj(".", c_not_implemented),
        ":." => conj(":.", c_not_implemented),
        "!." => conj("!.", c_not_implemented),
        "[." => conj("[.", c_not_implemented),
        "]." => conj("].", c_not_implemented),
        "`:" => conj("`:", c_not_implemented),
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
pub fn v_gcd_or(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_matrix_inverse(_y: &JArray) -> Result<JArray> {
pub fn v_matrix_divide(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_root(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_logarithm(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_magnitude(_y: &JArray) -> Result<JArray> {
pub fn v_residue(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_out_of(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_deal_fixed_seed(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_prime_factors(_y: &JArray) -> Result<JArray> {
pub fn v_prime_exponents(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_angle(_y: &JArray) -> Result<JArray> {
pub fn v_polar(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_transpose_dyad(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_stitch(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_copy(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_fetch(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_not_implemented_monad(_y: &JArray) -> Result<JArray> {
pub fn v_not_exist_monad(_y: &JArray) -> Result<JArray> {
pub fn v_not_implemented_dyad(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_self_classify(y: &JArray) -> Result<JArray> {
pub fn v_less(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_nub_sieve(_y: &JArray) -> Result<JArray> {
pub fn v_itemize(_y: &JArray) -> Result<JArray> {
pub fn v_laminate(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_words(y: &JArray) -> Result<JArray> {
pub fn v_sequential_machine(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_base_(_y: &JArray) -> Result<JArray> {
pub fn v_base(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_antibase_(_y: &JArray) -> Result<JArray> {
pub fn v_antibase(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_grade_up(y: &JArray) -> Result<JArray> {
pub fn v_sort_up(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_grade_down(_y: &JArray) -> Result<JArray> {
pub fn v_sort_down(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_catalogue(_y: &JArray) -> Result<JArray> {
pub fn v_from(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_map(_y: &JArray) -> Result<JArray> {
pub fn v_numbers(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_format(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_anagram_index(_y: &JArray) -> Result<JArray> {
pub fn v_anagram(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_cycledirect(_y: &JArray) -> Result<JArray> {
pub fn v_permute(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_raze_in(_y: &JArray) -> Result<JArray> {
pub fn v_member_in(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_index_of(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_member_interval(x: &JArray, y: &JArray) -> Result<JArray> {
pub fn v_steps(_y: &JArray) -> Result<JArray> {
pub fn v_index_of_last(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_interval_index(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_circle_function(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_roots(_y: &JArray) -> Result<JArray> {
pub fn v_polynomial(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_poly_deriv(_y: &JArray) -> Result<JArray> {
pub fn v_poly_integral(_x: &JArray, _y: &JArray) -> Result<JArray> {
pub fn v_extend_precision(_y: &JArray) -> Result<JArray> {
pub fn v_num_denom(x: &JArray, y: &JArray) -> Result<JArray> {
```
