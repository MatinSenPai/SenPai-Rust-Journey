use std::marker::PhantomData;

/// Splits `slice` into two independent mutable halves at index `mid` — a
/// simplified reimplementation of `std::slice::split_at_mut`. Safe Rust
/// cannot express this directly (see the README); raw pointers +
/// `unsafe` are the narrowly-scoped escape hatch.
///
/// Build the two halves with `std::slice::from_raw_parts_mut(ptr, len)`,
/// which turns a raw pointer and a length back into a `&mut [T]`. Its
/// safety contract is exactly what the `// SAFETY:` comment below argues:
/// the pointer must be valid for `len` elements, and the two ranges handed
/// out must not overlap.
///
/// # Panics
/// Panics if `mid > slice.len()` — this bounds check happens in ordinary
/// safe code, *before* the unsafe block, and is exactly what keeps the
/// unsafe code below actually safe to call.
pub fn split_at_mut_demo<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    assert!(mid <= len, "mid out of bounds");

    let ptr = slice.as_mut_ptr();

    // SAFETY: `mid <= len` was just asserted above, so both resulting
    // slices are within the bounds of the original allocation. The two
    // slices `[0, mid)` and `[mid, len)` are non-overlapping (mid is a
    // single, fixed split point), so handing out two simultaneous `&mut
    // [T]` into them does not violate the aliasing rule the borrow checker
    // couldn't verify on its own — we're upholding that guarantee by hand
    // instead.
    unsafe {
        todo!(
            "return a pair (first, second): first borrows `ptr`'s elements 0..mid, second \
             borrows its elements mid..len"
        )
    }
}

/// An owning wrapper around a single heap-allocated `T`, built directly on
/// `Box::into_raw` and `Box::from_raw`. Nobody would actually write this in
/// real code — `Box<T>` already does exactly this, correctly — building one
/// by hand, once, is how the mechanism stops being a black box.
///
/// Right after this struct (not inside any `impl` block), also write:
///
/// ```text
/// unsafe impl<T: Send> Send for OwnedBox<T> {}
/// ```
///
/// the same promise the README makes for `RawHolder<T>`. It asserts nothing
/// beyond what `Box<T>` — already `Send` whenever `T` is — already
/// guarantees; see the README for exactly what that promise obligates you
/// to have verified before you write it. Without it, `OwnedBox<T>` stays
/// `!Send` for every `T`, because `ptr: *mut T` alone is enough to block
/// the compiler's automatic `Send`.
pub struct OwnedBox<T> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

// TODO: add the `unsafe impl<T: Send> Send for OwnedBox<T> {}` described in
// the struct's doc comment above. There is nothing to fill in beyond that
// exact line — it does not go inside an `impl<T> OwnedBox<T> { ... }` block.

impl<T> OwnedBox<T> {
    /// Moves `value` onto the heap and takes ownership of it.
    pub fn new(value: T) -> Self {
        todo!("put `value` on the heap with Box::new, hand its raw pointer to Box::into_raw, and store the result in `ptr` alongside `_marker: PhantomData`")
    }

    /// Borrows the wrapped value.
    pub fn get(&self) -> &T {
        todo!("dereference `ptr` to produce a shared reference to the wrapped value")
    }
}

impl<T> Drop for OwnedBox<T> {
    /// Runs when an `OwnedBox<T>` goes out of scope. Must drop the wrapped
    /// value exactly once — no leak, no double free.
    fn drop(&mut self) {
        todo!("reconstruct a Box<T> from `ptr` with Box::from_raw, and let it drop normally")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_into_two_independent_halves() {
        let mut data = [1, 2, 3, 4, 5];
        let (left, right) = split_at_mut_demo(&mut data, 2);
        assert_eq!(left, &mut [1, 2]);
        assert_eq!(right, &mut [3, 4, 5]);
    }

    #[test]
    fn mutating_one_half_does_not_affect_the_other() {
        let mut data = [1, 2, 3, 4, 5];
        let (left, right) = split_at_mut_demo(&mut data, 2);
        left[0] = 100;
        right[0] = 200;
        assert_eq!(data, [100, 2, 200, 4, 5]);
    }

    #[test]
    fn mid_at_zero_or_len_are_valid_edge_cases() {
        let mut data = [1, 2, 3];
        let (left, right) = split_at_mut_demo(&mut data, 0);
        assert_eq!(left, &mut [] as &mut [i32]);
        assert_eq!(right, &mut [1, 2, 3]);

        let mut data2 = [1, 2, 3];
        let (left2, right2) = split_at_mut_demo(&mut data2, 3);
        assert_eq!(left2, &mut [1, 2, 3]);
        assert_eq!(right2, &mut [] as &mut [i32]);
    }

    #[test]
    #[should_panic(expected = "mid out of bounds")]
    fn panics_when_mid_exceeds_length() {
        let mut data = [1, 2, 3];
        split_at_mut_demo(&mut data, 10);
    }

    #[test]
    fn owned_box_new_and_get_roundtrip() {
        let boxed = OwnedBox::new(42);
        assert_eq!(*boxed.get(), 42);
    }

    #[test]
    fn owned_box_drop_runs_exactly_once() {
        use std::cell::Cell;

        struct DropCounter<'a>(&'a Cell<u32>);
        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let count = Cell::new(0u32);
        {
            let wrapped = OwnedBox::new(DropCounter(&count));
            assert_eq!(count.get(), 0);
            drop(wrapped);
        }
        assert_eq!(count.get(), 1);
    }

    // The `unsafe impl<T: Send> Send for OwnedBox<T> {}` requirement is a
    // compile-time trait bound, not a runtime behavior — there's no way to
    // test it that doesn't fail to *compile* until it's written, which
    // would take this whole skeleton down with it. `solution/src/lib.rs`
    // carries the two tests that check it: build the impl, then compare.
}
