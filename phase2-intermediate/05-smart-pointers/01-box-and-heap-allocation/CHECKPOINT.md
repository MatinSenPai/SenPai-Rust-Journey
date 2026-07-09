# Checkpoint

1. Why does `enum List { Cons(i32, List), Nil }` (no `Box`) fail to compile
   with `error[E0072]: recursive type has infinite size`? Walk through, in
   your own words, why the compiler can't assign this type a fixed size.
2. `Box<List>` is always 8 bytes on a 64-bit machine, no matter how many
   elements are in the list. Why doesn't the size of the `Box` depend on
   the size of what it points to?
3. `sum` is defined as `pub fn sum(&self) -> i32`. Why does it take `&self`
   rather than `self` (by value)? What would change about how callers can
   use a `List` after calling `.sum()` on it if it consumed `self` instead?
4. `rest.sum()` inside the `Cons` match arm is called directly on
   `rest: &Box<List>`, with no `(*rest).sum()` or similar. What mechanism
   makes that work?
5. Where else, besides recursive types, might you reach for `Box<T>`? (The
   README mentions one other major case — name it and explain briefly why
   it has the same "unknowable size at compile time" problem.)
