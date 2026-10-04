//! DELIBERATELY BROKEN — expected: E0004
//! Run `cargo run -p p3-07-02-sessions-vs-jwt --example 04-set-cookie-samesite-non-exhaustive-broken --features broken`
//! and read the error.

#[allow(dead_code)]
enum SameSite {
    Strict,
    Lax,
    None,
}

fn attribute(same_site: SameSite) -> &'static str {
    match same_site {
        SameSite::Strict => "SameSite=Strict",
        SameSite::Lax => "SameSite=Lax",
    }
}

fn main() {
    println!("{}", attribute(SameSite::Lax));
}
