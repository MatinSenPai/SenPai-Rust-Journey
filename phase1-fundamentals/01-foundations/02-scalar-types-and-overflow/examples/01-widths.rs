//! What the numeric types actually are, and where each one stops.
//!
//!     cargo run -p p1-01-02-scalar-types-and-overflow --example 01-widths

fn main() {
    println!("u8    {:>22} .. {}", u8::MIN, u8::MAX);
    println!("i8    {:>22} .. {}", i8::MIN, i8::MAX);
    println!("u32   {:>22} .. {}", u32::MIN, u32::MAX);
    println!("i32   {:>22} .. {}", i32::MIN, i32::MAX);
    println!("u64   {:>22} .. {}", u64::MIN, u64::MAX);
    println!("usize {:>22} .. {}", usize::MIN, usize::MAX);
    println!();
    println!("usize is {} bits on this machine", usize::BITS);
    println!();
    println!("f64 can hold {} exactly", 9007199254740992_i64);
    println!("bool is {} byte", size_of::<bool>());
    println!(
        "char is {} bytes — it holds one Unicode scalar, not one byte",
        size_of::<char>()
    );
}
