//! DELIBERATELY BROKEN — expected: E0184
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 06-copy-and-drop --features broken

// Every field is `Copy`, so 1.2.3 says this derive is allowed. The `Drop`
// below is what makes it impossible.
#[derive(Clone, Copy)]
struct Ticket {
    id: u32,
}

impl Drop for Ticket {
    fn drop(&mut self) {
        println!("returning ticket {}", self.id);
    }
}

fn main() {
    let first = Ticket { id: 7 };
    let second = first;
    println!("{} {}", first.id, second.id);
}
