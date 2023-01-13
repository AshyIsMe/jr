queens=: 3 : 0
 z=.i.n,*n=.y
 for. }.z do.
  b=. -. (i.n) e."1 ,. z +"1 _ ((-i.){:$z) */ _1 0 1
  z=. ((+/"1 b)#z),.n|I.,b
 end.
)
