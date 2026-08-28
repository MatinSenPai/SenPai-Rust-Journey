//! Reference solution for 1.2.1 — Stack and heap.
//!
//! These do not compute anything interesting. They ask you to measure where
//! your data actually is, which is the only way the distinction stops being
//! a diagram and starts being a fact about your program.

use std::mem::{size_of, size_of_val};

/// The stack size of three types, in this order: `[i64; 10]`, `Vec<i64>`,
/// `String`.
///
/// "Stack size" means how many bytes the value itself occupies — not
/// anything it points at.
///
/// On a 64-bit machine this is `(80, 24, 24)`.
pub fn stack_sizes() -> (usize, usize, usize) {
    (
        size_of::<[i64; 10]>(),
        size_of::<Vec<i64>>(),
        size_of::<String>(),
    )
}

/// The capacity of a `Vec<i32>` that started empty and had `pushes` values
/// pushed onto it.
///
/// Capacity is not the same as length: it is how much room has been reserved,
/// which grows in jumps rather than one at a time.
///
/// # Examples
///
/// `capacity_after_pushes(0)` returns `0`.
/// `capacity_after_pushes(1)` returns `4`.
/// `capacity_after_pushes(5)` returns `8`.
pub fn capacity_after_pushes(pushes: usize) -> usize {
    let mut values: Vec<i32> = Vec::new();
    for n in 0..pushes {
        values.push(n as i32);
    }
    values.capacity()
}

/// An empty `Vec<u8>` with room already reserved for `expected` bytes.
///
/// It has no elements in it — reserving room is not the same as filling it.
///
/// # Examples
///
/// `reserve_for(100)` has length `0` and capacity at least `100`.
pub fn reserve_for(expected: usize) -> Vec<u8> {
    Vec::with_capacity(expected)
}

/// How many bytes `text` occupies on the stack, and how many on the heap.
///
/// The first is the size of the `String` value itself. The second is the
/// number of bytes of text it is holding.
///
/// # Examples
///
/// `header_and_heap("hello".to_string())` returns `(24, 5)` on a 64-bit
/// machine. `header_and_heap("سلام".to_string())` returns `(24, 8)`.
pub fn header_and_heap(text: String) -> (usize, usize) {
    (size_of_val(&text), text.len())
}

/// Every byte `values` is responsible for: the value on the stack plus the
/// buffer it reserved on the heap.
///
/// Note that this counts **reserved** room, not used room. A `Vec` with three
/// items and room for ten is holding on to all ten slots' worth of memory.
///
/// # Examples
///
/// A `Vec<i64>` with capacity 10 accounts for `24 + 10 * 8` = `104` bytes on
/// a 64-bit machine, however few items are actually in it.
pub fn total_bytes(values: Vec<i64>) -> usize {
    size_of_val(&values) + values.capacity() * size_of::<i64>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn measures_the_stack_side_of_three_types() {
        let word = size_of::<usize>();
        let (array, vector, string) = stack_sizes();
        assert_eq!(array, 10 * size_of::<i64>());
        // A pointer, a length and a capacity.
        assert_eq!(vector, 3 * word);
        assert_eq!(string, 3 * word);
    }

    #[test]
    fn capacity_grows_in_jumps() {
        assert_eq!(capacity_after_pushes(0), 0);
        assert_eq!(capacity_after_pushes(1), 4);
        assert_eq!(capacity_after_pushes(4), 4);
        assert_eq!(capacity_after_pushes(5), 8);
        assert_eq!(capacity_after_pushes(8), 8);
        assert_eq!(capacity_after_pushes(9), 16);
    }

    #[test]
    fn reserving_room_does_not_fill_it() {
        let reserved = reserve_for(100);
        assert_eq!(reserved.len(), 0);
        assert!(reserved.capacity() >= 100);
        assert_eq!(reserve_for(0).capacity(), 0);
    }

    #[test]
    fn splits_a_string_into_its_two_halves() {
        let word = size_of::<usize>();
        assert_eq!(header_and_heap("hello".to_string()), (3 * word, 5));
        assert_eq!(header_and_heap(String::new()), (3 * word, 0));
        // Four Persian letters, two bytes each — the header does not change.
        assert_eq!(header_and_heap("سلام".to_string()), (3 * word, 8));
    }

    #[test]
    fn counts_reserved_room_not_used_room() {
        let word = size_of::<usize>();

        let mut roomy: Vec<i64> = Vec::with_capacity(10);
        roomy.push(1);
        roomy.push(2);
        roomy.push(3);
        assert_eq!(total_bytes(roomy), 3 * word + 10 * size_of::<i64>());

        let empty: Vec<i64> = Vec::new();
        assert_eq!(total_bytes(empty), 3 * word);
    }
}
