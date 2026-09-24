//! HTTP/1.1 defaults to a persistent connection unless someone sends
//! `Connection: close`; HTTP/1.0 defaults the other way — closed, unless
//! someone opts in with `Connection: keep-alive`. Same header, same two
//! token values, opposite meaning of *silence*, depending only on the
//! version on the request line.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 04-keep-alive-defaults

fn should_keep_alive(version: &str, connection_header: Option<&str>) -> bool {
    let says = |token: &str| {
        connection_header.is_some_and(|value| {
            value.split(',').any(|part| part.trim().eq_ignore_ascii_case(token))
        })
    };

    match version {
        "HTTP/1.1" => !says("close"),
        "HTTP/1.0" => says("keep-alive"),
        _ => false,
    }
}

fn main() {
    let cases = [
        ("HTTP/1.1", None),
        ("HTTP/1.1", Some("close")),
        ("HTTP/1.1", Some("keep-alive")),
        ("HTTP/1.0", None),
        ("HTTP/1.0", Some("keep-alive")),
        ("HTTP/1.0", Some("close")),
    ];
    for (version, header) in cases {
        let alive = should_keep_alive(version, header);
        println!(
            "{version}  Connection: {:<12} -> keep-alive = {alive}",
            header.unwrap_or("(absent)")
        );
    }
}
