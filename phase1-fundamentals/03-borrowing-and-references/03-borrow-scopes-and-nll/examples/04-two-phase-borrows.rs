//! 1.3.3 — three method calls that look like they break the aliasing rule
//! and do not.
//!
//! Each one hands `&mut` of a value to a method while the argument is still
//! reading that same value.

fn main() {
    let mut items = vec![10, 20, 30];
    items.push(items.len());
    println!("push(items.len()):      {items:?}");

    let mut names = vec![String::from("Matin"), String::from("Sora")];
    names.push(names[0].clone());
    println!("push(names[0].clone()): {names:?}");

    let mut text = String::from("hi");
    text.push_str(&text.len().to_string());
    println!("push_str(own length):   {text}");
}
