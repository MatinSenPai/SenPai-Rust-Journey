//! `double_it!`'s transcriber declares its own `start`. The caller has a
//! `start` too. Run this to see that they never collide.

macro_rules! double_it {
    ($x:expr) => {{
        let start = $x;
        start + start
    }};
}

fn main() {
    let start = 100;
    let doubled = double_it!(7);
    println!("caller's start: {start}");
    println!("double_it!(7):  {doubled}");
}
