//! DELIBERATELY BROKEN — expected: E0308.
//! `.map(|(_, v)| v)` hands back a `&String`, not a `&str` — an easy mistake
//! once `eq_ignore_ascii_case` is doing the case-insensitive part correctly
//! and the rest *looks* done.
//!
//!     cargo build -p p3-01-02-hand-rolled-http-parser --example 06-header-returns-string-not-str-broken --features broken

struct Headers {
    entries: Vec<(String, String)>,
}

impl Headers {
    fn get(&self, name: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v)
    }
}

fn main() {
    let headers = Headers {
        entries: vec![("Host".to_string(), "localhost".to_string())],
    };
    println!("{:?}", headers.get("host"));
}
