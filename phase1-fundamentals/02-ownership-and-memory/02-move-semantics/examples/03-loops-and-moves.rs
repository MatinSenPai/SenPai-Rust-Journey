//! The `&` from 1.1.6, explained.
//!
//!     cargo run -p p1-02-02-move-semantics --example 03-loops-and-moves

fn main() {
    let lines = vec![
        String::from("alpha"),
        String::from("beta"),
        String::from("gamma"),
    ];

    // Lending. The loop gets a look at each element; `lines` stays ours.
    let mut total = 0;
    for line in &lines {
        total += line.len();
    }
    println!("total bytes: {total}");
    println!("still here:  {lines:?}");

    // Giving. This loop takes the Vec, and after it, `lines` is gone.
    let mut longest = 0;
    for line in lines {
        if line.len() > longest {
            longest = line.len();
        }
    }
    println!("longest:     {longest}");

    // println!("{lines:?}");   // <- E0382: `lines` moved into the loop above

    // So which do you write? Lend unless you need to consume. The consuming
    // form is right when you genuinely want the elements themselves — here it
    // is not, because a length is all that was wanted.

    let numbers = vec![1, 2, 3];
    let mut sum = 0;
    for n in numbers {
        sum += n;
    }
    // `numbers` is gone even though `i32` copies: the Vec moved, not the i32s.
    println!("sum:         {sum}");
}
