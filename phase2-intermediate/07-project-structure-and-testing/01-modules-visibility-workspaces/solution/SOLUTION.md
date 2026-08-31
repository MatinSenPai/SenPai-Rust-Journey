# Solution — 2.7.1 Modules, visibility, re-exports, workspaces

## `Anime::public_rating_band` — a public reader over a private field

```rust
pub fn public_rating_band(&self) -> &'static str {
    match self.internal_rating {
        0..=3 => "low",
        4..=7 => "medium",
        _ => "high",
    }
}
```

Three arms over a `u8` clamped to `0..=10` by construction: `0..=3`,
`4..=7`, and a catch-all `_` for `8..=10`. The compiler would refuse a
`match` on an integer type that didn't cover every possible value, which
is exactly why the last arm is a bare `_` instead of spelling out
`8..=10` — both work, `_` is just shorter once the ranges you actually
care about are already covered.

## `pricing::discount_percent` — a `pub(super)` policy, kept out of the public API

```rust
pub(super) fn discount_percent(internal_rating: u8) -> u8 {
    match internal_rating {
        0..=3 => 0,
        4..=7 => 10,
        _ => 25,
    }
}
```

Same three-band shape as `public_rating_band`, on purpose — it's the same
underlying scale, just answering a different question ("how much of a
discount" instead of "how should I describe this"). The interesting part
is what's *not* here: no `pub`. `pricing` is a submodule of `catalog`,
and this function only needs to be reachable from `catalog` itself — the
one place that actually applies the discount. `pub(super)` says exactly
that: visible to the parent module, and nowhere else. Making it `pub`
would have worked too, but it would have promised something untrue —
that discount tiers are part of this crate's stable public surface,
when they're really an implementation detail of how `Anime` prices
itself.

## `Anime::rental_price_cents` — where the two modules meet

```rust
pub fn rental_price_cents(&self, base_price_cents: u32) -> u32 {
    let discount = pricing::discount_percent(self.internal_rating) as u32;
    base_price_cents * (100 - discount) / 100
}
```

This is the only place in the whole crate that actually calls
`pricing::discount_percent` — and it can, because `rental_price_cents`
is defined inside `catalog`, `pricing`'s direct parent, exactly the one
module `pub(super)` opens the door for. The formula matches the doc
comment word for word: multiply first, then divide, so
`1000 * (100 - 25) / 100 = 75000 / 100 = 750`, not a fraction rounded
early. Integer division truncates rather than rounds, which is exactly
why `rental_price_cents_rounds_down` expects `749`, not `750`, for a
999-cent base at a 25% discount: `999 * 75 = 74925`, and
`74925 / 100 = 749` with the remainder simply dropped.

## What this lesson was really about

- **Visibility is a statement about who a function is *for*, not just
  who can technically reach it.** `pricing::discount_percent` could have
  been `pub`; it wasn't, because the discount policy is `catalog`'s
  business alone.
- **`pub(crate)` and `pub(super)` answer different questions.**
  `internal_rating` needed to be readable from anywhere in this crate
  (the tests, sitting outside `catalog`); `discount_percent` only needed
  to be readable from one specific parent.
- **`mod pricing` nested inside `mod catalog` is the same tree the
  lesson built by hand, just doing real work.** Nothing about testing or
  calling these functions required flattening the module structure —
  the tree stayed exactly as deep as it needed to be.
