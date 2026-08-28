//! Run: cargo run -p p2-04-01-lifetime-basics-and-elision --example 03-elision-rule-three-method
//!
//! `Ticket` itself holds an owned `String` — no lifetime parameter on the
//! struct. Only the method below borrows, and only from `&self`.

struct Ticket {
    subject: String,
}

impl Ticket {
    /// Elided: rule 3 hands `self`'s lifetime to the return type for free.
    fn subject(&self) -> &str {
        &self.subject
    }

    /// The exact same signature, spelled out in full.
    fn subject_explicit<'a>(&'a self) -> &'a str {
        &self.subject
    }
}

fn main() {
    let ticket = Ticket {
        subject: String::from("printer is on fire"),
    };
    println!("elided:   {}", ticket.subject());
    println!("explicit: {}", ticket.subject_explicit());
}
