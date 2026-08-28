//! `Iterator::Item` is an associated type, so `Countdown::Item` is `u8` and
//! nothing else — neither call below ever names a type.

struct Countdown {
    remaining: u8,
}

impl Iterator for Countdown {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.remaining == 0 {
            return None;
        }
        let value = self.remaining;
        self.remaining -= 1;
        Some(value)
    }
}

fn main() {
    let full = Countdown { remaining: 5 }.count();
    let odd = Countdown { remaining: 5 }.filter(|v| v % 2 == 1).count();
    println!("full: {full}");
    println!("odd: {odd}");
}
