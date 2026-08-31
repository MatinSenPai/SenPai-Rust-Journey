//! DELIBERATELY BROKEN — expected: E0382
//! Run `cargo run -p p2-10-01-pattern-matching-depth --example 05-forgot-ref-broken --features broken`
//! and read the error.
//!
//! `event` is owned, not a reference, so matching it moves by default.
//! `text` here is bound by value — moving it out of `event` — and then
//! `event` is used again as a whole further down.

#[derive(Debug)]
enum LogEvent {
    Message { severity: u8, text: String },
}

fn main() {
    let event = LogEvent::Message {
        severity: 2,
        text: String::from("disk 91% full"),
    };

    match event {
        LogEvent::Message { text, .. } => println!("{text}"),
    }

    println!("{event:?}");
}
