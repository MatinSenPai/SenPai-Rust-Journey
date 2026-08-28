# Solution — 1.5.1 Structs and methods

```rust
impl Series {
    pub fn new(title: String, episodes: u32) -> Self {
        Self {
            title,
            episodes,
            watched: 0,
            favourite: false,
        }
    }

    pub fn remaining(&self) -> u32 {
        self.episodes.saturating_sub(self.watched)
    }

    pub fn watch_one(&mut self) -> bool {
        if self.watched < self.episodes {
            self.watched += 1;
            true
        } else {
            false
        }
    }

    pub fn mark_favourite(&mut self) {
        self.favourite = true;
    }

    pub fn summary(&self) -> String {
        let mut line = format!("{} {}/{}", self.title, self.watched, self.episodes);
        if self.favourite {
            line.push_str(" (favourite)");
        }
        line
    }

    pub fn into_title(self) -> String {
        self.title
    }
}
```

Six signatures, four different kinds of `self`. None of them is arbitrary.

## `new` — the associated function, and two uses of `Self`

```rust
pub fn new(title: String, episodes: u32) -> Self {
    Self {
        title,
        episodes,
        watched: 0,
        favourite: false,
    }
}
```

It takes no `self`, because there is no `Series` yet for it to take. That's the definition of an associated function.

`Self` appears twice here and means `Series` both times: once as the return type, once building the value. Writing `Series` in both places would be perfectly correct. `Self` is preferred because if the type is ever renamed, nothing inside the `impl` block has to change.

`title,` and `episodes,` are field init shorthand, which is the reason the parameters were deliberately named after the fields.

The other two fields are filled in by hand. That says "this is what starting a `Series` means", and it says it in one place rather than at every point in the program where a `Series` gets built. It's the sign the `E0063` section pointed at: when you keep retyping the same starting values, the type wants a `new`.

## `remaining` — the subtraction that could have panicked

```rust
pub fn remaining(&self) -> u32 {
    self.episodes.saturating_sub(self.watched)
}
```

The obvious version is `self.episodes - self.watched`, and it works for every sensible input. The last test doesn't give it a sensible input:

```rust
show.watched = 30;
assert_eq!(show.remaining(), 0, "never wraps around the bottom of a u32");
```

`28 - 30` on a `u32` underflows. In debug it panics; in release it lands on `4294967294` — which is what 1.1.2 showed you and why the `checked_` / `saturating_` / `wrapping_` family exists at all.

`saturating_sub` means "the floor is zero". For a progress counter that's exactly what you want.

`if self.watched >= self.episodes { 0 } else { self.episodes - self.watched }` is also correct and a little longer. Choosing between those two is taste; choosing between either of them and the raw subtraction is not.

And notice the `&self`: this method only reads, so the cheapest signature that does the job is the right one. The caller keeps their value and can call it again immediately.

## `watch_one` — changes something and reports back

```rust
pub fn watch_one(&mut self) -> bool {
    if self.watched < self.episodes {
        self.watched += 1;
        true
    } else {
        false
    }
}
```

`&mut self` because a field changes, and a `bool` back because the caller needs to know whether anything happened.

The `if` here is an **expression**, not a statement: both branches end in a `bool` with no semicolon, and that value leaves the function. That's 1.1.4's point. Writing `return true;` also works, but it isn't the Rust idiom.

Don't underrate the guard. Without the `if`, this method would push `watched` past `episodes`, and then `remaining` would have to cope with an impossible state. This is where the private fields earn their keep: the only way to raise `watched` from outside is this method, so "`watched` never exceeds `episodes`" is an **invariant** the type maintains itself.

Make `watched` `pub` and that guarantee evaporates on the spot.

## `mark_favourite` — one line, and why it deserves to exist

```rust
pub fn mark_favourite(&mut self) {
    self.favourite = true;
}
```

Fair question: why a whole method for what is `show.favourite = true;`?

Because outside this file that line doesn't exist at all — the field is private. The method is the only way in, and being the only way in means you can make it do more tomorrow (a timestamp, a log line, a count of favourites) without breaking a single caller.

It returns nothing because it has nothing to say. Its documentation promises that calling it twice is not an error, and the implementation satisfies that without trying.

## `summary` — build it, then add conditionally

```rust
pub fn summary(&self) -> String {
    let mut line = format!("{} {}/{}", self.title, self.watched, self.episodes);
    if self.favourite {
        line.push_str(" (favourite)");
    }
    line
}
```

`format!` builds the part that's always there and `push_str` adds the optional tail. One allocation for the `format!`, and the `push_str` usually fits inside the capacity already there — the same accounting as 1.4.3.

The tempting alternative writes two complete `format!` calls in two `if` branches. It works, and it repeats the main format string twice; the day you change the separator from `/` to something else, you have to remember there are two of them.

It's `&self` because it builds a fresh string and never touches the `Series` itself. A method returning a new `String` almost always takes `&self`.

## `into_title` — one line, and the whole ownership story

```rust
pub fn into_title(self) -> String {
    self.title
}
```

`self` means this method takes the whole `Series`. `self.title` moves that `String` straight out and returns it — no clone, because nobody else is going to use this struct. The other three fields are dropped right there.

With `&self`, that line would not compile: you can't move a value out from behind a shared reference. That's 1.2.2's `E0507`. The answer would then be `self.title.clone()` — an allocation, for a value that was about to be thrown away.

So taking `self` here is a deliberate decision: "I am sure nobody needs this `Series` afterwards, and in exchange I make no allocation." The signature declares that trade to the caller, and the `into_` prefix announces it before they even read the signature.

## What this lesson was really about

- **A struct is a type, not a bag.** The field names live in the type, and the compiler catches two `u32`s written the wrong way round.
- **The signature chooses `self`, and that's a promise.** `&self` is a look, `&mut self` an exclusive look, `self` a swallow. Decide what the caller should be left holding, then write it.
- **Privacy is what makes an invariant possible.** "`watched` never exceeds `episodes`" is only true while a method is the only way to raise it.
- **`derive` does the obvious thing.** `Debug` to see it, `Clone` to duplicate it, `PartialEq` for `==` — all three field by field.
- **`new` is a name, not a feature.** Rust has no constructors; it has associated functions and one very firmly held convention about what to call them.
