//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run -p p2-10-01-pattern-matching-depth --example 04-or-pattern-mismatch-broken --features broken`
//! and read the error.
//!
//! `status` is `u16` in `Request`, `uptime_secs` is `u64` in `Heartbeat`.
//! Binding both to the same name `code` in one or-pattern arm asks `code`
//! to be two different types at once.

enum LogEvent {
    Request { status: u16 },
    Heartbeat { uptime_secs: u64 },
}

fn code_of(event: LogEvent) -> String {
    match event {
        LogEvent::Request { status: code } | LogEvent::Heartbeat { uptime_secs: code } => {
            format!("{code}")
        }
    }
}

fn main() {
    println!("{}", code_of(LogEvent::Request { status: 200 }));
}
