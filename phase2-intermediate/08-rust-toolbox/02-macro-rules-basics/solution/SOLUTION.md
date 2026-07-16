# Solution

```rust
#[macro_export]
macro_rules! string_map {
    () => {
        ::std::collections::HashMap::new()
    };
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key.to_string(), $value.to_string()); )+
        map
    }};
}
```

Why two arms? A single `$( ... ),*` arm would also match the empty case —
but then the empty expansion would still be `let mut map = ...; map`, and
`mut` on a never-mutated variable is a warning. Splitting the base case
out keeps both expansions warning-free. The pairs arm shows the two
idioms you'll reuse forever: the `{{ ... }}` double brace (outer pair
closes the macro arm, inner pair is a block expression, so the expansion
can contain statements *and* produce a value), and `$( ... )+` in the
transcriber replaying the captures the matcher collected. Also notice the
`=>` in the matcher: that token is exactly why this can't be a function —
`"k" => "v"` isn't a Rust expression, so only a macro can accept the
syntax. Paths are written `::std::collections::HashMap` (fully qualified,
leading `::`) so the macro works even if the call site never imported
`HashMap` — a macro expands *at the call site* and sees the call site's
imports, not yours.

```rust
#[macro_export]
macro_rules! max_of {
    ( $only:expr ) => { $only };
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        ::std::cmp::max($first, $crate::max_of!( $( $rest ),+ ))
    };
}
```

Classic recursive shape. `max_of!(3, 9, 7)` matches the second arm with
`$first = 3`, `$rest = 9, 7`, expanding to `max(3, max_of!(9, 7))`; one
more round gives `max(3, max(9, max_of!(7)))`; now the single expression
matches the *first* arm (arms are tried top to bottom) and the recursion
bottoms out at `max(3, max(9, 7))`. Delete the base case and the
recursion has nowhere to stop — the compiler kills it with a "recursion
limit reached" error rather than looping forever. `$crate::max_of!` is
the hygiene-adjacent detail: `$crate` always names the crate that
*defined* the macro, so the recursive call resolves correctly even when
someone else's crate invokes `max_of!` without importing it under that
exact name.

```rust
#[macro_export]
macro_rules! timed {
    ( $work:expr ) => {{
        let start = ::std::time::Instant::now();
        let result = $work;
        (result, start.elapsed())
    }};
}
```

Two deliberate details. First, `let result = $work;` pastes the caller's
expression exactly **once** — the tempting shortcut `($work,
start.elapsed())` also uses it once, but any design that pastes `$work`
twice (say, logging it and returning it) would re-run the caller's side
effects; the counter test (`calls += 1` inside the block) exists to catch
exactly that class of bug. Second, hygiene in action: the transcriber's
`start` cannot collide with a caller's variable named `start` —
`timed!(start + 1)` at a call site where `start` is an integer works
fine, because the macro-internal `start` is a different name to the
compiler. That's the guarantee C's `#define` never had.

One closing honesty note: of these three, only `string_map!` and
`max_of!` *require* macros (novel syntax, variadic arity). `timed!` could
have been `fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration)` with a
closure at the call site — `timed(|| work())`. The macro buys slightly
nicer call-site syntax at the cost of everything the README warned about.
In real code, prefer the function; here it earns its place by teaching
the wrap-an-expression pattern you'll recognize in crates like `tracing`.
