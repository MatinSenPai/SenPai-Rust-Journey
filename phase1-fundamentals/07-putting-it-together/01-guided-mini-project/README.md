# 1.7.1 — A guided mini-project

## At a glance

After this lesson you can:

- Say why every `Watchlist` method signature — `add`, `find`, `titles` — is an ownership decision, not just a syntax choice, and tie that decision back to the lesson that taught it.
- Add a new variant to an `enum` and let the compiler hand you the complete list of places that now have to change.
- Truncate a Persian title correctly to a fixed number of characters — not bytes — with no risk of a panic.
- Build an error `enum` with a `From` impl and let `?` convert into it, instead of writing a `match` on every fallible call.

**Time:** ~90 minutes · **Prerequisites:**
[1.5.2 — Tuple structs and the newtype pattern](../../05-your-own-types/02-tuple-structs-and-newtype/README.md) ·
[1.5.3 — Enums as data](../../05-your-own-types/03-enums-as-data/README.md) ·
[1.5.4 — `match` in depth](../../05-your-own-types/04-match-in-depth/README.md) ·
[1.4.4 — Slicing text safely](../../04-text-and-strings/04-slicing-text-safely/README.md) ·
[1.6.3 — `Result` and the `?` operator](../../06-absence-and-failure/03-result-and-question-mark/README.md)

And honestly, nearly everything you've read since 1.1.1.

---

## Why this matters

The last thirty lessons were thirty pieces. `enum`, `struct`, `&`, `String` versus `&str`, `Option`, `Result` — each one in its own lesson, with its own examples, apart from the rest.

This lesson teaches no new concept. Its job is to show that those thirty pieces, put together, build a real program — not a toy that exists to illustrate one idea, but something you could actually use.

The program is a **watchlist**: an in-memory library for anime and series, that you can add titles to, search, rate, and summarise. The exact thing you've built a version of in MyAnimeList or Notion a hundred times — this time in Rust, and this time with real titles in Persian, because that is exactly where the text module (Phase 1.4) pays off immediately.

It's built in five stages, each one a subsection of "The concept", and each one code you actually run:

1. **The data** — an `enum` whose variants each carry different data, and a newtype that cannot be built wrong.
2. **The store** — a `struct` wrapping a `Vec`, with three methods that each make a different ownership decision.
3. **Reading it back** — an exhaustive `match` covering all four states, and what happens the moment a fifth is added.
4. **Text done properly** — a summary line built with `format!`, aligned columns, and a title truncated by character count.
5. **Failing well** — a `Result` for the one operation that can genuinely fail, and a `?` that does the error conversion itself.

No new syntax to learn, no new rule to memorise. Just seeing that the last thirty lessons were, for this one program at least, enough.

---

## The concept

### The data: an enum with four shapes

A watchlist needs to hold one thing above all: where each title currently stands. This is the same `Entry` shape you built in [1.5.3](../../05-your-own-types/03-enums-as-data/README.md), with this lesson's names on it:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Watching { episode: u32 },
    Finished { rating: Rating },
    Planned,
    Dropped { at: u32 },
}
```

Four variants, three shapes — one with no data, two with named data. A title that's "being watched" makes no sense without an episode; a title that's "finished" makes no sense without a rating. `Status` guarantees that through its type, not through a check at run time — the same "make the invalid state unwritable" argument from 1.5.3.

### Rating: a newtype that cannot be built wrong

A rating has to be from 0 to 10. You could use a plain `u8` and check the range everywhere it's used — but that means a sleeping bug at every place you forget. This does exactly what [1.5.2](../../05-your-own-types/02-tuple-structs-and-newtype/README.md) did with `Percent`, renamed for this domain:

```rust
struct Rating(u8);

impl Rating {
    fn new(value: u8) -> Rating {
        if value > 10 {
            Rating(10)
        } else {
            Rating(value)
        }
    }

    fn value(self) -> u8 {
        self.0
    }
}
```

```text
Rating::new(9):   9
Rating::new(255): 10
```

The field is private, `new` is the only door in, and `new` clamps the input rather than rejecting it. The result: from this point on, wherever you hold a `Rating`, you never have to re-check that it's at most 10 — the type itself already guaranteed it. `self`, not `&self`, on `value` — because `Rating` is eight bytes and `Copy`, exactly 1.5.2's reasoning.

```text
$ cargo run -p p1-07-01-guided-mini-project --example 01-the-data
Entry { title: "Cowboy Bebop", status: Watching { episode: 9 } }
Entry { title: "Frieren", status: Finished { rating: Rating(9) } }
Entry { title: "حمله به تایتان", status: Dropped { at: 12 } }
Entry { title: "Bocchi the Rock!", status: Planned }

Rating::new(9):   9
Rating::new(255): 10
```

### The store: three methods, three ownership decisions

`Watchlist` is nothing but a `Vec<Entry>`. What's interesting about it isn't the code inside — it's the signatures on its methods:

```rust
struct Watchlist {
    entries: Vec<Entry>,
}

impl Watchlist {
    fn add(&mut self, entry: Entry) { /* ... */ }
    fn find(&self, title: &str) -> Option<&Entry> { /* ... */ }
    fn titles(&self) -> Vec<&str> { /* ... */ }
}
```

Each of the three is an ownership decision that modules 1.2 and 1.3 built the rules for:

```senpai-visual
{"kind":"ownership","labels":["add(entry): takes ownership","list becomes the owner","find(&self): only looks","returns Option<&Entry>","titles(&self): only looks","returns Vec<&str>"]}
```

**`add(&mut self, entry: Entry)`** — the parameter is `Entry`, not `&Entry`. The list is going to hold this entry for as long as the list itself exists, and that's only possible if the list **owns** it. Taking a reference here would tie the list's own lifetime to whoever called `add` — exactly backwards. `&mut self` because pushing onto `self.entries` changes it.

**`find(&self, title: &str) -> Option<&Entry>`** — a search usually means reading a field or printing something. Returning an owned `Entry` would clone a `String` on every single lookup that never needed one — the same "wasteful" loop from 1.2.3, this time hiding inside your own method. `&self` for the same reason: `find` only looks, it changes nothing.

**`titles(&self) -> Vec<&str>`** — every title already belongs to an `Entry` inside `self.entries`. Cloning each one just to list them would be that same pointless clone again. `Vec<&str>` is a vector of borrowed looks, not copies — and because every one of them comes from `&self`, returning it compiles with no lifetime written anywhere; the same elision rule you've already seen on borrows.

```text
$ cargo run -p p1-07-01-guided-mini-project --example 02-the-store
titles: ["Cowboy Bebop", "حمله به تایتان"]
found:     Entry { title: "Cowboy Bebop", status: Watching { episode: 9 } }
not found: Ghost in the Shell
still borrowed from list: true
```

That last line says it plainly: whatever `find` handed back is still borrowed from `list`, and the compiler makes sure `list` isn't moved or dropped while that borrow is alive — without you writing a single word about a lifetime.

And if you're tempted right now to reach for a `HashMap<String, Entry>` so `find` is faster: fair instinct, but that tool arrives in Phase 2. For a list of a few hundred entries, a loop over a `Vec` is exactly the right amount of machinery.

### Reading it back: an exhaustive match

Now that the store exists, it's time to display it — and that's where `match` on `Status` comes in:

```rust
fn describe(entry: &Entry) -> String {
    let detail = match entry.status {
        Status::Watching { episode } => format!("watching, episode {episode}"),
        Status::Finished { rating } => format!("finished, {rating}/10"),
        Status::Planned => "planned".to_string(),
        Status::Dropped { at } => format!("dropped at episode {at}"),
    };
    format!("{} — {detail}", entry.title)
}
```

```text
$ cargo run -p p1-07-01-guided-mini-project --example 03-reading-it-back
Cowboy Bebop — watching, episode 9
Frieren — finished, 9/10
Bocchi the Rock! — planned
حمله به تایتان — dropped at episode 12
```

Four variants, four arms, no `_`. This is exactly 1.5.4's exhaustiveness: the compiler counted how many variants `Status` has and proved all four are covered. Now imagine the product wants a fifth state — "on hold". You add one variant, touch nothing else, and the compiler hands you the complete list of places that need to change. "Errors you will meet" shows exactly that.

### Text done properly: format! and safe truncation

A good summary needs two things: columns that stay aligned, and titles that shorten without breaking anything.

Alignment is free — Rust's `{:<width$}` padding counts **characters**, not bytes:

```text
$ cargo run -p p1-07-01-guided-mini-project --example 04-text-done-properly
Cowboy Beb…  watching
حمله به تا…  dropped
Frieren      finished
Bocchi the…  planned

4 entries — 1 watching, 1 finished, 1 planned, 1 dropped
```

The Persian row lines up exactly as well as the English rows — something that wouldn't be true in a language where padding is counted in bytes.

Truncation is a different story. This is [1.4.4](../../04-text-and-strings/04-slicing-text-safely/README.md)'s function, unchanged:

```rust
fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}
```

The index it returns came from `char_indices` itself, so it's always a char boundary — this slice can never panic, not on "Cowboy Bebop", not on "حمله به تایتان". Swap this for `&title[..10]` (1.4.4's `card_title` again) and on that third row — 25 bytes but only 14 characters — you either panic or cut a letter in half.

The ellipsis (…) is added only when something was genuinely cut, tested with `.chars().count()`, not `.len()`. "Frieren" is seven characters, under the ten-character cap, so it stays plain; the rest get cut and marked.

And the last line — "4 entries, 1 watching, 1 finished..." — is built from four separate counters, not a `HashMap<Status, u32>`. For four known cases, a `Vec` (or here, even simpler: four variables) reads just as clearly; open-ended keys are exactly where Phase 2's `HashMap` earns its place.

### Failing well: the one operation that can fail

Out of every `Watchlist` method, exactly one can genuinely fail: **rating a title that isn't in the list.** `add`, `find`, `titles` — none of them return `Result`, because none of them can fail.

```rust
enum WatchlistError {
    NotFound(String),
    InvalidRating(String),
}

fn rate(&mut self, title: &str, rating: Rating) -> Result<(), WatchlistError> {
    for entry in &mut self.entries {
        if entry.title == title {
            entry.status = Status::Finished { rating };
            return Ok(());
        }
    }
    Err(WatchlistError::NotFound(title.to_string()))
}
```

But in the real world, a rating arrives as text — and that's where a second way to fail shows up: the text might not even be a number. `.parse::<u8>()` returns a `ParseIntError` in that case, not a `WatchlistError`. This is where `From` comes in:

```rust
impl From<std::num::ParseIntError> for WatchlistError {
    fn from(err: std::num::ParseIntError) -> WatchlistError {
        WatchlistError::InvalidRating(err.to_string())
    }
}

fn rate_from_text(&mut self, title: &str, rating_text: &str) -> Result<(), WatchlistError> {
    let raw: u8 = rating_text.trim().parse()?;
    self.rate(title, Rating::new(raw))
}
```

That `?` on the `.parse()` line says exactly this: "if this is `Err`, convert it with `From` and return from this function right now." No `match`, no early-return block written by hand — `?` does both the unwrapping and the error-type conversion, because `WatchlistError` knows how to be built from a `ParseIntError`.

```text
$ cargo run -p p1-07-01-guided-mini-project --example 05-failing-well
Ok(())
Err(NotFound("Cowboy Bebop"))

Ok(())
Err(InvalidRating("invalid digit found in string"))
Err(InvalidRating("invalid digit found in string"))
```

That last line has a subtler point: "Cowboy Bebop" isn't in the list *and* the rating text isn't a number either — and the answer you get is `InvalidRating`, not `NotFound`. `parse` runs before the title is ever looked up; bad input is always reported as bad input, never as a missing title.

---

## Hands on

```sh
cargo run -p p1-07-01-guided-mini-project --example 01-the-data
cargo run -p p1-07-01-guided-mini-project --example 02-the-store
cargo run -p p1-07-01-guided-mini-project --example 03-reading-it-back
cargo run -p p1-07-01-guided-mini-project --example 04-text-done-properly
cargo run -p p1-07-01-guided-mini-project --example 05-failing-well
```

Then the broken one:

```sh
cargo run -p p1-07-01-guided-mini-project --example 06-a-new-status --features broken
```

Then try:

1. In `02-the-store`, actually uncomment the line that prints `bebop` again. Which of 1.2.2's errors do you get?
2. In `04-text-done-properly`, lower `MAX_CHARS` from 10 to 5. Which titles now get an ellipsis that didn't before?
3. In `05-failing-well`, swap the order of the last two lines in `main` — try the invalid rating first, then the missing title. Do the answers change? Should they?

---

## Errors you will meet

### `E0004` — a variant was added, at every site that matches on it

`examples/06-a-new-status.rs` is exactly the `Status` built above, plus one new variant — `OnHold { since: u32 }` — and the same two functions that `match` on it: `describe` and `status_tag`. Nothing else was touched.

```text
error[E0004]: non-exhaustive patterns: `Status::OnHold { .. }` not covered
  --> examples\06-a-new-status.rs:26:24
   |
26 |     let detail = match entry.status {
   |                        ^^^^^^^^^^^^ pattern `Status::OnHold { .. }` not covered
   |
note: `Status` defined here
  --> examples\06-a-new-status.rs:12:6
   |
12 | enum Status {
   |      ^^^^^^
...
17 |     OnHold { since: u32 },
   |     ------ not covered
   = note: the matched value is of type `Status`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
30 ~         Status::Dropped { at } => format!("dropped at episode {at}"),
31 ~         Status::OnHold { .. } => todo!(),
   |

error[E0004]: non-exhaustive patterns: `&Status::OnHold { .. }` not covered
  --> examples\06-a-new-status.rs:36:11
   |
36 |     match status {
   |           ^^^^^^ pattern `&Status::OnHold { .. }` not covered
   |
note: `Status` defined here
  --> examples\06-a-new-status.rs:12:6
   |
12 | enum Status {
   |      ^^^^^^
...
17 |     OnHold { since: u32 },
   |     ------ not covered
   = note: the matched value is of type `&Status`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
40 ~         Status::Dropped { .. } => "dropped",
41 ~         &Status::OnHold { .. } => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p1-07-01-guided-mini-project` (example "06-a-new-status") due to 2 previous errors
```

**What the compiler is objecting to:** two functions, two errors, one reason. `Status` now has five variants and both `match`es only cover four. The compiler didn't run the program to work this out — it just compared the number of variants against the arms and did the proof. Notice the message names exactly which variant is missing — `OnHold` — not just "something's missing somewhere".

**A small difference worth noticing between the two errors:** the first says `Status::OnHold` (no `&`), because `entry.status` is a copied value — `Status` derives `Copy`, so the `match` works on a copy. The second says `&Status::OnHold`, because `status` itself, the parameter, has type `&Status`. The same "the matched value is of type `&Progress`" note you saw in [1.5.4](../../05-your-own-types/04-match-in-depth/README.md), on a type you built yourself this time.

**The fix:** add a `Status::OnHold { since } => ...` arm to both matches (or `Status::OnHold { .. } => ...` in the second, since it never needed `since`).

**Why that's the fix:** this is exactly the bargain [1.5.4](../../05-your-own-types/04-match-in-depth/README.md) promised. If either `match` had ended in `_ => ...` instead, neither error would have appeared at all — the code would have compiled quietly, and "on hold" would have picked up whatever the default arm did without anyone deciding that was right. Naming every variant, instead of `_`, buys exactly this pair of noisy errors, at the price of peace of mind.

---

## Exercises

### Warm up

<details>
<summary>What does <code>Rating::new(15).value()</code> return?</summary>

`10`. `new` clamps the value rather than rejecting it — nothing you pass it can build more than `MAX_RATING`.

</details>

<details>
<summary>After <code>list.add(bebop);</code>, does <code>println!("{bebop:?}")</code> compile?</summary>

No. `add` took ownership of `bebop` — the same `E0382` from 1.2.2, on a type you wrote yourself.

</details>

<details>
<summary>You have a five-variant <code>Status</code> and a <code>match</code> with four named arms and no <code>_</code>. Does it compile?</summary>

No. The compiler counted that one variant isn't covered and gives `E0004` — whether or not you actually use the fifth variant anywhere.

</details>

<details>
<summary>Why can <code>&text[..n]</code> panic on a Persian title, but <code>truncate_to_chars(text, n)</code> never can?</summary>

The first is an arbitrary byte index that might land in the middle of a multi-byte character. The second only ever returns indices `char_indices` itself produced — and those are always char boundaries.

</details>

<details>
<summary>What error does <code>rate_from_text("Cowboy Bebop", "nine")</code> give, when "Cowboy Bebop" isn't in the list at all?</summary>

`InvalidRating`, not `NotFound`. The text is parsed before the title is looked up; a failed parse ends the function early via `?`.

</details>

<details>
<summary>True or false: every <code>Watchlist</code> method returns a <code>Result</code>.</summary>

False. Only `rate` and `rate_from_text` do. `add`, `find` and `titles` can't fail, so none of them has a `Result`.

</details>

### Repair

Run `examples/06-a-new-status.rs` with `--features broken`, read both `E0004` errors, then fix them both:

1. By adding a named `Status::OnHold { .. } => ...` arm to both matches.
2. Then, just to see the difference, replace the last arm of one of them with `_ => ...` and add a sixth variant. Which `match` stays silent this time? The same question 1.5.4 asked.

### Implement

Five methods in `src/lib.rs`:

```sh
cargo test -p p1-07-01-guided-mini-project
```

The types — `Rating`, `Status`, `Entry`, `WatchlistError` — are already fully written; your job is only `Watchlist`'s methods. Every method has a doc comment that states its behaviour exactly — nothing the tests check that the doc comment doesn't say.

Two things the tests are strict about: `titles` must come back in insertion order, not any other order; and `rate_from_text` must parse the text before looking the title up — an invalid rating, even against a title that doesn't exist, must give `InvalidRating`, not `NotFound`.

### Build

Write a new method on `Watchlist`: `fn render_all(&self) -> String`, combining stage 3 (describing each entry with `match`) and stage 4 (aligned, truncated columns) into a single method on the type itself — instead of two free functions each taking a `&Watchlist`.

There's no pre-written test for it; decide the exact shape of the output yourself, write the signature, and try it on a mix of Persian and English titles.

### Challenge (optional)

Write a `fn remove(&mut self, title: &str) -> Option<Entry>` that removes the title from the list and hands **ownership** of it back to the caller — unlike `find`, which only lends it.

Think about it: which `Vec` method fits better here, `remove` or `swap_remove`? Which one preserves the order of the remaining titles, and does that matter to you? And if two entries happen to share exactly the same title — something `find` never had to ask — which one should this method take?

Looking ahead: the `titles` you wrote in this lesson is a hand-written loop. Once you reach 2.2 (iterators), come back and rewrite it as `self.entries.iter().map(|e| e.title.as_str()).collect()`, and see how many lines disappear.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| newtype clamp | a constructor that adjusts bad input instead of rejecting it | `Rating::new` |
| ownership decision in a signature | `T` versus `&T` versus `&mut T`, on a parameter or a return | `add`, `find`, `titles` |
| exhaustive match | the compiler proves every variant is covered | `describe`, `status_tag` |
| `E0004` | a variant is missing from a `match` | anywhere an `enum` grows |
| truncation by character | a cut that always comes from `char_indices`, never panics | Persian and English titles alike |
| error `enum` + `From` | `?` performs the error-type conversion itself | `rate_from_text` |

### What you now know

- An `enum` with data-carrying variants makes an invalid state unwritable; an exhaustive `match` makes forgetting a state a compile error.
- Every method's signature is an ownership decision: an owned parameter means "I'm keeping this"; a borrowed parameter or return means "I'm only looking."
- String padding in Rust is counted in characters, but string slicing is counted in bytes — two entirely separate rules.
- Safe truncation comes from indices `char_indices` itself produced, never from an arbitrary number.
- `Result` is written only for an operation that can genuinely fail; everything else is simpler without it.
- An `impl From<X> for Y` means `?` can convert an `X` into a `Y` on its own, with no `match` written by hand.

### What comes back later

- **`From` and error conversion, in full** — [1.6.5](../../06-absence-and-failure/05-from-and-error-conversion/README.md)
- **The open-ended keys this lesson worked around with a `Vec`** — [Phase 2 — `HashMap`](../../../phase2-intermediate/01-collections/01-vec-and-hashmap/README.md)
- **`titles` as one line instead of a loop** — [Phase 2 — Iterators](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md)
- **When one variant grows far bigger than the rest** — [Phase 2 — `Box`](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md)
- **The full phase review, instead of one program** — [1.7.2 — Phase review](../02-phase-review/README.md)

### Can you explain?

- Why does `add` take ownership of an `Entry`, but `find` only lends one out?
- What happens to a pre-written exhaustive `match` when a new variant is added to its `enum`?
- Why can truncating a Persian title to N bytes panic, but truncating to N characters never can?
- Why do only two of `Watchlist`'s five methods return a `Result`?
- What does `?` do on its own, on a `ParseIntError`, inside a function returning `Result<_, WatchlistError>`?

---

## Going further

- [The Rust Book — an I/O project](https://doc.rust-lang.org/book/ch12-00-an-io-project.html) — the same idea, at the scale of a command-line tool.
- [`std::str::CharIndices`](https://doc.rust-lang.org/std/str/struct.CharIndices.html) — the documentation for the function that made safe truncation possible.
- [`std::convert::From`](https://doc.rust-lang.org/std/convert/trait.From.html) — the documentation for what `?` calls behind the scenes.
