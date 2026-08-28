//! `Option<T>` is an ordinary enum — the same shape as any enum from 1.5.3 —
//! with two variants: one that carries a value, one that carries nothing.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 01-some-and-none

fn main() {
    let recorded: Option<u32> = Some(25);
    let missing: Option<u32> = None;

    // `{:?}` — Debug — is the only way to print an Option directly. There is
    // no Display impl, because "display this as text" has no sane answer
    // for None.
    println!("recorded: {recorded:?}");
    println!("missing:  {missing:?}");

    // You can ask an Option what it is without opening it.
    println!();
    println!("recorded.is_some(): {}", recorded.is_some());
    println!("recorded.is_none(): {}", recorded.is_none());
    println!("missing.is_some():  {}", missing.is_some());
    println!("missing.is_none():  {}", missing.is_none());

    // Two Options of the same T compare with ==, variant and payload both.
    println!();
    println!("Some(25) == recorded: {}", Some(25) == recorded);
    println!("None == missing:      {}", None == missing);
    println!("recorded == missing:  {}", recorded == missing);
}
