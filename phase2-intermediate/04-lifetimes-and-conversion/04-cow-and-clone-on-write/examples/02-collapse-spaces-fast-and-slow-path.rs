//! The motivating shape for `Cow`: a function that usually does not need to
//! modify its input, but sometimes does. Comparing pointer addresses proves,
//! rather than just asserts, when an allocation actually happened.

use std::borrow::Cow;

fn collapse_spaces(input: &str) -> Cow<'_, str> {
    if !input.contains("  ") {
        return Cow::Borrowed(input);
    }
    let mut collapsed = String::with_capacity(input.len());
    let mut previous_was_space = false;
    for ch in input.chars() {
        if ch == ' ' && previous_was_space {
            continue;
        }
        previous_was_space = ch == ' ';
        collapsed.push(ch);
    }
    Cow::Owned(collapsed)
}

fn main() {
    let clean = "Trigun: 26 episodes";
    let messy = "Trigun:    26   episodes";

    let clean_result = collapse_spaces(clean);
    let messy_result = collapse_spaces(messy);

    println!("clean  in:  {clean:?}");
    println!("clean out:  {clean_result:?}");
    println!(
        "clean same address as input? {}",
        clean_result.as_ptr() == clean.as_ptr()
    );
    println!();
    println!("messy  in:  {messy:?}");
    println!("messy out:  {messy_result:?}");
    println!(
        "messy same address as input? {}",
        messy_result.as_ptr() == messy.as_ptr()
    );
}
