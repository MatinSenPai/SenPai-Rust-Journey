//! Three loops, and why Rust has three.
//!
//!     cargo run -p p1-01-05-control-flow --example 02-loops

fn main() {
    // `for` over a range. `0..5` includes 0 and excludes 5; `0..=5` includes
    // both. This is the loop you will write most often.
    print!("0..5:      ");
    for n in 0..5 {
        print!("{n} ");
    }
    println!();

    print!("0..=5:     ");
    for n in 0..=5 {
        print!("{n} ");
    }
    println!();

    // `for` over an array. No index, no length, no chance of an off-by-one.
    let readings = [12, 7, 19, 3, 14];
    let mut total = 0;
    for reading in readings {
        total += reading;
    }
    println!("total:     {total}");

    // `while` runs as long as its condition holds. Use it when the number of
    // turns is not known up front.
    let mut remaining = 100;
    let mut halvings = 0;
    while remaining > 1 {
        remaining /= 2;
        halvings += 1;
    }
    println!("halvings:  {halvings}");

    // `loop` runs forever until something breaks out of it. Unlike
    // `while true`, the compiler knows it never falls through on its own.
    let mut attempt = 0;
    let outcome = loop {
        attempt += 1;
        if attempt * attempt > 50 {
            // `break` can carry a value out, which makes `loop` an
            // expression like everything else.
            break attempt;
        }
    };
    println!("outcome:   {outcome}");

    // `continue` skips the rest of this turn.
    print!("odds:      ");
    for n in 0..10 {
        if n % 2 == 0 {
            continue;
        }
        print!("{n} ");
    }
    println!();

    // A label lets `break` leave an outer loop rather than the nearest one.
    let grid = [[1, 5, 9], [2, 6, 10], [3, 7, 11]];
    let mut found = (0, 0);
    'search: for row_index in 0..grid.len() {
        for column_index in 0..grid[row_index].len() {
            if grid[row_index][column_index] > 6 {
                found = (row_index, column_index);
                break 'search;
            }
        }
    }
    println!("found at:  {found:?}");
}
