//! Two faces of the same rule. Part one: matching a `&LogEvent` binds its
//! fields by reference automatically ("default binding modes" / "match
//! ergonomics") — no `ref`, no `&` in the pattern, and nothing is moved.
//! Part two: matching an *owned* value moves by default, so `ref` is the
//! one tool left for borrowing a single field out of it while leaving the
//! rest — and the value as a whole — usable afterward.
//!
//!     cargo run -p p2-10-01-pattern-matching-depth --example 02-binding-modes-and-ref

#[derive(Debug)]
enum LogEvent {
    Request { method: String, path: String },
    Message { severity: u8, text: String },
}

fn method_of(event: &LogEvent) -> Option<&str> {
    match event {
        // `event` is `&LogEvent`, so `method` arrives as `&String`, not
        // `String` — `.as_str()` bridges it to the `&str` this returns.
        LogEvent::Request { method, .. } => Some(method.as_str()),
        _ => None,
    }
}

fn main() {
    let request = LogEvent::Request {
        method: "POST".to_string(),
        path: "/login".to_string(),
    };
    println!("{:?}", method_of(&request));
    // `request` was only ever borrowed above — still fully usable here.
    println!("{request:?}");

    let owned = LogEvent::Message {
        severity: 2,
        text: "disk 91% full".to_string(),
    };
    match owned {
        // `owned` itself is not a reference, so without `ref` this arm
        // would MOVE `text` out of `owned` — see the broken example for
        // exactly what that costs.
        LogEvent::Message { severity, ref text } => println!("[{severity}] {text}"),
        LogEvent::Request { .. } => {}
    }
    // `text` was only borrowed, so `owned` as a whole is still here.
    println!("{owned:?}");
}
