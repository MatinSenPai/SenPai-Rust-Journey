# Solution — 2.3.5 Associated types versus generic parameters

```rust
impl Measures for Rectangle {
    type Output = u32;

    fn measure(&self) -> u32 {
        self.width * self.height
    }
}

impl DescribesAs<u32> for Rectangle {
    fn describe(&self) -> u32 {
        2 * (self.width + self.height)
    }
}

impl DescribesAs<String> for Rectangle {
    fn describe(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }
}
```

None of these needed a new trait or any extra type annotation on the `impl` itself — just bodies that stayed true to the two shapes "The concept" already showed.

## `Measures::measure` — one multiplication, one fixed type

```rust
self.width * self.height
```

`type Output = u32;` was already pinned down in the trait's definition — not in the method body, not at the call site (`r.measure()`, with no type written anywhere). The body only had to return exactly what that promise said.

## `DescribesAs<u32>::describe` and `DescribesAs<String>::describe` — two bodies, two separate `impl`s

```rust
2 * (self.width + self.height)
```

```rust
format!("{}x{}", self.width, self.height)
```

These are two completely separate `impl` blocks — not one shared method branching on a condition. `Rectangle` implements both `DescribesAs<u32>` and `DescribesAs<String>` at once, exactly the way `Celsius` implemented both `Converts<f64>` and `Converts<String>` at once in "The concept." That's also exactly why each test gives its result an explicit `let` type (`let perimeter: u32 = ...`, `let label: String = ...`) — without it, you'd hit the same `E0283` "Errors you will meet" showed you.

## What this lesson was really about

- **An associated type is decided once, and applies everywhere.** `Output = u32` was written exactly once; every call to `.measure()`, on every `Rectangle`, gets that same one type back — no exceptions.
- **A generic parameter means two fully independent `impl`s.** `describe()` on `DescribesAs<u32>` and `describe()` on `DescribesAs<String>` are two entirely separate functions that just happen to share a name; the compiler decides which one gets called based on the requested type (read off the `let`'s annotation).
- **The specification was the specification.** The exact format of `"3x4"` — no spaces, `width` before `height` — was stated in the exercise's own doc comment; nothing in the tests was a guess.
