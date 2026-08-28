//! Putting the two together: a Vec of Strings.
//!
//!     cargo run -p p1-01-06-vec-and-string-basics --example 03-collecting-lines

fn main() {
    let mut lines: Vec<String> = Vec::new();
    lines.push(String::from("alpha"));
    lines.push(String::from("beta"));
    lines.push(String::from("gamma"));

    println!("lines:     {lines:?}");
    println!("count:     {}", lines.len());

    // Building one String out of many. `&lines` lends the Vec for the loop
    // rather than consuming it.
    let mut joined = String::new();
    for line in &lines {
        joined.push_str(line);
        joined.push(' ');
    }
    println!("joined:    {joined:?}");

    // Note the trailing space. Getting rid of it is the sort of thing
    // `.join()` exists for, and 1.4.3 covers it properly.

    // Longest by byte length. Careful: byte length, not character count.
    let mut longest = 0;
    for line in &lines {
        if line.len() > longest {
            longest = line.len();
        }
    }
    println!("longest:   {longest}");

    // Vec has the same emptiness question as everything else.
    let empty: Vec<String> = Vec::new();
    println!("empty?     {}", empty.is_empty());
    println!("last:      {:?}", lines.last());
    println!("last:      {:?}", empty.last());
}
