//! Exercises for 2.6.1 — `Box` and heap allocation.
//!
//! The first three functions are plain mechanics: build a `Box`, pull a
//! value back out of one, mutate through a `&mut Box<T>`. The fourth
//! measures today's central claim — that `Box<T>`'s size never depends on
//! `T` — with real numbers. The fifth uses the `Playable` trait below to
//! build a `Vec` of boxed trait objects.

use std::mem::size_of;

/// Moves `value` onto the heap and returns a `Box` that owns it.
pub fn wrap(value: i32) -> Box<i32> {
    todo!("put `value` on the heap with Box::new and return the box")
}

/// Takes ownership of `boxed` and moves the `String` out of it, consuming
/// the box.
pub fn unwrap_box(boxed: Box<String>) -> String {
    todo!("move the String out of `boxed` and return it")
}

/// Adds 1 to the `i32` inside `boxed`, in place, through the box, and
/// returns the new value.
pub fn increment_boxed(boxed: &mut Box<i32>) -> i32 {
    todo!("add 1 to the value `boxed` points at, then return the new value")
}

/// Measures, with `std::mem::size_of`, the byte size of three types and
/// returns them as a tuple in this exact order: `Box<i32>`,
/// `Box<[u8; 4096]>`, `Box<Box<i32>>`. On a 64-bit machine, all three
/// numbers come out identical — that identity is the point of this
/// function.
pub fn box_size_report() -> (usize, usize, usize) {
    todo!(
        "measure and return, as a 3-tuple in that exact order, the byte size of a box around an \
         i32, a box around a 4096-byte byte array, and a box around a box around an i32"
    )
}

/// Something that can be played, and describes itself when it is.
pub trait Playable {
    fn play(&self) -> String;
}

/// A single track, identified only by its title.
pub struct Song {
    pub title: String,
}

impl Playable for Song {
    fn play(&self) -> String {
        format!("playing song: {}", self.title)
    }
}

/// An episode with a title and a running time.
pub struct Podcast {
    pub title: String,
    pub duration_minutes: u32,
}

impl Playable for Podcast {
    fn play(&self) -> String {
        format!(
            "playing podcast: {} ({} min)",
            self.title, self.duration_minutes
        )
    }
}

/// Returns a `Vec` holding one boxed `Song` (title `"Intro"`) followed by
/// one boxed `Podcast` (title `"Deep Dive"`, `duration_minutes` 42) — in
/// that order.
pub fn make_playable_list() -> Vec<Box<dyn Playable>> {
    todo!(
        "build a Vec of two boxed trait objects, in order: a Song titled Intro, then a Podcast \
         titled Deep Dive with duration_minutes 42"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_a_value_on_the_heap() {
        let boxed = wrap(5);
        assert_eq!(*boxed, 5);
    }

    #[test]
    fn unwraps_a_boxed_string() {
        assert_eq!(unwrap_box(Box::new("hi".to_string())), "hi".to_string());
        assert_eq!(unwrap_box(Box::new(String::new())), String::new());
    }

    #[test]
    fn increments_through_a_mutable_box_reference() {
        let mut n = Box::new(5);
        assert_eq!(increment_boxed(&mut n), 6);
        assert_eq!(increment_boxed(&mut n), 7);
        assert_eq!(*n, 7);
    }

    #[test]
    fn box_size_never_depends_on_what_is_inside() {
        let (small, large, nested) = box_size_report();
        assert_eq!(small, 8); // one pointer, on a 64-bit machine
        assert_eq!(large, 8);
        assert_eq!(nested, 8);
        assert_eq!(small, large);
        assert_eq!(large, nested);
    }

    #[test]
    fn builds_the_playable_list_in_order() {
        let list = make_playable_list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].play(), "playing song: Intro");
        assert_eq!(list[1].play(), "playing podcast: Deep Dive (42 min)");
    }
}
