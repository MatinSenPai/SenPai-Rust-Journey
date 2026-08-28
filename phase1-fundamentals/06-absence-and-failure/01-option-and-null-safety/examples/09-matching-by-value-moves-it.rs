//! DELIBERATELY BROKEN — expected: E0382.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 09-matching-by-value-moves-it --features broken

fn describe(nickname: Option<String>) -> String {
    match nickname {
        Some(name) => format!("hello, {name}"),
        None => "hello, stranger".to_string(),
    }
}

fn main() {
    let nickname = Some(String::from("Matin"));
    println!("{}", describe(nickname));
    println!("{}", describe(nickname));
}
