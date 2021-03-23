//! A no_std heapless set of utilities for handling const generic arrays. Sized arrays sacrifice
//! speed and convenience for simplicity and predictability. This is mostly used at very low levels
//! and/or on platforms without heaps. This crate will greatly improve the readability and
//! maintainability of code on those platforms.
//!
//! # Features
//!
//! This crate provides functions to `initialize`, `drift`, `slice`, `resize`, `splice`, `join` and
//! `superimpose` sized arrays. All of which are _features_ enabled by default, but can therefore
//! also be used separately. Let us go all of the _features_ one by one.
//!
//! This crate only contains functions which should never panic. Every invalid value given will
//! either result in data truncating or an array being fill up with extra data. The bounds,
//! specific behaviors and other important details are documented in each function's page.
//!
//! ## Initialize
//!
//! This feature provides 4 utility functions. These are
//! [`initialize_from`](crate::initialize_from), [`initialize_till`](crate::initialize_till),
//! [`initialize_from_option`](crate::initialize_from_option) and
//! [`initialize_from_result`](crate::initialize_from_result). All these functions provide an
//! simpler ways to initialize sized array using closures, as can be seen in their documentation.
//!
//! ## Drift / Superimpose
//!
//! The 2 drifting functions, which are [`drift_to_begin`](crate::drift_to_begin) and [`drift_to_end`](crate::drift_to_end), provide a way to
//! float a part of the sized array to the beginning or the end of the array. This provides a
//! solution to the common problem of prefixing or suffixing some elements to an array. It can
//! be seen as a more optimized shortcut for the the combination of
//! [`sized_slice`](crate::sized_slice) and
//! [`superimpose`](crate::superimpose). There is also the more general form of [`superimpose`](crate::superimpose), which allows for one
//! sized array to be superimposed upon another.
//!
//! ## Slice / Resize
//!
//! Ordinary slices of sized array have the disadvantage of either losing size metadata or needing
//! a `.try_into().unwrap()` appended, which can panic. The [`sized_slice`](crate::sized_slice) utility function
//! provides a way to deal with slicing into sized arrays which never panics. In a similar way to
//! slicing dealing scaling arrays is rather cumbersome. [`array_resize`](crate::array_resize) provide a simple way to
//! deal with all the truncating or expanding of data without the possibility for panics.
//!
//!
//! ## Splice / Join
//!
//! The [`splice`](crate::splice) and [`join`](crate::join) utilities are basically more optimized combinations of
//! [`sized_slice`](crate::sized_slice) and [`superimpose`](crate::superimpose). Making splicing and joining arrays at specific indices can
//! be very handy for dealing with packet and data streams.
//!
//! # Usage
//!
//! Since we are using sized arrays, all utilities heavily rely on const generics. Furthermore, all
//! functions are only implemented for types with the [`Copy`](::core::marker::Copy) trait. Some
//! utilities, namely the functions without additional `fill` parameter, also depend on the
//! [`Default`](::core::default::Default) trait.
//!
//! Here are some examples or the usage of this crate.
//!
//! ## Initializing
//!
//! ```
//! use array_utils::initialize_from;
//!
//! // Use a closure which doubles the index
//! let even_numbers: [usize; 5] = initialize_from(|index| index * 2);
//! assert_eq!(even_numbers, [0, 2, 4, 6, 8]);
//! ```
//!
//! ## Drifting
//!
//! ```
//! use array_utils::{ drift_to_begin, drift_to_end };
//!
//! let array = [1, 2, 3, 0, 0, 0, 0];
//! // Float the elements with indices `0..` to the beginning with a margin of `1` elements,
//! // filling in `0x00` for all new elements.
//! assert_eq!(drift_to_begin(array, 0, 1, 0x00), [0, 1, 2, 3, 0, 0, 0]);
//!
//! // Float the elements with indices `..3` to the end with a margin of `0` elements,
//! // filling in `42` for all new elements.
//! assert_eq!(drift_to_end(array, 3, 0, 42), [42, 42, 42, 42, 1, 2, 3]);
//! ```

#![no_std]
#![warn(missing_docs)]

const fn min_of_sizes(x: usize, y: usize) -> usize {
    if x < y {
        x
    } else {
        y
    }
}

/// Initialize a sized array from a closure taking the index and outputting the elements.
///
/// Generates a new sized array generated from generator closure, which turns a index into a
/// element of the generated array.
///
/// # Examples
///
/// ```
/// use array_utils::initialize_from;
///
/// // Size is most of the time automatically inferred
/// assert_eq!(initialize_from(|index| index), [0, 1, 2, 3, 4, 5]);
/// assert_eq!(initialize_from(|index| 2 * index), [0, 2, 4, 6, 8, 10]);
///
/// fn get_prime(index: usize) -> usize {
///     // ...snip
///     # [2, 3, 5, 7, 9, 11][index]
/// }
///
/// assert_eq!(initialize_from(get_prime), [2, 3, 5, 7, 9, 11]);
/// ```
///
/// # Panics
///
/// Only panics when the given closure `f` panics.
#[cfg(feature = "initialize")]
pub fn initialize_from<T, F, const OUTPUT_SIZE: usize>(f: F) -> [T; OUTPUT_SIZE]
where
    T: Copy + Default,
    F: Fn(usize) -> T,
{
    let mut buffer = [T::default(); OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        buffer[i] = f(i);
    }
    buffer
}

/// Initialize a sized array from a closure till a certain value appears.
///
/// Generates a new sized array generated from generator closure, which turns a index into a
/// element of the generated array. If the given `till` value is found, the rest of the output
/// array is filled with the `fill` value. Along with the generated array, this utility returns
/// at what index the given `till` value was found (`OUTPUT_SIZE` if not found).
///
/// # Examples
///
/// ```
/// use array_utils::initialize_till;
///
/// let till_five: ([usize; 8], usize) = initialize_till(
///     |index| index,  // Generator closure
///     5,              // Till this value
///     42              // Fill the rest with
/// );
/// assert_eq!(till_five, ([0, 1, 2, 3, 4, 42, 42, 42], 5));
///
/// // Especially useful for null terminated data streams.
/// # static mut COUNTER: usize = 0;
/// fn get_stream_byte() -> u8 {
///     # unsafe {COUNTER += 1;
///     # [0, 4, 2, 1, 3, 3, 7, 0][COUNTER]}
///     // ...snip
/// }
///
/// // Fetch stream bytes till it find a `0` byte. Fill the rest with `0` bytes.
/// assert_eq!(initialize_till(|_| get_stream_byte(), 0, 0), ([4, 2, 1, 3, 3, 7, 0, 0, 0], 6));
/// ```
///
/// # Panics
///
/// Only panics if the given `f` panics.
#[cfg(feature = "initialize")]
pub fn initialize_till<T, F, const OUTPUT_SIZE: usize>(
    f: F,
    till: T,
    fill: T,
) -> ([T; OUTPUT_SIZE], usize)
where
    T: Copy + PartialEq,
    F: Fn(usize) -> T,
{
    let mut buffer = [fill; OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        let value = f(i);
        if value == till {
            return (buffer, i);
        }

        buffer[i] = value;
    }
    (buffer, OUTPUT_SIZE)
}

/// Initialize a sized array from a closure taking the index and outputting an
/// [`Option`](::core::option::Option) of a element, stopping when the first
/// [`None`](::core::option::Option) is encountered.
///
/// Generates a new sized array generated from generator closure, which turns a index into a
/// [`Option<T>`](::core::option::Option) with `T` being elements of the generated array. If a
/// [`None`] value is found, the rest of the output array is filled with the `fill` value. Along
/// with the generated array, this utility returns at what index the given
/// [`None`](::core::option::Option) value was found
/// (`OUTPUT_SIZE` if not found).
///
/// # Examples
///
/// ```
/// use array_utils::initialize_from_option;
/// assert_eq!(
///     initialize_from_option(|index| if index == 5 { None } else { Some(index) }, 42),
///     ([0, 1, 2, 3, 4, 42, 42, 42], 5)
/// );
/// ```
///
/// # Panics
///
/// Only panics if the given `f` panics.
#[cfg(feature = "initialize")]
pub fn initialize_from_option<T, F, const OUTPUT_SIZE: usize>(
    f: F,
    fill: T,
) -> ([T; OUTPUT_SIZE], usize)
where
    T: Copy,
    F: Fn(usize) -> Option<T>,
{
    let mut buffer = [fill; OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        match f(i) {
            None => return (buffer, i),
            Some(value) => buffer[i] = value,
        }
    }
    (buffer, OUTPUT_SIZE)
}

/// Initialize a sized array from a closure taking the index and outputting an
/// [`Result`](::core::result::Result) of a element, stopping when the first
/// [`Err`](::core::result::Result) is encountered.
///
/// Generates a new sized array generated from generator closure, which turns a index into a
/// [`Result<T, E>`](::core::result::Result) with `T` being elements of the generated array. If a
/// [`Err`] value is found, the rest of the output array is filled with the `fill` value. Along
/// with the generated array, this utility returns at what index the given
/// [`Err`](::core::result::Result) value was found
/// (`OUTPUT_SIZE` if not found).
///
/// # Examples
///
/// ```
/// use array_utils::initialize_from_result;
/// assert_eq!(
///     initialize_from_result(|index| if index == 5 { Err(()) } else { Ok(index) }, 42),
///     ([0, 1, 2, 3, 4, 42, 42, 42], 5)
/// );
/// ```
///
/// # Panics
///
/// Only panics if the given `f` panics.
#[cfg(feature = "initialize")]
pub fn initialize_from_result<T, F, E, const OUTPUT_SIZE: usize>(
    f: F,
    fill: T,
) -> ([T; OUTPUT_SIZE], usize)
where
    T: Copy,
    F: Fn(usize) -> Result<T, E>,
{
    let mut buffer = [fill; OUTPUT_SIZE];
    for i in 0..OUTPUT_SIZE {
        match f(i) {
            Err(_) => return (buffer, i),
            Ok(value) => buffer[i] = value,
        }
    }
    (buffer, OUTPUT_SIZE)
}

/// Create an array containing a slice of original array at the end of the array.
///
/// Floats a part of sized `array` with the range `..till` to the end of the result array
/// with `margin` elements after the slice. All elements (including the margin) not filled with
/// the slice will be filled with the `fill` value.
///
/// # Examples
///
/// ```
/// use array_utils::drift_to_end;
///
/// // Float the elements with indices `..3` to the end with a margin of `0` elements,
/// // filling in `42` for all new elements.
/// assert_eq!(drift_to_end([1, 2, 3, 0, 0, 0, 0], 3, 0, 42), [42, 42, 42, 42, 1, 2, 3]);
/// ```
///
/// # Notes
///
/// * If `till` is equal to `0` the resulting buffer will be `[fill; SIZE]`.
/// * If `margin` is greater or equal to `SIZE` the resulting buffer will be `[fill; SIZE]`.
#[cfg(feature = "drift")]
pub fn drift_to_end<T, const SIZE: usize>(
    array: [T; SIZE],
    till: usize,
    margin: usize,
    fill: T,
) -> [T; SIZE]
where
    T: Copy,
{
    let mut buffer = [fill; SIZE];
    for i in 0..till {
        buffer[SIZE - margin - till + i] = array[i];
    }
    buffer
}

/// Create an array containing a slice of original array at the beginning of the array.
///
/// Floats a part of sized `array` with the range `from..` to the beginning of the result array
/// with `margin` elements before the slice. All elements (including the margin) not filled with
/// the slice will be filled with the `fill` value.
///
/// # Examples
///
/// ```
/// use array_utils::drift_to_begin;
///
/// // Float the elements with indices `0..` to the beginning with a margin of `1` elements,
/// // filling in `0x00` for all new elements.
/// assert_eq!(drift_to_begin([1, 2, 3, 0, 0, 0, 0], 0, 1, 0x00), [0, 1, 2, 3, 0, 0, 0]);
/// ```
///
/// # Notes
///
/// * If `till` is equal to `0` the resulting buffer will be `[fill; SIZE]`.
/// * If `margin` is greater or equal to `SIZE` the resulting buffer will be `[fill; SIZE]`.
#[cfg(feature = "drift")]
pub fn drift_to_begin<T, const SIZE: usize>(
    array: [T; SIZE],
    from: usize,
    margin: usize,
    fill: T,
) -> [T; SIZE]
where
    T: Copy,
{
    let mut buffer = [fill; SIZE];
    for i in from..SIZE {
        if margin + i - from >= SIZE {
            break;
        }

        buffer[margin + i - from] = array[i];
    }
    buffer
}

/// Resize a sized array to a different size.
///
/// Copy over the element from `array` into the resulting array. Truncating the original array or
/// filling unfilled elements the `fill` value.
///
/// # Examples
///
/// ```
/// use array_utils::array_resize;
///
/// // Truncating unnecessary values
/// assert_eq!(array_resize([1, 2, 3], 0), [1, 2]);
///
/// // Inserting the `fill` value
/// assert_eq!(array_resize([1, 2, 3], 0), [1, 2, 3, 0]);
/// ```
#[cfg(feature = "resize")]
pub fn array_resize<T, const INPUT_SIZE: usize, const OUTPUT_SIZE: usize>(
    array: [T; INPUT_SIZE],
    fill: T,
) -> [T; OUTPUT_SIZE]
where
    T: Copy,
{
    let mut buffer = [fill; OUTPUT_SIZE];
    for i in 0..min_of_sizes(INPUT_SIZE, OUTPUT_SIZE) {
        buffer[i] = array[i];
    }
    buffer
}

/// Superimpose an sized `sub_array` upon another `main_array` at index `starting_from`.
///
/// Create a copy of the `main_array` and insert all elements of `sub_array` into it,
/// starting from the `starting_from` index. If the `sub_array` has more elements than fit in the
/// `main_array` they are ignored.
///
/// # Examples
///
/// ```
/// use array_utils::superimpose;
///
/// assert_eq!(
///     superimpose([0; 8], [1, 3, 3, 7], 2),
///     [0, 0, 1, 3, 3, 7, 0, 0]
/// );
///
/// // Elements that don't fit in the main array size are ignored.
/// assert_eq!(
///     superimpose([1, 2, 3], [4, 2], 2),
///     [1, 2, 4]
/// );
/// ```
#[cfg(feature = "superimpose")]
pub fn superimpose<T, const MAIN_SIZE: usize, const SUB_SIZE: usize>(
    mut main_array: [T; MAIN_SIZE],
    sub_array: [T; SUB_SIZE],
    starting_from: usize,
) -> [T; MAIN_SIZE]
where
    T: Copy,
{
    for i in starting_from..min_of_sizes(starting_from + SUB_SIZE, MAIN_SIZE) {
        main_array[i] = sub_array[i - starting_from];
    }
    main_array
}

/// Join two sized arrays together into a new array.
///
/// Create a sized array which contain all the elements of `left` and `right` back to back. If
/// there are any elements left to fill, they are filled up with the `fill` value. Any values of
/// `left` or `right` that don't fit in the given buffer are ignored.
///
/// # Examples
///
/// ```
/// use array_utils::join;
///
/// assert_eq!(join([1, 2, 3], [4, 5, 6], 0), [1, 2, 3, 4, 5, 6]);
///
/// // Leftover elements are filled up
/// assert_eq!(join([1, 2, 3], [4, 5, 6], 0), [1, 2, 3, 4, 5, 6, 0, 0]);
///
/// // The input arrays are truncated if the resulting array is too short.
/// assert_eq!(join([1, 2, 3], [4, 5, 6], 0), [1, 2, 3, 4, 5]);
/// ```
#[cfg(feature = "join")]
pub fn join<T, const LEFT_SIZE: usize, const RIGHT_SIZE: usize, const RESULT_SIZE: usize>(
    left: [T; LEFT_SIZE],
    right: [T; RIGHT_SIZE],
    fill: T,
) -> [T; RESULT_SIZE]
where
    T: Copy,
{
    let mut buffer = [fill; RESULT_SIZE];

    for i in 0..min_of_sizes(LEFT_SIZE, RESULT_SIZE) {
        buffer[i] = left[i];
    }

    for i in LEFT_SIZE..min_of_sizes(LEFT_SIZE + RIGHT_SIZE, RESULT_SIZE) {
        if i - LEFT_SIZE >= RIGHT_SIZE {
            break;
        }

        buffer[i] = right[i - LEFT_SIZE];
    }

    buffer
}

/// Splice a sized arrays together into a two arrays.
///
/// Create two arrays the left being filled up first, then the right. If the given `original`
/// array is two small to fill both buffers, the `fill` value is used for the remaining elements.
///
/// # Examples
///
/// ```
/// use array_utils::splice;
///
/// assert_eq!(splice([1, 2, 3, 4, 5, 6], 0), ([1, 2, 3], [4, 5, 6]));
///
/// // Leftover elements are not used
/// assert_eq!(splice([1, 2, 3, 4, 5, 6, 0, 0], 0), ([1, 2, 3], [4, 5, 6]));
///
/// // If the `original` buffer is to small the remaining elements are filled in.
/// assert_eq!(splice([1, 2, 3, 4, 5], 0), ([1, 2, 3], [4, 5, 0]));
/// ```
#[cfg(feature = "splice")]
pub fn splice<T, const ORIGINAL_SIZE: usize, const LEFT_SIZE: usize, const RIGHT_SIZE: usize>(
    original: [T; ORIGINAL_SIZE],
    fill: T,
) -> ([T; LEFT_SIZE], [T; RIGHT_SIZE])
where
    T: Copy,
{
    let mut left = [fill; LEFT_SIZE];
    let mut right = [fill; RIGHT_SIZE];

    for i in 0..min_of_sizes(LEFT_SIZE, ORIGINAL_SIZE) {
        left[i] = original[i];
    }

    for i in LEFT_SIZE..min_of_sizes(LEFT_SIZE + RIGHT_SIZE, ORIGINAL_SIZE) {
        if i - LEFT_SIZE >= RIGHT_SIZE {
            break;
        }

        right[i - LEFT_SIZE] = original[i];
    }

    (left, right)
}

/// Create a sized slice of an array.
///
/// Create a copy a part of sized array `original` from the index `from` till the index `till`.
/// Filling the elements which are are contained in the `original` array with the `fill` value.
///
/// # Examples
///
/// ```
/// use array_utils::sized_slice;
///
/// assert_eq!(sized_slice([1, 2, 3, 4, 5, 6, 7, 8, 9], 2, 6, 0), [3, 4, 5, 6]);
/// assert_eq!(sized_slice([1, 2, 3, 4, 5, 6, 7, 8, 9], 6, 8, 0), [7, 8, 0, 0, 0, 0]);
/// ```
#[cfg(feature = "slice")]
pub fn sized_slice<T, const ORIGINAL_SIZE: usize, const SLICE_SIZE: usize>(
    original: [T; ORIGINAL_SIZE],
    from: usize,
    till: usize,
    fill: T,
) -> [T; SLICE_SIZE]
where
    T: Copy,
{
    let mut buffer = [fill; SLICE_SIZE];

    for i in from..min_of_sizes(till, ORIGINAL_SIZE) {
        if i - from >= SLICE_SIZE {
            break;
        }

        buffer[i - from] = original[i];
    }
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "initialize")]
    fn init_from() {
        assert_eq!(initialize_from(|index| index), []);
        assert_eq!(initialize_from(|index| index), [0, 1, 2, 3, 4]);
        assert_eq!(initialize_from(|index| 2 * index), [0, 2, 4, 6, 8]);
        assert_eq!(initialize_from(|_| 1), [1; 20]);
        assert_eq!(initialize_from(|index| 5 + index), [5, 6, 7, 8, 9, 10]);
    }

    #[test]
    #[cfg(feature = "initialize")]
    fn init_till() {
        assert_eq!(
            initialize_till(|index| index, 4, 42),
            ([0, 1, 2, 3, 42, 42], 4)
        );
        assert_eq!(initialize_till(|index| index, 5, 42), ([0, 1, 2, 3, 4], 5));
        assert_eq!(
            initialize_till(|index| 2 * index, 8, 42),
            ([0, 2, 4, 6, 42], 4)
        );
        assert_eq!(initialize_till(|_| 1, 3, 42), ([1; 20], 20));
        assert_eq!(
            initialize_till(|index| 5 + index, 20, 42),
            ([5, 6, 7, 8, 9, 10], 6)
        );
    }

    #[test]
    #[cfg(feature = "initialize")]
    fn init_from_option() {
        assert_eq!(
            initialize_from_option(|index| if index == 10 { None } else { Some(index) }, 42),
            ([0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 10)
        );
        assert_eq!(
            initialize_from_option(|index| if index == 10 { None } else { Some(index) }, 42),
            ([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 42, 42, 42, 42], 10)
        );
        assert_eq!(
            initialize_from_option(|index| if index > 4 { None } else { Some(index) }, 42),
            ([0, 1, 2, 3, 4, 42, 42, 42, 42], 5)
        );
    }

    #[test]
    #[cfg(feature = "initialize")]
    fn init_from_result() {
        assert_eq!(
            initialize_from_result(|index| if index == 10 { Err(()) } else { Ok(index) }, 42),
            ([0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 10)
        );
        assert_eq!(
            initialize_from_result(|index| if index == 10 { Err(()) } else { Ok(index) }, 42),
            ([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 42, 42, 42, 42], 10)
        );
        assert_eq!(
            initialize_from_result(|index| if index > 4 { Err(()) } else { Ok(index) }, 42),
            ([0, 1, 2, 3, 4, 42, 42, 42, 42], 5)
        );
    }

    #[test]
    #[cfg(feature = "drift")]
    fn drift_st() {
        assert_eq!(
            drift_to_begin(initialize_from(|index| index), 10, 2, 42),
            superimpose([42; 13], [10, 11, 12], 2)
        );
        assert_eq!(
            drift_to_begin(initialize_from(|index| index), 10, 0, 42),
            superimpose([42; 13], [10, 11, 12], 0)
        );
        assert_eq!(
            drift_to_begin(initialize_from(|index| index), 10, 1, 42),
            superimpose([42; 13], [10, 11, 12], 1)
        );
    }

    #[test]
    #[cfg(feature = "drift")]
    fn drift_nd() {
        assert_eq!(
            drift_to_end(initialize_from(|index| index), 3, 2, 42),
            [42, 42, 0, 1, 2, 42, 42]
        );
        assert_eq!(
            drift_to_end(initialize_from(|index| index), 3, 0, 42),
            [42, 42, 0, 1, 2]
        );
        assert_eq!(
            drift_to_end(initialize_from(|index| index), 3, 1, 42),
            [0, 1, 2, 42]
        );
    }

    #[test]
    #[cfg(feature = "resize")]
    fn arr_resize() {
        let array: [usize; 10] = initialize_from(|index| index);
        assert_eq!(array_resize(array, 42), [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(
            array_resize(array, 42),
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 42, 42]
        );
    }

    #[test]
    #[cfg(feature = "superimpose")]
    fn super_impose() {
        let array: [usize; 10] = initialize_from(|index| index);

        assert_eq!(
            superimpose(array, [0, 1, 2, 3], 4),
            [0, 1, 2, 3, 0, 1, 2, 3, 8, 9]
        );
        assert_eq!(
            superimpose(array, [0, 1, 2, 3], 8),
            [0, 1, 2, 3, 4, 5, 6, 7, 0, 1]
        );
        assert_eq!(superimpose(array, [0, 1, 2, 3], 10), array);
        assert_eq!(superimpose(array, [0, 1, 2, 3], 0), array);
    }

    #[test]
    #[cfg(feature = "join")]
    fn join_arrays() {
        assert_eq!(
            join([4, 5, 6, 7], [0, 1, 2, 3], 0),
            [4, 5, 6, 7, 0, 1, 2, 3]
        );
        assert_eq!(
            join([4, 5, 6, 7, 8], [0, 1, 2, 3], 0),
            [4, 5, 6, 7, 8, 0, 1, 2, 3]
        );
        assert_eq!(
            join([4, 5, 6, 7], [0, 1, 2, 3], 0),
            [4, 5, 6, 7, 0, 1, 2, 3, 0]
        );
    }

    #[test]
    #[cfg(feature = "splice")]
    fn splice_arrays() {
        assert_eq!(
            splice([4, 5, 6, 7, 0, 1, 2, 3], 0),
            ([4, 5, 6, 7], [0, 1, 2, 3])
        );
        assert_eq!(
            splice([4, 5, 6, 7, 8, 0, 1, 2, 3], 0),
            ([4, 5, 6, 7, 8], [0, 1, 2, 3])
        );
        assert_eq!(
            splice([4, 5, 6, 7, 0, 1, 2, 3, 0], 0),
            ([4, 5, 6, 7], [0, 1, 2, 3])
        );
        assert_eq!(
            splice([4, 5, 6, 7, 0, 1, 2, 3, 0], 0),
            ([4, 5, 6, 7], [0, 1, 2, 3, 0, 0, 0, 0])
        );
    }

    #[test]
    #[cfg(feature = "slice")]
    fn sized_slices() {
        assert_eq!(
            sized_slice([4, 5, 6, 7, 0, 1, 2, 3], 4, 8, 0),
            ([0, 1, 2, 3])
        );
        assert_eq!(
            sized_slice([4, 5, 6, 7, 0, 1, 2, 3], 4, 10, 0),
            ([0, 1, 2, 3, 0, 0])
        );
        assert_eq!(
            sized_slice([4, 5, 6, 7, 0, 1, 2, 3], 0, 10, 0),
            ([4, 5, 6, 7, 0, 1])
        );
    }
}
