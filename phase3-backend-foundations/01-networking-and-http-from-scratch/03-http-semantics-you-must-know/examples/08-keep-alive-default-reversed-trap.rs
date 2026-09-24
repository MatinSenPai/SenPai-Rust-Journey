//! No compiler error, no panic — this `should_keep_alive` has the two
//! versions' defaults backwards: it treats HTTP/1.1 as closed unless told
//! otherwise, and HTTP/1.0 as open unless told otherwise — the exact
//! reverse of the real spec defaults. It compiles and runs fine; it is
//! just wrong.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 08-keep-alive-default-reversed-trap

fn should_keep_alive_reversed(version: &str, connection_header: Option<&str>) -> bool {
    let says = |token: &str| {
        connection_header.is_some_and(|value| {
            value.split(',').any(|part| part.trim().eq_ignore_ascii_case(token))
        })
    };

    match version {
        "HTTP/1.1" => says("keep-alive"), // bug: should default to true, not require opt-in
        "HTTP/1.0" => !says("close"),     // bug: should default to false, not require opt-out
        _ => false,
    }
}

fn main() {
    println!(
        "HTTP/1.1, no Connection header -> keep-alive = {}  (should be true)",
        should_keep_alive_reversed("HTTP/1.1", None)
    );
    println!(
        "HTTP/1.0, no Connection header -> keep-alive = {}  (should be false)",
        should_keep_alive_reversed("HTTP/1.0", None)
    );
}
