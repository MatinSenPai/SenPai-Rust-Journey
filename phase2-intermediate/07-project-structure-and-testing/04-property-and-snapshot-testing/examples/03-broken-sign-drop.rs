//! DELIBERATELY BROKEN — expected: a proptest failure — the round-trip
//! property fails and proptest shrinks it to a minimal counterexample.
//!
//!     cargo test -p p2-07-04-property-and-snapshot-testing --example 03-broken-sign-drop --features broken

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn format(&self) -> String {
        // BUG: unsigned_abs() throws away the sign of `x`.
        format!("{},{}", self.x.unsigned_abs(), self.y)
    }

    fn parse(s: &str) -> Option<Point> {
        let (x_str, y_str) = s.split_once(',')?;
        let x = x_str.parse().ok()?;
        let y = y_str.parse().ok()?;
        Some(Point { x, y })
    }
}

fn main() {
    let original = Point { x: -1, y: 0 };
    let text = original.format();
    let round_tripped = Point::parse(&text);
    println!("{original:?} -> \"{text}\" -> {round_tripped:?} — the sign of x is just gone");
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig {
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        #[test]
        fn round_trip(x in any::<i32>(), y in any::<i32>()) {
            let original = Point { x, y };
            let parsed = Point::parse(&original.format());
            prop_assert_eq!(parsed, Some(original));
        }
    }
}
