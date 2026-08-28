# 05.1 — `Box` and heap allocation

## Stack vs. heap, for real this time

You've been told informally that Rust has "the stack" and "the heap," but
this lesson is where it actually matters. Two memory regions, two very
different rule sets:

- **The stack** is a fixed-size region that grows and shrinks in strict
  last-in-first-out order, exactly like a stack of plates. Pushing and
  popping is just moving a pointer — extremely fast. The catch: to put a
  value on the stack, the compiler must know **exactly how many bytes it
  takes up, at compile time**, before the program ever runs.
- **The heap** is a much larger, less structured region. You explicitly
  request ("allocate") a chunk of a given size at run time, get back a
  pointer to it, and that memory stays reserved until something explicitly
  frees it. Slower than the stack (the allocator has to find free space),
  but it can hold values whose size isn't known until run time, or values
  that need to outlive the function that created them.

Every value you've used so far — `i32`, `bool`, even a `Book` struct with a
`String` field — lives on the stack by default. (The `String`'s *character
data* is actually on the heap already — that's how it can grow — but the
`String` value itself, the little struct holding a pointer/length/capacity,
sits on the stack. `Box` is about explicitly putting a whole value on the
heap, not about strings specifically.)

## `Box<T>`: the simplest smart pointer

```rust
let boxed: Box<i32> = Box::new(5);
println!("{}", *boxed); // 5 — deref to read the value inside
```

A "smart pointer" is a struct that acts like a pointer (it lets you get at
some data) but also owns what it points to and carries extra behavior.
`Box<T>` is the simplest one: it's just "a `T`, heap-allocated, with a
plain pointer to it stored on the stack." When the `Box` goes out of scope,
Rust frees the heap memory automatically — same ownership rules as
everything else, just with one extra layer of indirection.

You almost never need to write `*boxed` to use what's inside, though.
`Box<T>` implements `Deref` and `DerefMut`, which means Rust will
automatically follow the pointer for you when you call a method: if `T` has
a method `some_method(&self)`, then `boxed.some_method()` just works,
no manual dereferencing needed. This is the same mechanism that lets
`String` methods work directly — it's not special-cased to `Box`, it's a
general trait any pointer-like type can implement.

## Why would you ever need this? Recursive types

This is the motivating case, and it comes up constantly once you start
building trees, linked lists, or ASTs (abstract syntax trees, e.g. for a
parser). Consider a classic "cons list" (a linked list built from nested
pairs, the name comes from Lisp):

```rust
enum List {
    Cons(i32, List), // <-- won't compile!
    Nil,
}
```

Try to compile this and you get `error[E0072]: recursive type has infinite
size`. Why? To lay out `List` in memory, the compiler needs to know its
size — and `List`'s size depends on the size of its `Cons` variant, which
contains *another* `List`, whose size depends on *its* `Cons` variant's
`List`... it never bottoms out. There's no finite number of bytes that
could hold "a `List`," because a `List` might contain another `List` might
contain another, arbitrarily deep.

The fix: put the recursive field behind a `Box`.

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

Now `Cons` holds an `i32` plus a `Box<List>` — and a `Box<List>`, no matter
what it points to or how deep the list gets, is *always* exactly one
pointer wide (8 bytes on a 64-bit machine). The compiler can lay out `List`
with a known, fixed size, because the recursion happens through a heap
allocation rather than by value. This is the canonical reason `Box`
exists: it turns "infinitely deep, unknowable size" into "one pointer,
known size," at the cost of a heap allocation per link.

## The other major use: `Box<dyn Trait>`

If you've reached module 3 (generics and traits) already, you've seen that
a `dyn Trait` (a "trait object") also has an unknowable size at compile
time — different concrete types implementing the same trait can be wildly
different sizes. `Box<dyn Trait>` solves that exactly the same way: heap
allocate it, keep a fixed-size pointer on the stack. You'll see this
pattern again anywhere you need "some type that implements this trait, I
don't care which one, decided at run time" — e.g. a `Vec<Box<dyn Shape>>`
holding a mix of circles and squares.

## Your task

Implement `List::sum` and `from_vec` in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
