
NB. adventofcode 2022 day2
s2=:(,;._2~ LF=]) 0 : 0
A Y
B X
C Z
)

NB. A rock B paper C scissors
NB. X rock Y paper Z scissors

wins=:(,;._2~ LF=]) 0 : 0
A Y
B Z
C X
)

draws=: (,;._2~ LF=]) 0 : 0
A X
B Y
C Z
)

scores=: ". (}."1 s2) rplc ;/'X1Y2Z3'
scores =: scores + (6 * s2 e. wins) + (3 * s2 e. draws)

] d2p1=: +/scores

NB. day 2 part 2
NB. X lose, Y draw, Z win
scores=: ". (}."1 s2) rplc ;/'X0Y3Z6'
NB. A1, B2, C3
hands=: ". }: (LF;';') rplc~  0 : 0
'A X';'3'
'A Y';'1'
'A Z';'2'
'B X';'1'
'B Y';'2'
'B Z';'3'
'C X';'2'
'C Y';'3'
'C Z';'1'
)
points =: ". _1 ,\ s2 rplc hands
] d2p2=: +/ scores + points

