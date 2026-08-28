//! DELIBERATELY BROKEN — expected: a panic at run time (this one compiles)
//!
//! Three `Guard`s open, then a panic before a fourth finishes setting up.
//! Watch what happens to the three that were already alive.
//!
//!     cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
//!
//! Then compare against a build that skips unwinding entirely — no code
//! change needed, only an environment variable overriding the dev profile:
//!
//!     (PowerShell) $env:CARGO_PROFILE_DEV_PANIC='abort'; cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken
//!     (bash)       CARGO_PROFILE_DEV_PANIC=abort cargo run -p p1-06-04-panic-vs-result --example 06-drop-during-unwind --features broken

struct Guard {
    name: &'static str,
}

impl Guard {
    fn new(name: &'static str) -> Guard {
        println!("open  {name}");
        Guard { name }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}

fn main() {
    let _a = Guard::new("a");
    let _b = Guard::new("b");
    let _c = Guard::new("c");
    println!("all three open, about to fail partway through setup");
    panic!("simulated failure while building the fourth resource");
}
