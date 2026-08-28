//! Not everything moves. The rule is about who is responsible for a buffer.
//!
//!     cargo run -p p1-02-02-move-semantics --example 02-what-moves

fn main() {
    // Numbers do not move. `a` is still perfectly usable afterwards, because
    // there is no heap buffer for anyone to be responsible for — copying the
    // four bytes copies the entire value.
    let a = 5_i32;
    let b = a;
    println!("a: {a}, b: {b}");

    // Same for the rest of the simple types.
    let flag = true;
    let flag_again = flag;
    let letter = 'س';
    let letter_again = letter;
    let ratio = 1.5_f64;
    let ratio_again = ratio;
    println!("bool:  {flag} {flag_again}");
    println!("char:  {letter} {letter_again}");
    println!("f64:   {ratio} {ratio_again}");

    // And for arrays and tuples, as long as everything inside behaves the
    // same way.
    let readings = [1, 2, 3];
    let readings_again = readings;
    println!("array: {readings:?} {readings_again:?}");

    let pair = (1_i32, true);
    let pair_again = pair;
    println!("tuple: {pair:?} {pair_again:?}");

    // But a tuple with a String inside it does move, because now there is a
    // buffer to be responsible for.
    let labelled = (1_i32, String::from("owned"));
    let labelled_again = labelled;
    println!("mixed: {labelled_again:?}");
    // println!("{labelled:?}");   // <- E0382

    // The dividing line: does this value own something on the heap?
    println!();
    println!("copied:  i32, u8, bool, char, f64, &T, and arrays/tuples of those");
    println!("moved:   String, Vec<T>, and anything containing one");
}
