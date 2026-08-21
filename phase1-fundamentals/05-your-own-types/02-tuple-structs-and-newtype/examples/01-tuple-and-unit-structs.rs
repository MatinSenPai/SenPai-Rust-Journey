//! A struct whose fields have no names, and a struct with no fields at all.
//!
//! Run `cargo run -p p1-05-02-tuple-structs-and-newtype --example
//! 01-tuple-and-unit-structs`.

/// One field, and no name for it.
#[derive(Debug)]
struct Meters(f64);

/// Three fields, at positions 0, 1 and 2.
#[derive(Debug)]
struct Rgb(u8, u8, u8);

/// No fields at all.
#[derive(Debug)]
struct Marker;

fn main() {
    let height = Meters(1.83);
    println!("debug form:   {height:?}");
    println!("field .0:     {}", height.0);

    let orange = Rgb(255, 165, 0);
    println!("rgb:          {} {} {}", orange.0, orange.1, orange.2);
    println!("rgb debug:    {orange:?}");

    let Meters(raw) = height;
    println!("destructured: {raw}");

    let marker = Marker;
    println!("unit struct:  {marker:?}");
    println!("size of Marker: {}", size_of::<Marker>());

    println!("size of Meters: {}", size_of::<Meters>());
    println!("size of Rgb:    {}", size_of::<Rgb>());
}
