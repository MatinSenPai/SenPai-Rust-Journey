//! The combinators that mutate an `Option` in place, or let you look inside
//! one without taking ownership of what's there.

fn main() {
    // .take() — remove the value, leaving `None` behind.
    let mut slot: Option<String> = Some("draft".to_string());
    let taken = slot.take();
    println!(".take()     taken={taken:?} slot={slot:?}");

    // .replace() — put a new value in, handing back whatever was there.
    let mut counter: Option<u32> = Some(1);
    let previous = counter.replace(2);
    println!(".replace(2) previous={previous:?} counter={counter:?}");

    // .as_ref() — look inside without moving the String out.
    let owned: Option<String> = Some("hello".to_string());
    let length: Option<usize> = owned.as_ref().map(|s| s.len());
    println!(".as_ref()   length={length:?} owned still usable: {owned:?}");

    // .as_mut() — a mutable look inside, to edit in place.
    let mut editable: Option<String> = Some("hi".to_string());
    if let Some(s) = editable.as_mut() {
        s.push('!');
    }
    println!(".as_mut()   editable={editable:?}");

    // .cloned() / .copied() — turn a reference-shaped Option into an owned one.
    let borrowed: Option<&String> = owned.as_ref();
    let cloned: Option<String> = borrowed.cloned();
    println!(".cloned()   cloned={cloned:?}");

    let numbers = vec![1, 2, 3];
    let first_ref: Option<&i32> = numbers.first();
    let first_val: Option<i32> = first_ref.copied();
    println!(".copied()   first_val={first_val:?}");

    // .zip() — combine two Options into one pair, only if both are Some.
    let x: Option<i32> = Some(3);
    let y: Option<i32> = Some(4);
    println!(".zip() both present: {:?}", x.zip(y));
    println!(".zip() one missing:  {:?}", x.zip(None::<i32>));

    // .or() / .or_else() — fall back to a different Option entirely.
    let primary: Option<i32> = None;
    println!(".or()      {:?}", primary.or(Some(99)));
    println!(".or_else() {:?}", primary.or_else(|| Some(100)));

    // .filter() — keep the value only if a predicate holds. The closure gets
    // a *reference* to the value, not the value itself.
    let n: Option<i32> = Some(8);
    println!(".filter() even: {:?}", n.filter(|v| *v % 2 == 0));
    println!(".filter() odd:  {:?}", n.filter(|v| *v % 2 != 0));

    // .is_some_and() — a predicate check without unwrapping first.
    println!(".is_some_and() > 5: {}", n.is_some_and(|v| v > 5));

    // .ok_or() / .ok_or_else() — the bridge to Result. Their full story is
    // 1.6.3; for now just see the shape: an absent value becomes a named
    // error instead of silence. They come in the same eager/lazy pair as
    // .unwrap_or() / .unwrap_or_else() above.
    let bridged = n.ok_or("missing");
    println!(".ok_or(\"missing\"):        {bridged:?}");
    let bridged_none = None::<i32>.ok_or("missing");
    println!(".ok_or(\"missing\") on None: {bridged_none:?}");
    let bridged_lazy = None::<i32>.ok_or_else(|| "missing".to_string());
    println!(".ok_or_else() on None:     {bridged_lazy:?}");
}
