//! An or-pattern (`|`) can match several different variant *shapes* in one
//! arm, but every alternative must bind the same names to the same types —
//! the compiler checks this before it checks anything else about the arm.
//! Here `status` and `retry_after` are both `u16`, so binding them to one
//! shared name, `n`, is legal.
//!
//!     cargo run -p p2-10-01-pattern-matching-depth --example 01-or-pattern-type-consistency

enum LogEvent {
    Request { status: u16 },
    RateLimited { retry_after: u16 },
}

/// A single number worth reporting, whichever shape the event turned out
/// to be: the HTTP status if it's a `Request`, the retry delay if it's a
/// `RateLimited`.
fn number_worth_reporting(event: &LogEvent) -> u16 {
    match event {
        LogEvent::Request { status: n } | LogEvent::RateLimited { retry_after: n } => *n,
    }
}

fn main() {
    let request = LogEvent::Request { status: 503 };
    let limited = LogEvent::RateLimited { retry_after: 30 };

    println!("{}", number_worth_reporting(&request));
    println!("{}", number_worth_reporting(&limited));
}
