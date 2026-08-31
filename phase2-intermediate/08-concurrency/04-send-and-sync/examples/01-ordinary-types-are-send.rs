//! Most types are already `Send` — nothing to opt into, nothing to write.

use std::thread;

fn main() {
    let name = String::from("senpai");
    let numbers = vec![1, 2, 3, 4, 5];

    let handle = thread::spawn(move || {
        let total: i32 = numbers.iter().sum();
        format!("{name} counted a total of {total}")
    });

    println!("{}", handle.join().unwrap());
}
