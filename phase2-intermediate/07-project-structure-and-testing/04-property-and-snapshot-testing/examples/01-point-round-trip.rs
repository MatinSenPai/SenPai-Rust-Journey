//! A round-trip property: parse(format(p)) should give back `p`, for every
//! `Point`, not just the ones you thought to try by hand.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn format(&self) -> String {
        format!("{},{}", self.x, self.y)
    }

    fn parse(s: &str) -> Option<Point> {
        let (x_str, y_str) = s.split_once(',')?;
        let x = x_str.parse().ok()?;
        let y = y_str.parse().ok()?;
        Some(Point { x, y })
    }
}

fn main() {
    let p = Point { x: -7, y: 3 };
    let text = p.format();
    println!("{p:?} -> \"{text}\" -> {:?}", Point::parse(&text));
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // The kind of test 2.7.2 taught: one hand-picked input, one assertion.
    // It passes — and proves the round trip only for (3, 4).
    #[test]
    fn round_trips_a_hand_picked_point() {
        let p = Point { x: 3, y: 4 };
        assert_eq!(Point::parse(&p.format()), Some(p));
    }

    proptest! {
        #[test]
        fn round_trip(x in any::<i32>(), y in any::<i32>()) {
            let original = Point { x, y };
            let parsed = Point::parse(&original.format());
            prop_assert_eq!(parsed, Some(original));
        }
    }
}
