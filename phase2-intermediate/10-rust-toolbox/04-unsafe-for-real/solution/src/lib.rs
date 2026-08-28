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
}
