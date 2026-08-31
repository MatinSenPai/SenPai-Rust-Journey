use std::marker::PhantomData;

pub fn split_at_mut_demo<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    assert!(mid <= len, "mid out of bounds");

    let ptr = slice.as_mut_ptr();

    // SAFETY: `mid <= len` was just asserted above, so both resulting
    // slices are within the bounds of the original allocation, and the
    // two ranges `[0, mid)` / `[mid, len)` never overlap.
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}

pub struct OwnedBox<T> {
    ptr: *mut T,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for OwnedBox<T> {}

impl<T> OwnedBox<T> {
    pub fn new(value: T) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(value)),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> &T {
        // SAFETY: `ptr` was produced by `Box::into_raw` in `new` and never
        // freed before this call — `Drop::drop` is the only place that
        // frees it, and it can't run while `&self` is held.
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: same allocation as `get`; `&mut self` guarantees
        // exclusive access to it for the duration of the borrow returned.
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for OwnedBox<T> {
    fn drop(&mut self) {
        // SAFETY: `ptr` was produced by `Box::into_raw` in `new`, this is
        // the only place that ever reconstructs a `Box` from it, and
        // `drop` runs at most once per value.
        unsafe {
            drop(Box::from_raw(self.ptr));
        }
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
    fn owned_box_get_mut_writes_through() {
        let mut boxed = OwnedBox::new(10);
        *boxed.get_mut() += 5;
        assert_eq!(*boxed.get(), 15);
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

    #[test]
    fn owned_box_is_send_when_t_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OwnedBox<i32>>();
    }

    #[test]
    fn owned_box_moves_across_a_real_thread() {
        let wrapped = OwnedBox::new(String::from("senpai"));
        let handle = std::thread::spawn(move || wrapped.get().clone());
        assert_eq!(handle.join().unwrap(), "senpai");
    }
}
