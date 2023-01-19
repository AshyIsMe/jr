pattern =: _10 {."1 (7) {."1 (_10) {. 7 {. '.O'&i.;._2 (0 : 0)
.O
..O
OOO
)

NB. Game Of Life - one step
lifeprog =: 2 10 $ 0 0 0 1 0 0 0 0 0 0 0 0 0 1 1 0 0 0 0 0
life =: (_2 ]\ 1 1 3 3) (lifeprog {~ ([: < (<1 1)&{ , +/@:,));._3 (0)&([ , [ ,~ [ ,. [ ,.~ ])

NB. Run 4 iterations
' *' {~ life^:(<5) pattern
