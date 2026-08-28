//! Exercises for 1.1.5 — Control flow.
//!
//! One for each shape: an `if` chain, a `for`, a `while`, an early `return`
//! out of a loop, and a `loop` that breaks with a value.

/// The letter grade for a percentage score.
///
/// 90 and above is `'A'`, 80 to 89 is `'B'`, 70 to 79 is `'C'`, 60 to 69 is
/// `'D'`, and anything below 60 is `'F'`.
///
/// # Examples
///
/// `grade(95)` returns `'A'`.
/// `grade(70)` returns `'C'`.
/// `grade(0)` returns `'F'`.
pub fn grade(score: u32) -> char {
    todo!("work down the bands until one of them matches")
}

/// Every whole number from 1 to `n` added together.
///
/// `sum_to(0)` is `0` — there is nothing to add.
///
/// # Examples
///
/// `sum_to(5)` returns `15`.
/// `sum_to(1)` returns `1`.
/// `sum_to(100)` returns `5_050`.
pub fn sum_to(n: u32) -> u32 {
    todo!("visit every number from 1 to n and keep a running total")
}

/// How many decimal digits `n` is written with.
///
/// `count_digits(0)` is `1`: zero is written with one digit.
///
/// # Examples
///
/// `count_digits(7)` returns `1`.
/// `count_digits(1_000)` returns `4`.
/// `count_digits(4_294_967_295)` returns `10`.
pub fn count_digits(n: u32) -> u32 {
    todo!("strip one digit at a time and count how many times you can")
}

/// The position of the first negative reading.
///
/// When there is no negative reading at all, return `6` — the length of the
/// array, which is one past the last valid index.
///
/// (That convention is how C answers this question, and it is a bad answer:
/// the "not found" value is a number, so nothing stops you indexing with it.
/// 1.6.1 replaces it with something the compiler checks.)
///
/// # Examples
///
/// `index_of_first_negative([5, 3, -2, 8, -9, 1])` returns `2`.
/// `index_of_first_negative([-1, 0, 0, 0, 0, 0])` returns `0`.
/// `index_of_first_negative([1, 2, 3, 4, 5, 6])` returns `6`.
pub fn index_of_first_negative(readings: [i32; 6]) -> usize {
    todo!("walk the positions in order and stop at the first one that matches")
}

/// How many steps the Collatz sequence takes to get from `start` down to 1.
///
/// Each step: if the number is even, halve it; if it is odd, treble it and add
/// one. `start` is at least 1, and `collatz_steps(1)` is `0` because it is
/// already there.
///
/// # Examples
///
/// `collatz_steps(1)` returns `0`.
/// `collatz_steps(6)` returns `8` — 6, 3, 10, 5, 16, 8, 4, 2, 1.
/// `collatz_steps(7)` returns `16`.
pub fn collatz_steps(start: u32) -> u32 {
    todo!("keep applying the rule until you reach 1, counting the steps")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_every_band() {
        assert_eq!(grade(100), 'A');
        assert_eq!(grade(90), 'A');
        assert_eq!(grade(89), 'B');
        assert_eq!(grade(80), 'B');
        assert_eq!(grade(70), 'C');
        assert_eq!(grade(69), 'D');
        assert_eq!(grade(60), 'D');
        assert_eq!(grade(59), 'F');
        assert_eq!(grade(0), 'F');
    }

    #[test]
    fn adds_up_to_n() {
        assert_eq!(sum_to(0), 0);
        assert_eq!(sum_to(1), 1);
        assert_eq!(sum_to(5), 15);
        assert_eq!(sum_to(100), 5_050);
    }

    #[test]
    fn counts_decimal_digits() {
        assert_eq!(count_digits(0), 1);
        assert_eq!(count_digits(7), 1);
        assert_eq!(count_digits(10), 2);
        assert_eq!(count_digits(99), 2);
        assert_eq!(count_digits(1_000), 4);
        assert_eq!(count_digits(4_294_967_295), 10);
    }

    #[test]
    fn finds_the_first_negative() {
        assert_eq!(index_of_first_negative([5, 3, -2, 8, -9, 1]), 2);
        assert_eq!(index_of_first_negative([-1, 0, 0, 0, 0, 0]), 0);
        assert_eq!(index_of_first_negative([0, 0, 0, 0, 0, -1]), 5);
        assert_eq!(index_of_first_negative([1, 2, 3, 4, 5, 6]), 6);
    }

    #[test]
    fn counts_collatz_steps() {
        assert_eq!(collatz_steps(1), 0);
        assert_eq!(collatz_steps(2), 1);
        assert_eq!(collatz_steps(6), 8);
        assert_eq!(collatz_steps(7), 16);
        assert_eq!(collatz_steps(27), 111);
    }
}
