//! Every way you'll format a line for the next few phases.
//!
//!     cargo run -p p0-03-hello-rust --example 02-formatting

fn main() {
    let language = "Rust";
    let day = 1;

    // A `{}` hole, filled by the argument that follows.
    println!("Day {} of learning {}", day, language);

    // The same thing, naming the variable inside the hole. Shorter, and it
    // stays readable when there are four of them.
    println!("Day {day} of learning {language}");

    // Width and alignment: pad to 8 characters. Useful for lining up output.
    println!("[{:>8}] right", language);
    println!("[{:<8}] left", language);

    // `{:?}` asks for the *debug* view rather than the display view. You'll
    // meet the difference properly in Phase 2.
    println!("debug view: {:?}", language);
}
