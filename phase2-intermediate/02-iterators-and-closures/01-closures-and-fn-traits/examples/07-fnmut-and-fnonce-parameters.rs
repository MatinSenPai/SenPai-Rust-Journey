//! `FnMut` and `FnOnce` show up in signatures for different reasons than
//! `Fn`: `FnMut` when the function may call the closure many times and the
//! closure needs to change something on each call; `FnOnce` when the
//! function only ever calls it once and lets it give up ownership of
//! whatever it captured.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 07-fnmut-and-fnonce-parameters

// May call `f` many times, and a realistic `f` wants to mutate something
// (a counter, a log) on every call -- so this needs `FnMut`, not `Fn`.
fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    for _ in 0..n {
        f();
    }
}

// Calls `f` exactly once and hands back whatever it produced -- so it only
// ever needs the weakest guarantee, `FnOnce`. This also accepts closures
// that are `Fn` or `FnMut`, since both of those satisfy `FnOnce` too.
fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

fn main() {
    let mut log = Vec::new();
    call_n_times(|| log.push("tick"), 3);
    println!("{log:?}");

    let owned = String::from("payload");
    let result = run_once(move || owned);
    println!("{result}");
}
