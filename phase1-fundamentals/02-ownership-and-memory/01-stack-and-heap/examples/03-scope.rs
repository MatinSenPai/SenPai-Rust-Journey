//! A value lives until the closing brace of the block that owns it.
//!
//!     cargo run -p p1-02-01-stack-and-heap --example 03-scope

fn main() {
    let outer = String::from("I last until the end of main");
    println!("start:      {outer}");

    {
        // A new scope. Anything declared here belongs to it.
        let inner = String::from("I last until the next brace");
        println!("inside:     {inner}");
        println!("also here:  {outer}");
    } // <- `inner` ends here. Its heap buffer is released, right now, with
      //    no garbage collector involved and nothing for you to write.

    println!("after:      {outer}");

    // Try uncommenting this. See 04-out-of-scope for the error.
    // println!("{inner}");

    // Scopes nest, and so do lifetimes. Each closing brace releases whatever
    // that block owned, in reverse order of declaration.
    {
        let first = String::from("declared first");
        let second = String::from("declared second");
        println!("nested:     {first} / {second}");
        {
            let deepest = String::from("deepest");
            println!("deeper:     {deepest}");
        }
        println!("back out:   {first}");
    }

    println!("end:        {outer}");
}
