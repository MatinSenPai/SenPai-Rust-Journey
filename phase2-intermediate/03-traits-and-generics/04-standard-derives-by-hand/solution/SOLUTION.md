# Solution — 2.3.4 The standard derives, implemented by hand

```rust
impl fmt::Debug for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Track")
            .field("title", &self.title)
            .field("artist", &self.artist)
            .field("seconds", &self.seconds)
            .finish()
    }
}

impl fmt::Display for Track {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} by {}", self.title, self.artist)
    }
}

impl Default for Track {
    fn default() -> Self {
        Track { title: String::new(), artist: String::new(), seconds: 0 }
    }
}

impl PartialEq for Track {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.artist == other.artist
    }
}

impl Ord for Track {
    fn cmp(&self, other: &Self) -> Ordering {
        self.seconds.cmp(&other.seconds).then_with(|| self.title.cmp(&other.title))
    }
}

impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.artist.hash(state);
    }
}
```

None of these six pieces needed anything beyond the tools "The concept" already showed you — just aimed at your own type this time.

## `Debug` — exactly the builder example 01 used

```rust
f.debug_struct("Track")
    .field("title", &self.title)
    .field("artist", &self.artist)
    .field("seconds", &self.seconds)
    .finish()
```

The spec asked for exactly the shape `#[derive(Debug)]` produces — type name, then fields in declaration order. Reaching for `debug_struct` instead of a raw `write!` is exactly what the second test, `debug_alternate_form_is_indented`, depends on: without the builder, `{:#?}` would never have gotten its indentation.

## `Display` — one `write!`, no quotes

```rust
write!(f, "{} by {}", self.title, self.artist)
```

The spec asked for exactly this string: title, the word "by", artist — no quotes, nothing extra. `seconds` never shows up here at all; `Display` is for the listener, not the debugger.

## `Default` — three fields, three blank values

```rust
Track { title: String::new(), artist: String::new(), seconds: 0 }
```

Nothing subtle here — just spell out all three fields explicitly as zero/empty. `String::new()` builds an empty `String`, not an `Option` and not a panic.

## `PartialEq` — only `title` and `artist`; `seconds` never enters the picture

```rust
self.title == other.title && self.artist == other.artist
```

The spec was explicit: `seconds` is never part of equality — two recordings of the same song, even with different tracked runtimes, still count as the same track. The `impl Eq for Track {}` you were given for free makes that official: because the `PartialEq` above really is reflexive (neither `String` nor `u32` has `f64`'s `NaN` problem), `Eq`'s extra promise costs nothing to make.

## `Ord` — `seconds` first, `title` breaks the tie

```rust
self.seconds.cmp(&other.seconds).then_with(|| self.title.cmp(&other.title))
```

`.then_with(...)` is exactly what the spec asked for: "order by `seconds` first; when that ties, fall back to `title`." The closure inside `.then_with()` only runs at all when the first comparison actually came back equal — exactly the laziness needed for a tie to stay a tie until you resolve it.

## `Hash` — exactly the two fields `PartialEq` looks at

```rust
self.title.hash(state);
self.artist.hash(state);
```

This is the lesson's whole point in one function: `Hash` has to see exactly the fields `PartialEq` above sees — `title` and `artist`, not `seconds`. Add `self.seconds.hash(state);` back in, and two `Track`s that `PartialEq` considers equal (because it only checks `title`/`artist`) would get different hashes — and `hash_matches_eq_so_a_hashset_dedupes` catches exactly that: a `HashSet` that should hold one entry would hold two.

## What this lesson was really about

- **Each of the six traits carries its own contract, not one shared mechanism.** `Debug` is mechanical, `Display` is a decision, `Eq` is an extra promise on top of `PartialEq`, `Ord` carries that same promise one level up, and `Hash` always has to stay in step with `Eq`.
- **`f64` is missing `Eq`, `Ord`, and `Hash` for one single root cause:** `NaN` isn't reflexive, so its equality can't be complete, its order can't be total, and nothing built on top of either — like a hash — can honestly rely on them.
- **The compiler can only help so much.** A `Hash` that disagrees with `Eq` — exactly like this `Track` would if it also hashed `seconds` — produces no error at all. Knowing the two have to agree is the only defense you get.
