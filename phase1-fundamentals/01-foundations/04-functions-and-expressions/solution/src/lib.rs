//! Reference solution for 1.1.4 — Functions and expressions.
//!
//! Every one of these has a body that is a single expression, or a couple of
//! `let`s and then a single expression. None of them needs `return`, and none
//! of them needs a condition — a decision can be an expression too.

/// The length of the long side of a right-angled triangle whose short sides
/// are `a` and `b`.
///
/// That is the square root of `a * a + b * b`. `f64` has a `.sqrt()` method.
///
/// # Examples
///
/// `hypotenuse(3.0, 4.0)` returns `5.0`.
/// `hypotenuse(5.0, 12.0)` returns `13.0`.
/// `hypotenuse(0.0, 0.0)` returns `0.0`.
pub fn hypotenuse(a: f64, b: f64) -> f64 {
    (a * a + b * b).sqrt()
}

/// How many boxes it takes to hold `items`, when each box holds `per_box`.
///
/// A part-full box still counts as a box: 13 items at 5 per box need 3 boxes,
/// not 2. `per_box` is never zero.
///
/// # Examples
///
/// `box_count(13, 5)` returns `3`.
/// `box_count(10, 5)` returns `2`.
/// `box_count(1, 5)` returns `1`.
/// `box_count(0, 5)` returns `0`.
pub fn box_count(items: u32, per_box: u32) -> u32 {
    items.div_ceil(per_box)
}

/// Whether `year` is a leap year in the Gregorian calendar.
///
/// The rule has three parts: a year divisible by 4 is a leap year, except that
/// a year divisible by 100 is not, except that a year divisible by 400 is.
///
/// Write it as one expression. You have not met `if` yet and you do not need
/// it: `&&`, `||` and `==` produce a `bool` on their own.
///
/// # Examples
///
/// `is_leap_year(2024)` returns `true`.
/// `is_leap_year(2023)` returns `false`.
/// `is_leap_year(1900)` returns `false` — divisible by 100.
/// `is_leap_year(2000)` returns `true` — divisible by 400.
pub fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// What an order costs in rial, after a percentage discount.
///
/// The subtotal is the unit price times the quantity. The discount is that
/// percentage of the subtotal, rounded **down** to a whole rial, and taken off.
///
/// `discount_percent` is between 0 and 100.
///
/// # Examples
///
/// `total_price(1_000, 3, 0)` returns `3_000`.
/// `total_price(1_000, 3, 10)` returns `2_700`.
/// `total_price(999, 1, 33)` returns `670` — a third of 999 is 329.67, and the
/// discount rounds down to 329.
pub fn total_price(unit_price_rial: u64, quantity: u64, discount_percent: u64) -> u64 {
    let subtotal = unit_price_rial * quantity;
    let discount = subtotal * discount_percent / 100;
    subtotal - discount
}

/// The diagonal of a screen in inches.
///
/// `width_px` and `height_px` are the screen's size in pixels and `dpi` is how
/// many pixels there are per inch. The diagonal in pixels divided by `dpi` is
/// the diagonal in inches.
///
/// Call [`hypotenuse`] rather than repeating it.
///
/// # Examples
///
/// `diagonal_inches(3.0, 4.0, 1.0)` returns `5.0`.
/// `diagonal_inches(30.0, 40.0, 10.0)` returns `5.0`.
pub fn diagonal_inches(width_px: f64, height_px: f64, dpi: f64) -> f64 {
    hypotenuse(width_px, height_px) / dpi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_long_side() {
        // Every expected value here is a whole number and exactly
        // representable, so `assert_eq!` on `f64` is safe. See 1.1.2.
        assert_eq!(hypotenuse(3.0, 4.0), 5.0);
        assert_eq!(hypotenuse(5.0, 12.0), 13.0);
        assert_eq!(hypotenuse(8.0, 15.0), 17.0);
        assert_eq!(hypotenuse(0.0, 0.0), 0.0);
    }

    #[test]
    fn counts_part_full_boxes_as_boxes() {
        assert_eq!(box_count(13, 5), 3);
        assert_eq!(box_count(10, 5), 2);
        assert_eq!(box_count(1, 5), 1);
        assert_eq!(box_count(0, 5), 0);
        assert_eq!(box_count(5, 5), 1);
        assert_eq!(box_count(6, 5), 2);
    }

    #[test]
    fn applies_all_three_leap_year_rules() {
        assert!(is_leap_year(2024));
        assert!(is_leap_year(2000));
        assert!(is_leap_year(1600));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
    }

    #[test]
    fn discounts_an_order() {
        assert_eq!(total_price(1_000, 3, 0), 3_000);
        assert_eq!(total_price(1_000, 3, 10), 2_700);
        assert_eq!(total_price(999, 1, 33), 670);
        assert_eq!(total_price(1_000, 2, 100), 0);
        assert_eq!(total_price(0, 5, 50), 0);
        assert_eq!(total_price(100, 0, 25), 0);
    }

    #[test]
    fn measures_a_screen() {
        assert_eq!(diagonal_inches(3.0, 4.0, 1.0), 5.0);
        assert_eq!(diagonal_inches(30.0, 40.0, 10.0), 5.0);
        assert_eq!(diagonal_inches(0.0, 0.0, 96.0), 0.0);
    }
}
