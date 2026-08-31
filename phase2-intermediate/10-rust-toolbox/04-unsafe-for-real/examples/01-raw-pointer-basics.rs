//! Creating a raw pointer is always safe. Dereferencing one is the one
//! moment that needs `unsafe` — and once you're in, raw pointers don't obey
//! the aliasing rule the way `&mut T` does.

fn main() {
    // Creating a raw pointer: no `unsafe` needed.
    let mut value = 10;
    let ptr: *mut i32 = &mut value;

    // Dereferencing one: this is operation #1 from the list, and the only
    // reason `unsafe` shows up here at all.
    let doubled = unsafe { *ptr * 2 };
    println!("doubled: {doubled}");

    // Two `*mut i32` pointing at the very same slot, at the same time — the
    // compiler lets this compile. `&mut i32` twice over would not.
    let mut data = [1, 2, 3];
    let a: *mut i32 = &mut data[0];
    let b: *mut i32 = &mut data[0];
    unsafe {
        *a += 10;
        *b += 100;
    }
    println!("aliased writes: {data:?}");

    // Pointer arithmetic is safe to *compute* — `.add(n)` moves the pointer
    // `n` elements forward. Only the dereference that follows needs `unsafe`.
    let mut more = [1, 2, 3, 4, 5];
    let first = more.as_mut_ptr();
    unsafe {
        *first *= 10;
        *first.add(4) *= 10;
    }
    println!("first and last, scaled: {more:?}");
}
