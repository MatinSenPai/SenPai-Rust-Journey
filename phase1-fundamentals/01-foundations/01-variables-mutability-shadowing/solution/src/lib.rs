//! Reference solution for 1.1.1 — Variables, mutability, shadowing.

/// Seconds in `hours` hours plus `minutes` minutes plus `seconds` seconds.
pub fn total_seconds(hours: u32, minutes: u32, seconds: u32) -> u32 {
    let from_hours = hours * 3600;
    let from_minutes = minutes * 60;
    from_hours + from_minutes + seconds
}

/// `a`, `b` and `c` added up through one mutable binding.
pub fn running_total(a: u32, b: u32, c: u32) -> u32 {
    let mut total = a;
    total += b;
    total += c;
    total
}

/// `raw` doubled, plus ten, halved — each step shadowing the last.
pub fn scaled(raw: u32) -> u32 {
    let raw = raw * 2;
    let raw = raw + 10;
    let raw = raw / 2;
    raw
}

/// How many whole orders fit in `stock`.
pub fn full_orders(stock: u32) -> u32 {
    stock / MAX_PER_ORDER
}

/// The largest number of items one order may contain.
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
