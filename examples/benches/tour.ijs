NB. knight moves for each square of a (y,y) board
kmoves=: 3 : 0
 t=. (>,{;~i.y) +"1/ _2]\2 1 2 _1 1 2 1 _2 _1 2 _1 _2 _2 1 _2 _1
 (*./"1 t e. i.y) <@#"1 y#.t
)

ktour=: 3 : 0
 if. 1>:y do. i.,~y return. end.
 m=. kmoves y
 p=. *:y
 stack=. ,&.>|.y (<:/~@] #&, * +/ ]) i.>.-:y
 while. #stack do.
  s=. >{:stack
  if. a: e. (((i.p)-.s){m)-.&.><s do.
   if. (#s)=p-1 do. (,~y)$/:s,(i.p)-.s return. end.
   stack=. }:stack continue.
  end.
  stack=. (}:stack),(<s),&.>s-.~({:s){::m
 end.
)
