//! Two pairs of "does the same thing, two different ways" functions —
//! the material `benches/comparison.rs` measures. Both variants in both
//! pairs are fully implemented here (unlike most lessons in this repo,
//! there's no `todo!()` in this file): this lesson's exercise is writing
//! the *benchmark*, not these functions. See README.md.

/// The textbook naive recursive Fibonacci: `fib(n) = fib(n-1) + fib(n-2)`.
/// Correct, but does an enormous amount of redundant work — `fib(30)`
/// recomputes `fib(2)` hundreds of thousands of times, because nothing is
/// memoized. Big-O: exponential, roughly O(2^n).
pub fn fib_recursive(n: u64) -> u64 {
    if n < 2 {
        n
    } else {
        fib_recursive(n - 1) + fib_recursive(n - 2)
    }
}

/// The same function, computed by walking forward and keeping only the
/// last two values — no recursion, no redundant work. Big-O: linear,
/// O(n). Same output as `fib_recursive` for every input; see the tests
/// below. This is the pair `benches/comparison.rs` puts side by side to
/// make "algorithmic complexity, not micro-optimization, is usually where
/// the real win is" visible in an actual measurement.
pub fn fib_iterative(n: u64) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        let next = a + b;
        a = b;
        b = next;
    }
    a
}

/// Builds one `String` out of `words` by starting from `String::new()`
/// (zero capacity) and `push_str`-ing each word plus a separator. Every
/// time the growing buffer's length would exceed its current capacity,
/// the allocator has to grow it — typically by allocating a new, larger
/// buffer, copying every byte written so far into it, and freeing the
/// old one. For a long `words` slice, that's several reallocations and
/// O(n) bytes copied more than once each.
pub fn concat_naive(words: &[&str]) -> String {
    let mut out = String::new();
    for word in words {
        out.push_str(word);
        out.push(' ');
    }
    out
}

/// Same output as `concat_naive`, but computes the exact final byte
/// length up front and allocates it once with `String::with_capacity`.
/// Every subsequent `push_str`/`push` writes into already-allocated
/// space — no reallocation, no re-copying, for the entire loop. This is
/// the Phase 1 String/&str lesson's "own vs. borrow" distinction showing
/// up again as a performance concern: preallocating is only possible
/// because we can cheaply compute the total owned size before writing
/// any of it.
pub fn concat_with_capacity(words: &[&str]) -> String {
    let total_len: usize = words.iter().map(|w| w.len() + 1).sum();
    let mut out = String::with_capacity(total_len);
    for word in words {
        out.push_str(word);
        out.push(' ');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fib_recursive_matches_known_values() {
        assert_eq!(fib_recursive(0), 0);
        assert_eq!(fib_recursive(1), 1);
        assert_eq!(fib_recursive(10), 55);
        assert_eq!(fib_recursive(20), 6765);
    }

    #[test]
    fn fib_iterative_matches_fib_recursive_across_a_range() {
        for n in 0..25 {
            assert_eq!(fib_iterative(n), fib_recursive(n), "mismatch at n={n}");
        }
    }

    #[test]
    fn concat_naive_and_concat_with_capacity_produce_identical_output() {
        let words = ["senpai", "is", "learning", "rust"];
        assert_eq!(concat_naive(&words), concat_with_capacity(&words));
        assert_eq!(concat_naive(&words), "senpai is learning rust ");
    }

    #[test]
    fn concat_handles_an_empty_slice() {
        assert_eq!(concat_naive(&[]), "");
        assert_eq!(concat_with_capacity(&[]), "");
    }
}
