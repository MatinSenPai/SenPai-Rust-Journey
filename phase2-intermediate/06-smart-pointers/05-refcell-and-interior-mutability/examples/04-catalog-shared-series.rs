//! `RefCell` does not have to wrap the whole value — here it wraps just the
//! one field that needs to change, and `Rc` shares the whole `Series`.

use std::cell::RefCell;
use std::rc::Rc;

struct Series {
    title: String,
    episodes_watched: RefCell<u32>,
}

fn main() {
    let frieren = Rc::new(Series {
        title: String::from("Frieren"),
        episodes_watched: RefCell::new(0),
    });
    let catalog: Vec<Rc<Series>> = vec![Rc::clone(&frieren)];
    let now_watching = Rc::clone(&frieren);

    *now_watching.episodes_watched.borrow_mut() += 1;
    *catalog[0].episodes_watched.borrow_mut() += 1;

    println!(
        "{}: {} episodes watched ({} owners share it)",
        frieren.title,
        frieren.episodes_watched.borrow(),
        Rc::strong_count(&frieren)
    );
}
