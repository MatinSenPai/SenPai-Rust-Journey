# Solution

```rust
pub fn drop_order_in_one_scope() -> Vec<String> {
    take_log();
    {
        let _a = Tracker::new("a");
        let _b = Tracker::new("b");
        let _c = Tracker::new("c");
    }
    take_log()
}
```

`a`, `b`, `c` are declared in that order, but the log comes back
`["c", "b", "a"]` — **reverse** declaration order. This isn't arbitrary:
later-declared values may (in general) depend on earlier ones being still
alive (e.g. holding a reference into them — Phase 1's next module), so
Rust must tear things down in the opposite order it built them up, the same
way you'd close nested parentheses from the inside out.

```rust
pub fn early_drop_demo() -> Vec<String> {
    take_log();
    {
        let first = Tracker::new("first");
        let _second = Tracker::new("second"); // dropped naturally at block end
        drop(first);
    }
    take_log()
}
```

`drop(first)` forces `first` to clean up immediately, out of the normal
end-of-scope order — that's why the log comes back `["first", "second"]`
instead of the reverse-order `["second", "first"]` you'd get without the
explicit `drop`. `std::mem::drop` isn't magic; it's just an ordinary
function that takes ownership of its argument and does nothing with it —
the value gets dropped simply because that function's (trivial) scope
immediately ends.

```rust
fn create_tracker(name: &str) -> Tracker {
    Tracker::new(name)
}

pub fn move_extends_lifetime() -> Vec<String> {
    take_log();
    {
        let _t = create_tracker("moved");
    }
    take_log()
}
```

The log only ever contains one `"moved"` entry, logged once, at the end of
the *outer* block — not inside `create_tracker`. `create_tracker` builds
the `Tracker` and immediately returns it, moving ownership to the caller;
since the value's owner never changes location-in-scope until the outer
`{ ... }` ends, that's where — and only where — it drops.
