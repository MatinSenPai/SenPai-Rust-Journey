//! The `Set-Cookie` lines for a session cookie, the way a dev server and a
//! production server should send them, and what each attribute is for.
//!
//!     cargo run -p p3-07-02-sessions-vs-jwt --example 02-cookie-attributes

fn cookie(secure: bool, same_site: &str, max_age: Option<u64>) -> String {
    let mut c = String::from("sid=3f9a; Path=/; HttpOnly");
    if secure {
        c.push_str("; Secure");
    }
    c.push_str(&format!("; SameSite={same_site}"));
    if let Some(n) = max_age {
        c.push_str(&format!("; Max-Age={n}"));
    }
    c
}

fn main() {
    println!("dev  : {}", cookie(false, "Lax", Some(3600)));
    println!("prod : {}", cookie(true, "Lax", Some(3600)));
    println!("bank : {}", cookie(true, "Strict", Some(900)));
    println!();
    println!("HttpOnly -> page JavaScript cannot read it (limits XSS theft)");
    println!("Secure   -> only sent over HTTPS (limits network sniffing)");
    println!("SameSite -> not sent on most cross-site requests (limits CSRF)");
    println!("Max-Age  -> the browser forgets it; the SERVER still decides");
}
