NB. should be pulled in from stdlib.ijs
split =: ({. ,&< }.)
LF =: 10{a.
s3 =: 0 : 0
vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw
)
s3 =: (LF=s3) < ;._2 s3

rucksacks =: (-:@# split ]) &> s3
dups =: ([: ([: ~. e./ # {.) [: > -:@# split ]) &> s3
alphabet =: a. {~ (96+i.27),(65+i.26)

d3p1=: +/ alphabet i. dups

NB. day 3 part 2

groups =: _3 ]\ s3

badge =: {{
's1 s2 s3'=. y
~. s1 #~ (s1 e. s2) *. (s1 e. s3)
}}

badges =: badge"1 groups

d3p2 =: +/ alphabet i. badges

d3p1; d3p2
