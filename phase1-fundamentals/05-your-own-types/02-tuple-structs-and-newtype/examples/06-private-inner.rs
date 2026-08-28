//! DELIBERATELY BROKEN — expected: E0616
//! Run `cargo run -p p1-05-02-tuple-structs-and-newtype --example
//! 06-private-inner --features broken` and read the error.
//!
//! The point of a validating newtype is that the wrapped value is not
//! reachable from outside. Here is what that refusal looks like.

mod rates {
    #[derive(Debug)]
    pub struct Percent(u8);

    impl Percent {
        pub fn new(value: u8) -> Percent {
            if value > 100 {
                Percent(100)
            } else {
                Percent(value)
            }
        }
    }
}

fn main() {
    let fee = rates::Percent::new(9);
    println!("{}", fee.0);
}
