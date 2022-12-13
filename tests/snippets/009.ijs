s =: 0 : 0
nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg
)

parse =: <
general =: [: ; {{ x + 1 i.~ (x = [: #@:~. x {. ])\. y }} &.>
a =: 4 general ]
b =: 14 general ]
(a parse s); (b parse s)
