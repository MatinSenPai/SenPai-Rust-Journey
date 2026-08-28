//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run --example 05-map-instead-of-and-then --features broken` and
//! read the error.

fn find_setting(key: &str) -> Option<i32> {
    match key {
        "timeout" => Some(30),
        "retries" => Some(3),
        _ => None,
    }
}

fn double_if_positive(n: i32) -> Option<i32> {
    if n > 0 {
        Some(n * 2)
    } else {
        None
    }
}

fn main() {
    let timeout: Option<i32> = find_setting("timeout").map(double_if_positive);
    println!("timeout: {timeout:?}");
}
