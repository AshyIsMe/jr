NB. This file generates `smoke.toml` if you run `gen-runlist` right.
NB. Or, if you have `just` installed, just `just`.
NB. Then, you can run the smoke test, with `cargo test`.
NB. So, edit this file, run `just`, then run `cargo test`, then come back here and edit out what doensn't work.

NB. literal conversions
0
1
2
0.5
1r2
2j3
2.1j3.1

NB. literal promotions
1 0.5 2j3
1 4j2 3j3 2.00
NB. failing: 1.00 0.00

NB. comparisons
3 = 5
5 = 5
5j5 = 5j5
5.1 = 5.1
5r2 = 2.5
5r3 = 2
5j0 = 5

NB. less than
3 < 5
3.2 < 5.1
5 < 5r2
5 < 37r2
NB. domain error: 5.1 < 6j3

NB. floor
<. 4.6 4 _4 _4.6
3 <. 4 _4
<./7 8 5 9 2

NB. decrement
<: 2 3 5 7
<: 3 2
NB. this outputs floating somehow: <: 3.0 2.0

NB. larger than
1 2 3 4 5  >  5 4 3 2 1

NB. ceil
>. 4.6 4 _4 _4.6
3>.4 _4
>./7 8 5 9 2
NB. not implemented: >./\7 8 5 9 2

NB. increment
>: 2 3 5 7

NB. negative / infinity
_
__
_2 _ 3 + 5
NB. not implemented: 10 ^. 0
NB. not implemented: _:"0 1 2 3 4
NB. parse error? 3.14"0 1 2 3 4

NB. conj / plus
3j4 * 3j_4
j. 0 1 2 3 4
j. i. 5
(i. 5) + 2 * j. (i. 5)
+ (i. 5) + 2 * j. (i. 5)
(0 1j2 2j4 3j6 4j8) * (+(0 1j2 2j4 3j6 4j8))

NB. real
+. 0 1j2 2j4 3j6
NB. incorrect datatype: +. 3

NB. double
+: 3 0 _2
NB. not implemented: (0 1) +:/ (0 1)

NB. signum
NB. incorrect datatype: * _3 0 5 5r2
* _4 0 4
3*5
3.2*5
3.2*5r2
NB. incorrect datatype: 3*3r1
NB. incorrect datatype: _7*3r1
__*3j3
* 0 1
* 1 2

NB. length angle
NB. test framework fails for float comparison: *. 3j4
NB. test framework fails for float comparison: *. 3j4 5r2

NB. square
*: 2 3 4
*: 2.1 4.3
NB. float comparison fails legitimately? *: 3.2

0 *: 0
1 *: 0
0 *: 1
1 *: 1

NB. negate
-2 0 _2
2-2
2-3
NB. incorrect datatype: 2-0.0
2-0.5
1-0

NB. not
-. 0
-. 1
-. 2
-. 3j5 1
-. 3r2 7
NB. test framework bug: -. 0.5 0.7 3j5

NB. halve
NB. incorrect datatype: -: i. 5

NB. reciprocal
NB. incorrect datatype: % 1
% 0.25
% 0
% 6j2

NB. incorrect datatype: 3 % 5
3.2 % 1.6
3r4 % 2
3r4 % 7.5
3r4 % 5r12
NB. incorrect datatype: 0 % 1
NB. incorrect result: 0 % 0
NB. incorrect datatype: 1 % 0
NB. incorrect datatype: 1 % 1

NB. sqrt
%:-4
%:-1
%:4 9 16
NB. incorrect precision: %: 625r100
NB. nonce: %: 5j2