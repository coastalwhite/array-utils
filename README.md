# array-utils

<!-- cargo-sync-readme start -->

A no_std heapless set of utilities for handling const generic arrays. Sized arrays sacrifice
speed and convenience for simplicity and predictability. This is mostly used at very low levels
and/or on platforms without heaps. This crate will greatly improve the readability and
maintainability of code on those platforms.

# Features

This crate provides functions to `initialize`, `drift`, `slice`, `resize`, `splice`, `join` and
`superimpose` sized arrays. All of which are _features_ enabled by default, but can therefore
also be used separately. Let us go all of the _features_ one by one.

This crate only contains functions which should never panic. Every invalid value given will
either result in data truncating or an array being fill up with extra data. The bounds,
specific behaviors and other important details are documented in each function's page.

## Initialize

This feature provides 4 utility functions. These are
[`initialize_from`](https://docs.rs/array-utils/latest/array-utils/fn.initialize_from.html), [`initialize_till`](https://docs.rs/array-utils/latest/array-utils/fn.initialize_till.html),
[`initialize_from_option`](https://docs.rs/array-utils/latest/array-utils/fn.initialize_from_option.html) and
[`initialize_from_result`](https://docs.rs/array-utils/latest/array-utils/fn.initialize_from_result.html). All these functions provide an
simpler ways to initialize sized array using closures, as can be seen in their documentation.

## Drift / Superimpose

The 2 drifting functions, which are [`drift_to_begin`](https://docs.rs/array-utils/latest/array-utils/fn.drift_to_begin.html) and [`drift_to_end`](https://docs.rs/array-utils/latest/array-utils/fn.drift_to_end.html), provide a way to
float a part of the sized array to the beginning or the end of the array. This provides a
solution to the common problem of prefixing or suffixing some elements to an array. It can
be seen as a more optimized shortcut for the the combination of
[`sized_slice`](https://docs.rs/array-utils/latest/array-utils/fn.sized_slice.html) and
[`superimpose`](https://docs.rs/array-utils/latest/array-utils/fn.superimpose.html). There is also the more general form of [`superimpose`](https://docs.rs/array-utils/latest/array-utils/fn.superimpose.html), which allows for one
sized array to be superimposed upon another.

## Slice / Resize

Ordinary slices of sized array have the disadvantage of either losing size metadata or needing
a `.try_into().unwrap()` appended, which can panic. The [`sized_slice`](https://docs.rs/array-utils/latest/array-utils/fn.sized_slice.html) utility function
provides a way to deal with slicing into sized arrays which never panics. In a similar way to
slicing dealing scaling arrays is rather cumbersome. [`array_resize`](https://docs.rs/array-utils/latest/array-utils/fn.array_resize.html) provide a simple way to
deal with all the truncating or expanding of data without the possibility for panics.


## Splice / Join

The [`splice`](https://docs.rs/array-utils/latest/array-utils/fn.splice.html) and [`join`](https://docs.rs/array-utils/latest/array-utils/fn.join.html) utilities are basically more optimized combinations of
[`sized_slice`](https://docs.rs/array-utils/latest/array-utils/fn.sized_slice.html) and [`superimpose`](https://docs.rs/array-utils/latest/array-utils/fn.superimpose.html). Making splicing and joining arrays at specific indices can
be very handy for dealing with packet and data streams.

# Usage

Since we are using sized arrays, all utilities heavily rely on const generics. Furthermore, all
functions are only implemented for types with the [`Copy`](https://doc.rust-lang.org/stable/core/marker/macro.Copy.html) trait. Some
utilities, namely the functions without additional `fill` parameter, also depend on the
[`Default`](https://doc.rust-lang.org/stable/core/default/macro.Default.html) trait.

Here are some examples or the usage of this crate.

## Initializing

```rust
use array_utils::initialize_from;

// Use a closure which doubles the index
let even_numbers: [usize; 5] = initialize_from(|index| index * 2);
assert_eq!(even_numbers, [0, 2, 4, 6, 8]);
```

## Drifting

```rust
use array_utils::{ drift_to_begin, drift_to_end };

let array = [1, 2, 3, 0, 0, 0, 0];
// Float the elements with indices `0..` to the beginning with a margin of `1` elements,
// filling in `0x00` for all new elements.
assert_eq!(drift_to_begin(array, 0, 1, 0x00), [0, 1, 2, 3, 0, 0, 0]);

// Float the elements with indices `..3` to the end with a margin of `0` elements,
// filling in `42` for all new elements.
assert_eq!(drift_to_end(array, 3, 0, 42), [42, 42, 42, 42, 1, 2, 3]);
```

<!-- cargo-sync-readme end -->

## License

Licensed with a MIT license.
