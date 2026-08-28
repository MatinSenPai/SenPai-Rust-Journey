# Solution — 2.3.1 Defining and implementing traits

## `AnimeSeries`

```rust
impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title(), self.episodes)
    }
}
```

`title` is just a clone of the field — there's nothing else to return. `summary` overrides the default to also include the episode count, and to get the title it calls the `title()` method rather than reading the field directly (`self.title`) — the same pattern the trait's own default body uses.

## `MangaVolume`

```rust
impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }
}
```

That's it. No `summary` override, because none was needed. Calling `summary` on a `MangaVolume` runs the default body written once inside the trait itself — which calls `title()`, and this time it's `MangaVolume`'s version that answers. Nothing here is copied or regenerated; it's the same one body, for every type that doesn't override it.

## `GameTitle`

```rust
impl Summarize for GameTitle {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {}h to beat", self.title(), self.hours_to_beat)
    }
}
```

The exact same shape as `AnimeSeries` — a required `title` override, an optional `summary` override with its own format. Worth noticing: `GameTitle` has nothing to do with `AnimeSeries` or `MangaVolume` — no shared field, no base struct — and it still fulfills the same `Summarize` contract without any extra effort.

## `shelf_summary`

```rust
pub fn shelf_summary(series: &AnimeSeries, volume: &MangaVolume) -> String {
    format!("{}; {}", series.summary(), volume.summary())
}
```

This function is not generic at all — it takes two *concrete*, *different* types as parameters, calls `summary()` on each one, and joins the two strings with `"; "`. No generics or `dyn` were needed, because we already knew the first parameter would always be an `AnimeSeries` and the second always a `MangaVolume`. If you wanted the same idea to generalize to *any* combination of types that have `Summarize` — one function instead of a separate version for every pair of types — that is exactly what [2.3.2](../../02-generic-functions-and-structs/README.md) teaches you.
