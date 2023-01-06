NB. This file generates `smoke.toml` if you run `gen-runlist` right.
NB. Or, if you have `just` installed, just `just`.
NB. Then, you can run the smoke test, with `cargo test`.
NB. So, edit this file, run `just`, then run `cargo test`, then come back here and edit out what doesn't work.

NB. literal conversions
0
1
0 0
0 1
0 1 0
0 1 0 1
0 1 0 1 0
0 1 0 1 0 1
0 1 0 1 0 1 0
0 1 0 1 0 1 0 1
0 1 0 1 0 1 0 1 0
0 1 0 1 0 1 0 1 0 1
0 1 0 1 0 1 0 1 0 1 0
0 1 0 1 0 1 0 1 0 1 0 1
0 1 0 1 0 1 0 1 0 1 0 1 0
0 1 0 1 0 1 0 1 0 1 0 1 0 1
0 1 0 1 0 1 0 1 0 1 0 1 0 1 0
2
0.5
1r2
2j3
2.1j3.1
0x
1x
2x
__j__
_j_
_3j_3.3
3e1
0x 1x
<1
<2
0 $ 0
(0$0) $ 17
0: 5
1: 5
2: 5
9: 5
5 2: 5
5 6 2: 5
_0: 5
_1: 5
_2: 5
_9: 5
5 _2: 5
5 6 _2: 5
_: 5
5 _: 5
_ _: _
16bff
16b0e
16b16
16b0
16b1
16b12
16b123
16bffffffff
3b012

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

NB. less than or equal
1 <: 3
1 2 3 <: 4 5 6
4 5 6 <: 1 2 3

NB. floor
<. 4.6 4 _4 _4.6
3 <. 4 _4
<./7 8 5 9 2

NB. decrement
<: 2 3 5 7
<: 3 2
NB. this outputs floating somehow: <: 3.0 2.0

NB. box
><0
><0 1
><0 1 2
><i. 2 3

NB. larger than
1 2 3 4 5  >  5 4 3 2 1

NB. larger than or equal
2 >: 1
4 5 6 >: 1 2 3
1 2 3 >: 4 5 6

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

NB. copy
2 4 # (2 3 $ 7 8 9 4 5 6)
2 # 5 6 7
2 3 # 2
2 4 # 7j7

NB. length angle
NB. test framework fails for float comparison: *. 3j4
NB. test framework fails for float comparison: *. 3j4 5r2

NB. lcm / and
3 *. 1
1 *. 0
0 *. 1
0 *. 0
1 *. 1
64 *. 2

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

NB. less
'encephalogram' -. 'e'
'encephalogram' -. 'a'
'encephalogram' -. 'aeiouAEIOU'
NB. wrong, needs to understand agreement (sigh): (i. 3 3) -. 3 4 5

NB. halve
NB. incorrect datatype: -: i. 5

NB. match
'no' -: |. 'on'
'no' -: 'yes'

NB. reciprocal
NB. incorrect datatype: % 1
% 0.25
% 0
NB. test framework can't cope with float maths: % 0.2j0.4

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

NB. exponential / power
3^2
3^3
9^0.5
32^1r5

NB. reflexive / passive
^~ 3
3.2 %~ 16
16 %~ 3.2

NB. grade
/: 3 1 4 2 1 3 3

NB. sort
'abcd' /: 4 2 3 1
7 8 9 10 /: 4 2 3 1
7 8 9 10 /: 4 2 3
7 8 9 10 /: 4 2
\:~ 'abecedarian'
NB. nonce: laminate: \:~"1 'dozen',:'disk'

NB. self classify
= i. 3
= 5 4 3 4 5
= 3 3 $ i. 6
= 1
= 'do what you want because a pirate is free, yar har diddledee dee'

NB. prefix
]\ 'banana'
+/\ 1 2 3 4 5 6

NB. infix
_2 [\ ('aba';'XYZT';'ba';'+')
_3 <\ 'abcdefg'
 3 <\ 'abcdefgh'

NB. suffix / outfix
<\. (1 2 3 4 5)
(4&{.)\. 'potato'


NB. same / left / right
] i. 2 3
[ 'abcde'
(i. 2 3) [ 'abcde'
(i. 2 3) ] 'abcde'

NB. take
0 {. 2
1 {. 2
2 {. 2
2 {. 'abcde'
_2 {. 'abcde'
6 {. 3 1 4 1 5 9 6 2 3 5
6 {. 3 1 4
3 {. 5 }. 'abcdefghijklmnopqrstuvwxyz'
2 ({. ,&< }.) 'abcde'
> (5;(0 $ 0))
> ((i. 0 4); (i. 2 2))

NB. fetch
1 {:: 'foo'; 'bar'
0 {:: <5
0 {:: ((<5);(<<4))
1 {:: ((<5);(<<4))
(2 1 $ ((<5);(<<4)))
0 {:: 5
0 {:: (2 1 $ ((<5);(<<4)))
;/"1 i.4 4
(<0 0) {:: ;/"1 i.4 4
(<0 2) {:: ;/"1 i.4 4
(<1 2) {:: ;/"1 i.4 4
(<2 0) {:: ;/"1 i.4 4

NB. rank
(5"_) 0
(5"_) (0 $ 0)
(5"_) i. 2 3
(''"_) 'hello'
(''"_) 5 6

NB. amend
'x' 0 3} 'cross'
'gw' 0 3} 'cross'

NB. amend (legacy definition: curlyrtu)
2 (>./)} 0 1 2

NB. cap
2 (>. % [: <. + * -) 2

NB. reverse
|. i.6
|. i. 2 3

NB. rotate shift
2 |. i.6
_1 |. i.6
0 |. i. 6
2 |. i. 2 3
_1 |. i. 2 3

NB. append
5,3
5,3 6
2 5,3
'abc','d'
(i. 2 3),(10 + i. 2 3)
(i. 2 3),(10 + i. 3 3)
5,(0 $ 0)
(0 $ 0),6
5,(0 $ 0),6

NB. ravel items
,. 'a'
,. i.3

NB. link
5;3
6;7;8
0 2 ; 4 2 5 7
'good' ; 'morning'
'alpha' ; 'bravo' ; 'charlie'
'Gauss';100
'Fred';30;40
5 ; 12 ; 1995
2 2 $ 1;2;3;4
'abc' ; 1 2 3 ; (i. 2 2)
(<'abc');(<'def');(<'ghi')
(<'abc');(<'def');<(<'ghi')

NB. words
;: '<;._1'
;: 'Fine, easy as 1 2 3?'

NB. raze
; <1
; 1 2 3; 4 5 6; 7 8 9
; (1 $ < (1 $ 1))

NB. cut
<;._2 (0 1 2 0 0 1 0)
<;._2 (1 2 0 0 1 0)
$;._2 (1 2 0 0 1 0)
#;._2 (1 2 0 0 1 0)
><;._2 (1 2 0 0 1 0)
1 1 0 0 0 0 1 ,;.2 i. 7
1 1 0 0 1 0 0 ,;.2 i. 7
0 0 1 0 1 1 0 ,;.2 i. 7
0 0 1 0 1 1 0 ,;._2 i. 7
<;.1 'a man, a plan, a canal: panama'
<;.1 ] 3 1 4 1 5 9 2 6 5 3 5 3 2 3 8
+/;.1 (3 1 4 1 5 9 2 6 5 3 5 3 2 3 8)
<;.2 'a man, a plan, a canal: panama'
<;._1 'a man, a plan, a canal: panama'
<;._2 'a man, a plan, a canal: panama'

NB. shape
2 0 $ <''

NB. nub
~. (3 3 $ 1 2 3 1 2 3 4 5 6)
~. 2 3 4 3 2 5 4 1

NB. transpose
|: (0 $ 0)
|: 2 3
|: i. 2 4

NB. oblique / key
1 2 3 1 3 2 1 </. 'abcdefg'
1 2 3 1 3 2 1 #/. 'abcdefg'

NB. numbers
0". 4 1 $ '1001'
0". 2 2 $ '0101'
0". 2 3 $ '1.5101'
0 ". 'addx 15 '
NB. we compute an atom 'cos we just guessed the reshape: 0". 1 4 $ '1001'

NB. do
". '1000 2000 3000;4000;5000 6000;7000 8000 9000;10000'
NB. no direct: ". '{{ x }}'
NB. we still don't understand gerunds: ". '+`*'

NB. format
": 5
": 17.7

NB. tie
*`+
*`0:
+&>`0:
+&.>`0:
(}. , {.)`0:
(0 ~: 5&(+&>))`0:
NB. not serialising bodies: {{ x + y }}`0:

NB. agenda
5 (+`-@.1) 2
5 (+`-@.0) 2
*`0:@.1 ''

NB. atop / at
#@> 'Newton';'Einstein'
2 3 <@, 4 5
(1 2 3 4) */ @  + (7 5 3 2)
(1 2 3 4) */ @: + (7 5 3 2)

NB. cor
0 : 'hello'
(3 (4 : 'x + y') 5)

NB. bondo
1&+ 5
(+&1) 5
*:&+: 3 4 5
+/&+: 3 4 5
!5
3 +&! 4
NB. bizarro fill / box: 'Dennis';'Richard';'Ken' ,&> 'Ritchie';'Stallman';'Iverson'
'Dennis';'Richard';'Ken' (>@[ , >@])"0 'Ritchie';'Stallman';'Iverson'

NB. ampdot
NB. floats: 3 +&.^. 4
i.&.> (1;2;2 3)
#&.> ('foo'; 'ba')
#&.> 1 {. ('foo'; 'ba')
#&.> <'foo'
# (&.>) < 'foo'
(3;5) (+&.>) (<7)

NB. integers
i. 0
i. 1
i. 2
i. _1
i. _2
i. 2 3
i. 2 _3
i. _2 3
i. _2 _3

NB. index of
'ABCXYZ' i. (3 4 $ 'AYBXCZQAYBCA')
'ABCXYZ' i."_ 0 (3 2 $ 'AYBXCZ')

NB. indexes (bool)
I. 0 0 1 0 1 0

NB. member interval
'c' E. 'cocoa'
'co' E. 'cocoa'

NB. NB.
NB. NB. at the start of a line is handled by the test framework today.
5 NB. 6
5 NB. it's a boy!

NB. controls
3 {{ x + y }} 5

NB. foreign
3!:3 ] 2
3!:3 ] 2 4 7 _2
2 (3!:4) 97

NB. name list
{{ 4!:1 [ 0 1 2 3 [ a =. 4 5 6 [ b =. 7 8 9 }} ''
NB. jsoft bug, apparently nothing starts with an empty string: {{ '' 4!:1 [ 0 [ abc =. 4 5 6 [ def =. 7 8 9 }} ''
{{ 'a' 4!:1 [ 0 [ abc =. 4 5 6 [ def =. 7 8 9 }} ''
{{ 'ab' 4!:1 [ 0 [ abc =. 4 5 6 [ def =. 7 8 9 }} ''

NB. name erase
4!:55 <'a'
4!:55 'a';'b'

NB. platform information
NB. platform dependent actual result
$ 9!:12 ''

NB. agreement / empty
{{ 3 4 5 }}"1 (0 5 $ 0)
{."1 (0 1 $ 0)

NB. torture
#/.~@/:~'AGCTTTTCATTCTGACTGCAACGGGCAATATGTCTCTGTGTGGATTAAAAAAAGAGTGTCTGATAGCAGC'

NB. AoC
><;._2 (0".><;._2 ('1000',' ','2000',' ','3000',' ',' ','4000',' ',' ','5000',' ','6000',' ',' ','7000',' ','8000',' ','9000',' ',' ','10000',' ',' '))
>./ +/ "1 >        ". '1000 2000 3000;4000;5000 6000;7000 8000 9000;10000'
>./ +/ &>          ". '1000 2000 3000;4000;5000 6000;7000 8000 9000;10000'
+/ 3 {. \:~ +/ &>  ". '1000 2000 3000;4000;5000 6000;7000 8000 9000;10000'
6 }. 'helicopter'
6 {. 'helicopter'
(<'helico'),(<'pter')
'helico' ,&< 'pter'
6 ({. ,&< }.) 'helicopter'
'cat' e. 'abcd'
('aba'; 'ba') I. @ E. &.> <'ababa'

;/i.5
0;1;2;3;4
NB. (;/i.5) = 0;1;2;3;4
0;1
(,. ,"1 (<:@# # '-'&[)) 'ABCD'
(>:i. #'ABCD') |."0 1 (,. ,"1 (<:@# # '-'&[)) 'ABCD'

NB. empty should be fine
}. 0$0
{. 0$0
(0$0) }. 1 2 3
NB. weird special case in from, who knows
(0$0) { 1 2 3

NB. power conjunction
>:^:1 2 4 (5)
2 *^:< 1 2 3
2 *^:< i.2 2

