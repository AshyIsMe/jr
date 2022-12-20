NB. Diamond Kata

NB.    dia 'ABCD'
NB. ---A---
NB. --B-B--
NB. -C---C-
NB. D-----D
NB. -C---C-
NB. --B-B--
NB. ---A---

dia =: {{ (] ,"1 }.@|."1) ([ , }.@|.) (>:i. #y) |."0 1 (,. ,"1 (<:@# # '-'&[)) y}}

dia 'ABCD'
