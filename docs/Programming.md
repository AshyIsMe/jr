Programming in J
================

J translation of John Earnest's Programming in K guide: https://github.com/JohnEarnest/ok/blob/gh-pages/docs/Programming.md

Filtering
---------

Use (copy)[https://code.jsoftware.com/wiki/Vocabulary/number#dyadic] (`#`) with the predicate array on the left:

       y=:i.10
	   P=: {{0<3|y}}    NB. not multiples of 3
	   (P y)#y  NB. copy 1 of each match
    1 2 4 5 7 8

       (0<3|y)#y    NB. without the direct definition

Zipping
-------

You have two lists and want to map a verb over pairings of their elements. Use (table)[https://code.jsoftware.com/wiki/Vocabulary/slash#dyadic] `/` and rank:

       'abc' ,/"0 'ABC'
    aA
    bB
    cC

       1 2 3 (+/"0) 4 5 6
    5 7 9
    
Although there's better ways for both of those particular examples:

       'abc' ,. 'ABC'
    aA
    bB
    cC
    
       1 2 3 + 4 5 6
    5 7 9


Construction from indexing directly:

       a. {~ (65+i.26)+32* 26$ 0 1
    AbCdEfGhIjKlMnOpQrStUvWxYz


Conditionals and Alternatives
-----------------------------

Cartesian Product
-----------------
 
Iterative Algorithms
--------------------

Sequential Processing
---------------------
Making The Grade
----------------

Encode and Decode
-----------------
