//! Three situations, three different right answers.
//!
//!     cargo run -p p1-02-03-clone-and-copy --example 03-when-to-clone

fn main() {
    // 1. You only need to read it. Borrow. No clone, no allocation.
    let name = String::from("Matin");
    println!("length:     {}", length_of(&name));
    println!("still ours: {name}");

    // 2. You need to keep one and give one away. Clone once, deliberately.
    let template = String::from("report-");
    let for_them = template.clone();
    println!("kept:       {template}");
    println!("given:      {for_them}");

    // 3. You are finished with it. Move it. No clone at all.
    let finished = String::from("done");
    let consumed = consume(finished);
    println!("consumed:   {consumed}");

    // And the mistake to recognise: cloning inside a loop when borrowing
    // would do. Each turn of the first loop allocates and then throws the
    // allocation away.
    let lines = vec![
        String::from("alpha"),
        String::from("beta"),
        String::from("gamma"),
    ];

    let mut wasteful = 0;
    for line in &lines {
        let copy = line.clone(); // allocates
        wasteful += copy.len();
    } // and frees, having learnt nothing a borrow could not have told us

    let mut frugal = 0;
    for line in &lines {
        frugal += line.len();
    }

    println!();
    println!("same answer: {wasteful} / {frugal}");
    println!("first version allocated 3 times for nothing");
}

fn length_of(text: &str) -> usize {
    text.len()
}

fn consume(text: String) -> usize {
    text.len()
}
