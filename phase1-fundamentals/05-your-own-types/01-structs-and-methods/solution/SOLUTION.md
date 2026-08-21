# Solution

`Self` and `Book` are identical here — `Self` is just shorthand for "the
type this `impl` block is for," so `-> Self` and `-> Book` compile to the
same thing inside `impl Book { ... }`. The convention is to prefer `Self`:
if you ever rename `Book`, every method's signature stays correct with no
edits.

`self.chapters_read as f64` is Rust's explicit numeric cast syntax — there's
no implicit int-to-float promotion like some languages have; you always say
what you mean.

`describe` builds `star` first as a plain `&str` (`" *"` or `""`), then
uses it inside one `format!` call — simpler than branching on the whole
format string.

On recall question 2: if `describe` took `&mut self` instead of
`&self`, calling it would require an *exclusive* borrow of the `Book` even
though it never writes to it — meaning you couldn't, for example, call
`describe()` on two books at once, or hold a shared reference to a book
elsewhere while calling `describe()` on it. Taking `&self` when you only
read is what lets callers keep using shared access freely; it's the same
"aliasing XOR mutability" idea from the borrowing module, just applied
automatically to whatever `self` a method is called on.
