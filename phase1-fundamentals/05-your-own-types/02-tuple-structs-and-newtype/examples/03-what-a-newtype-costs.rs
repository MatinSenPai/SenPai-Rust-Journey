//! What a newtype costs at run time, and what it costs you.
//!
//! Run `cargo run -p p1-05-02-tuple-structs-and-newtype --example
//! 03-what-a-newtype-costs`.

#[derive(Debug, Clone, Copy)]
struct Meters(f64);

#[derive(Debug, Clone, Copy)]
struct Feet(f64);

#[derive(Debug, Clone, Copy)]
struct AccountId(u64);

/// The compile-time cost of a newtype: this function, which you write.
fn to_feet(distance: Meters) -> Feet {
    Feet(distance.0 * 3.280_84)
}

fn main() {
    println!("f64        {} bytes", size_of::<f64>());
    println!("Meters     {} bytes", size_of::<Meters>());
    println!("Feet       {} bytes", size_of::<Feet>());
    println!("u64        {} bytes", size_of::<u64>());
    println!("AccountId  {} bytes", size_of::<AccountId>());

    let height = Meters(1.83);
    let converted = to_feet(height);
    println!("{height:?} is {converted:?} — {} feet", converted.0);

    // The wrapper is gone by the time this runs: the addition below is the
    // same machine instruction as adding two bare f64 values.
    let total = Meters(height.0 + Meters(0.17).0);
    println!("total {total:?}");

    // Same eight bytes, four different meanings. Only one of them is checked.
    let raw = 1.83_f64;
    println!("raw f64 {raw}, wrapped {:?}", Meters(raw));
    let id = AccountId(1001);
    println!("id {id:?} holds {}, which is not the same type", id.0);
}
