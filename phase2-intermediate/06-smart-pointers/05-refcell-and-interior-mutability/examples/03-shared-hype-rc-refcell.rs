//! The classic combo: `Rc<RefCell<T>>` — shared ownership, and every owner
//! can still mutate the one value underneath.

use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    let hype = Rc::new(RefCell::new(0));
    let a = Rc::clone(&hype); // second handle, same RefCell<i32>
    let b = Rc::clone(&hype); // third handle, same RefCell<i32>

    *a.borrow_mut() += 10;
    *b.borrow_mut() += 5;

    println!("hype via a: {}", a.borrow());
    println!("hype via b: {}", b.borrow());
    println!("owners:     {}", Rc::strong_count(&hype));
}
