# Checkpoint

1. If you remove `Copy` from `largest`'s bound (leaving just `T: PartialOrd`)
   and try to compile, Rust complains about moving out of a borrowed
   context. Where in the function body does that happen, and why does
   `Copy` fix it?
2. `Stack<T>` has exactly one struct definition in `src/lib.rs`, but the
   tests use it as both `Stack<i32>` and `Stack<String>`. At the machine-code
   level, after `cargo build`, how many distinct compiled versions of
   `Stack`'s methods actually exist? What is this process called?
3. In Python, you could write one `class Stack:` and push mixed types onto
   the same instance (`s.push(1); s.push("two")`) without the language
   objecting. Can you do that with `Stack<T>` as defined here? Why or why
   not?
4. `largest` takes `&[T]` (a slice reference), not `Vec<T>` or `&Vec<T>`.
   Given what you know from the ownership/borrowing modules, why is a
   read-only borrow the right choice for a function that only needs to look
   at the values, not keep or modify them?
