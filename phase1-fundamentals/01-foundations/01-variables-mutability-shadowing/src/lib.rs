//! Exercises for 1.1.1 — Variables, mutability, shadowing.
//!
//! Everything here is deliberately numeric. Text has its own module later;
//! keeping these to numbers means the only new ideas are `let`, `mut`,
//! shadowing and `const`.

/// Seconds in `hours` hours plus `minutes` minutes plus `seconds` seconds.
///
/// Build it in named steps rather than one long expression — the point of
/// the exercise is `let` bindings, not arithmetic.
///
/// # Examples
///
/// `total_seconds(1, 0, 0)` returns `3600`.
/// `total_seconds(0, 2, 30)` returns `150`.
/// `total_seconds(0, 0, 0)` returns `0`.
pub fn total_seconds(hours: u32, minutes: u32, seconds: u32) -> u32 {
    todo!("convert hours and minutes into seconds, then add all three together")
}

/// `a`, `b` and `c` added up — using **one** mutable binding that you update
/// three times, not a single sum expression.
///
/// Written this way on purpose: it's the shape `mut` exists for.
///
/// # Examples
///
/// `running_total(1, 2, 3)` returns `6`.
/// `running_total(0, 0, 0)` returns `0`.
pub fn running_total(a: u32, b: u32, c: u32) -> u32 {
    todo!("start a mutable total at `a`, then add `b` and then `c` to it")
}

/// `raw` put through three steps, each one shadowing the last: double it,
/// add ten, then halve it.
///
/// Use the same name three times rather than inventing `doubled`,
/// `plus_ten`, `halved`. That is what shadowing is for.
///
/// # Examples
///
/// `scaled(10)` returns `15` — 10 doubled is 20, plus 10 is 30, halved is 15.
/// `scaled(0)` returns `5`.
/// `scaled(3)` returns `8` — 6, then 16, then 8.
pub fn scaled(raw: u32) -> u32 {
    todo!("shadow `raw` through the three steps in order and return the result")
}

/// How many whole orders fit in `stock`, given [`MAX_PER_ORDER`] items per
/// order. A partly-filled order does not count.
///
/// # Examples
///
/// `full_orders(120)` returns `2` — two orders of 50, with 20 left over.
/// `full_orders(50)` returns `1`.
/// `full_orders(49)` returns `0`.
pub fn full_orders(stock: u32) -> u32 {
    todo!("divide the stock by the constant; whole-number division already drops the remainder")
}

/// The largest number of items one order may contain.
///
/// A `const` is computed before the program runs and has no address of its
/// own — it is substituted wherever you use it.
pub const MAX_PER_ORDER: u32 = 50;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_seconds() {
        assert_eq!(total_seconds(1, 0, 0), 3600);
        assert_eq!(total_seconds(0, 2, 30), 150);
        assert_eq!(total_seconds(0, 0, 0), 0);
        assert_eq!(total_seconds(2, 1, 1), 7261);
    }

    #[test]
    fn accumulates() {
        assert_eq!(running_total(1, 2, 3), 6);
        assert_eq!(running_total(0, 0, 0), 0);
        assert_eq!(running_total(10, 0, 5), 15);
    }

    #[test]
    fn scales_in_three_steps() {
        assert_eq!(scaled(10), 15);
        assert_eq!(scaled(0), 5);
        assert_eq!(scaled(3), 8);
    }

    #[test]
    fn counts_full_orders() {
        assert_eq!(full_orders(120), 2);
        assert_eq!(full_orders(50), 1);
        assert_eq!(full_orders(49), 0);
        assert_eq!(full_orders(0), 0);
    }
}
