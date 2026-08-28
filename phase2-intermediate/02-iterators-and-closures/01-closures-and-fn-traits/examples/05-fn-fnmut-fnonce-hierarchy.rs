//! What a closure's body does with its captures decides which trait(s) it
//! implements. This is inferred, never written by hand.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 05-fn-fnmut-fnonce-hierarchy

fn main() {
    // Only reads `greeting` -> `Fn`. Callable any number of times.
    let greeting = String::from("hi");
    let read_only = || println!("read-only: {greeting}");
    read_only();
    read_only();

    // Mutates captured `hits` -> `FnMut` (and, since every `FnMut` is also
    // a valid `FnOnce`, it implements that too -- just not the stronger `Fn`).
    let mut hits = 0;
    let mut count_calls = || {
        hits += 1;
        println!("mutate: called {hits} time(s)");
    };
    count_calls();
    count_calls();

    // Moves `payload` out of its own captures -> `FnOnce` only. This one
    // cannot be called a second time; try it and see 12-fnonce-closure-called-twice.rs.
    let payload = String::from("payload");
    let consume = move || payload;
    let taken = consume();
    println!("consume: took ownership of {taken}");
}
