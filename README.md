# jr - j in rust ... eventually

j is amazing: https://www.jsoftware.com/

rust is also great: https://doc.rust-lang.org/stable/book/

How hard could it be to write our own j interpreter in rust?...

* http://craftinginterpreters.com/


## What is jr?

Jr is a rust implementation of the [J programming language](https://www.jsoftware.com/).
It is intended to be feature compatible with the official [jsoftware implementation](https://github.com/jsoftware/jsource) (though not necessarily bug compatible).

It relies on the [ndarray crate](https://docs.rs/ndarray/latest/ndarray/).

Some extremely useful j books and documentation:

* [J Wiki](https://code.jsoftware.com/wiki/Main_Page)
* [An Implementation of J](https://www.jsoftware.com/ioj/ioj.htm)
* [J for C Programmers](https://www.jsoftware.com/help/jforc/contents.htm)
* [Learning J](https://www.jsoftware.com/help/learning/contents.htm)
* [J Primer](https://www.jsoftware.com/help/primer/contents.htm)

Note: [Arthur Whitney](https://aplwiki.com/wiki/Arthur_Whitney) and [Roger Hui](https://aplwiki.com/wiki/Roger_Hui) style C ([Incunabulum](https://code.jsoftware.com/wiki/Essays/Incunabulum)) is _not_ a direct inspiration for the rust code in this project.

## Why

For fun! :D

Also to get better at rust and j.

## What's left to do?

_CURRENT STATUS:_ [STATUS.md](./STATUS.md)

_TODO:_

* Implement the rest of the primitives (see: (STATUS.md) and `src/lib.rs`)
* Tests, lots more tests needed
* [Foreigns](https://code.jsoftware.com/wiki/Vocabulary/Foreigns)
* [Locales](https://code.jsoftware.com/wiki/Vocabulary/Locales) - partial
* Draw the rest of the owl


_Done:_

* Basic scanning and tokenizing
* Basic verb and adverb evaluation
* A few primitive verbs implemented for integer nouns
* Finish `src/lib.rs:eval()`
* Implement the verb rank concept
* [J compatible display](https://www.jsoftware.com/ioj/iojDisp.htm) of nouns


## usage

``` sh
cargo build --release

./target/release/jr
   1 2 3 + 4 5 6
[5, 7, 9]
```

Tests:

``` sh
cargo test

# quick run
cargo run
```
