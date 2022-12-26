## Conversion

So, you've got some data, and you want to produce a J type.

### Atoms

`JArray` is directly convertable from all the basic types: `JArray::from(5i64)`.


### Homogenous lists

`JArray::from_list(iterable)` can construct a list (`shape() == [iterable.len()]`)
from an iterable of any basic type. This is infallible and cheap.


### Multi-dimensional

For example:

```j
   i. 2 3
0 1 2
3 4 5
```

 * You could write this as `JArray::from_list(0..6).into_shape(&[2, 3])?`,
   i.e. generate the list, then reshape it to `2 3` (which is what `i.` does,
   logically); this is like `2 3 $ i. 6`.

 * You could write the version with promotion, which will take two sub-lists,
   and fill them out to the right shape (which is a no-op in this case), and
   promote them (which is also a no-op); this is like `> (< 0 1 2), (<3 4 5)`:

```rust
JArray::from_promo_fill([
    JArray::from([0, 1, 2]),
    JArray::from([3, 4, 5]),
])
```


### Boxed lists

A boxed list is just a list of regular arrays, and can be built with `from_list`.


### Heterogeneous input lists, non-boxed output lists

You could build parts, and promo-fill them, like `> (<0 1 2),(<3.5 4.2 5.5)`:
```text
  0   1   2
3.5 4.2 5.5
```

```rust
JArray::from_promo_fill([
    JArray::from_list([0,   1,   2]),
    JArray::from_list([3.5, 4.2, 5.5]),
])
```

You could build `Elem`s manually, and `promote_to_array`, which probably isn't a
great idea unless you're already operating on `Elem`s.

`promote_to_array(vec![Elem::Num(Num::from(5i64))])`
