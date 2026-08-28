# Solution — 2.3.6 Supertraits, blanket impls, the orphan rule and the newtype escape hatch

```rust
pub trait Priced {
    fn price_cents(&self) -> u32;
}

pub trait Discounted: Priced {
    fn discounted_cents(&self) -> u32 {
        self.price_cents() * 90 / 100
    }
}

impl Discounted for Book {}
impl Discounted for Pen {}

pub fn amount_saved<T: Discounted>(item: &T) -> u32 {
    item.price_cents() - item.discounted_cents()
}

pub struct Cart(pub Vec<Pen>);

impl Cart {
    pub fn total_cents(&self) -> u32 {
        self.0.iter().map(Pen::price_cents).sum()
    }
}

impl std::fmt::Display for Cart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} pen(s), {} cents total",
            self.0.len(),
            self.total_cents()
        )
    }
}
```

All four pieces share one idea: traits can depend on each other (`Discounted` leans on `Priced`), and the newtype (`Cart`) is nothing but a wrapped `Vec<Pen>`, written specifically so `Display` — a foreign trait — can be implemented for it.

## `Discounted::discounted_cents` — the supertrait's default body

```rust
fn discounted_cents(&self) -> u32 {
    self.price_cents() * 90 / 100
}
```

This body lives inside `Discounted`'s own definition, not inside `impl Discounted for Book`. Because `Discounted: Priced`, the compiler guarantees every `Self` that reaches this body also has a `price_cents()` — exactly what "The concept" showed: calling a supertrait's method from inside the subtrait. The multiplication happens before the division (not the other way around), with integer math, so `101 * 90 / 100 = 9090 / 100 = 90`, not 90.9 — exactly what the doc comment promised.

`impl Discounted for Book {}` and `impl Discounted for Pen {}` both have empty bodies — they just pick up this same default, unchanged.

## `amount_saved` — one bound, two methods

```rust
pub fn amount_saved<T: Discounted>(item: &T) -> u32 {
    item.price_cents() - item.discounted_cents()
}
```

The signature only asks for `T: Discounted`, but the body calls both `.price_cents()` (from `Priced`) and `.discounted_cents()` (from `Discounted`). The compiler accepts this because `Discounted: Priced` means any `T` that satisfies `Discounted` has already satisfied `Priced` too — writing `T: Discounted + Priced` would have been redundant.

## `Cart::total_cents` — an ordinary forwarding method

```rust
pub fn total_cents(&self) -> u32 {
    self.0.iter().map(Pen::price_cents).sum()
}
```

`Cart` doesn't inherit a single method from `Vec` — this is exactly the price "The concept" said a newtype pays. `self.0.iter()` reaches into the wrapper explicitly, `Pen::price_cents` (the method's name, not a call) is handed to `.map()` as a function, and `.sum()` adds the results.

## `Display for Cart` — where the lesson lands

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
        f,
        "{} pen(s), {} cents total",
        self.0.len(),
        self.total_cents()
    )
}
```

This is where the whole lesson meets: `Display` is a foreign trait, `Vec` is a foreign type, and `Cart` — the local wrapper — is the only reason this `impl` compiles at all. The function calls `self.total_cents()` again rather than re-computing the sum; the `cart_display_matches_total_cents` test checks exactly that — if the formula were rewritten here, the two could drift apart one day.

## What this lesson was really about

- **A supertrait is a dependency, not inheritance.** Writing `impl Discounted for Book {}` doesn't hand you anything from `Priced` for free; you still write `impl Priced for Book` separately. The supertrait only guarantees you already did.
- **Supertrait methods are reachable from a subtrait bound.** `amount_saved<T: Discounted>` never needed `+ Priced`.
- **A blanket impl means writing one impl for a thousand types, once.** The same trick the standard library used for `impl<T, U> Into<U> for T where U: From<T>`.
- **The orphan rule means: trait or type, at least one has to be local.** `Display` is foreign, `Vec` is foreign — `impl Display for Vec<Pen>` directly never compiles.
- **A newtype is an escape hatch.** `Cart(Vec<Pen>)` creates a fresh local type; now `impl Display for Cart` is legal, because one side (`Cart`) is local.
