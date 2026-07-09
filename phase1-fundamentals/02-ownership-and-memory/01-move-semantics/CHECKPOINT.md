# Checkpoint

1. Uncomment the `moved_value_demo` block, run `cargo check -p
   p1-02-01-move-semantics`. Paste the exact error code (e.g. `E0382`) and
   explain in your own words what it's telling you. Re-comment the block
   when done.
2. `reclaim_and_extend` takes `s: String` and returns `String`. Why does it
   need to return `s` at all — what would happen to the caller's ability to
   use their string if this function just took `s` and returned `()`
   instead?
3. `total_length` takes `Vec<String>` (not `&Vec<String>`). After calling
   `total_length(strings)`, is `strings` still usable in the caller? Why did
   the exercise ask for `Vec<String>` here rather than something that lets
   the caller keep using it?
4. In Python, is there any equivalent to "this variable is no longer valid
   after this line" being enforced by the language itself, rather than just
   a bug you might introduce (e.g. using a variable after `del x`)?
