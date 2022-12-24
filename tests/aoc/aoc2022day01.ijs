
NB. Relies on stdlib.ijs from official j.
NB. TODO: resolve tilde to $HOME
NB. (0!:10) <'~/j903/system/main/stdlib.ijs'
(0!:10) <'/Users/aaron/j903/system/main/stdlib.ijs'
rplc=:stringreplace~

NB. day1  ---------------------------------------------------
s1 =: 0 : 0
1000
2000
3000

4000

5000
6000

7000
8000
9000

10000
)

] d1p1=: >./ +/ &> ". }: s1 rplc (LF,LF);';';LF;' '
] d1p2=: +/ 3 {. \:~ +/ &> ". }: s1 rplc (LF,LF);';';LF;' '
