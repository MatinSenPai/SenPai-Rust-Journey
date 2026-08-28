# Solution — 1.2.5 `Drop` and RAII

```rust
pub fn scope_order() -> Vec<String> {
    {
        let _a = Tracker::new("a");
        let _b = Tracker::new("b");
        let _c = Tracker::new("c");
    }
    take_log()
}

pub fn release_early() -> Vec<String> {
    {
        let early = Tracker::new("early");
        let _late = Tracker::new("late");
        drop(early);
    }
    take_log()
}

pub fn handed_over() -> Vec<String> {
    {
        let given = Tracker::new("given");
        consume(given);
        let _kept = Tracker::new("kept");
    }
    take_log()
}

pub fn from_a_vec(names: Vec<String>) -> Vec<String> {
    {
        let mut group = Vec::with_capacity(names.len());
        for name in names {
            group.push(Tracker::new(&name));
        }
    }
    take_log()
}

pub fn custom_order() -> Vec<String> {
    {
        let a = Tracker::new("a");
        let b = Tracker::new("b");
        let _c = Tracker::new("c");
        drop(b);
        drop(a);
    }
    take_log()
}
```

In all five, the only thing you wrote is **where** each value lives. Not one line says "clean up".

## `scope_order` — the base rule

```rust
let _a = Tracker::new("a");
let _b = Tracker::new("b");
let _c = Tracker::new("c");
```

The log comes back `["c", "b", "a"]`. Three bindings in one block are cleaned up in reverse declaration order.

Two things that are easy to miss:

**The names start with an underscore, but they are not a bare `_`.** Written `let _ = Tracker::new("a");`, that value would be bound to nothing and cleaned up right there on the same line — and the log would be `["a", "b", "c"]`, exactly the opposite of what the test wants. A lone underscore means "I don't want this"; `_a` means "I want it, I just don't use it".

**`take_log()` is outside the block.** Called inside, none of the three would have been cleaned up yet and you'd get an empty list back. Where that call goes is part of the answer.

## `release_early` — taking the decision away from the scope

```rust
let early = Tracker::new("early");
let _late = Tracker::new("late");
drop(early);
```

Without that last line the answer is `["late", "early"]`. With it, `["early", "late"]`.

`drop(early)` takes ownership away from this block and gives it to the `drop` function; that function has no body, so its scope ends immediately and the value dies there. The compiler knows this too, and does not place a second call at the closing brace.

Note that `early` is no longer `_early`: it is genuinely used now, so its name shouldn't apologise for itself.

## `handed_over` — a move carries the cleanup with it

```rust
let given = Tracker::new("given");
consume(given);
let _kept = Tracker::new("kept");
```

The log is `["given", "kept"]`. `"given"` was cleaned up before `"kept"` existed at all, because `consume` became its owner and `consume`'s scope ended immediately.

This function is `release_early` told a different way, and comparing the two makes the point: `drop(x)` has no power that `consume(x)` lacks. **Any** function that takes the value by value and doesn't hand it back pulls the cleanup into itself. `std::mem::drop` is just a clear name for doing that.

## `from_a_vec` — the one that isn't reversed

```rust
let mut group = Vec::with_capacity(names.len());
for name in names {
    group.push(Tracker::new(&name));
}
```

This gives `["a", "b", "c"]`, not `["c", "b", "a"]`.

There aren't three bindings here; there is **one**, called `group`. That single value is cleaned up at the closing brace, and the `Vec`'s destructor walks its elements from index zero upwards. Reverse order is the rule for bindings, not the rule for data.

That difference matters in real code. If the order several connections close in matters to you, holding them in several `let`s and holding them in one `Vec` are two different behaviours.

And `for name in names` consumes `names` itself — exactly what you want, since you don't need it afterwards. `Vec::with_capacity` is free to write here because the count is known up front ([1.2.3](../../03-clone-and-copy/README.md)).

## `custom_order` — both rules at once

```rust
let a = Tracker::new("a");
let b = Tracker::new("b");
let _c = Tracker::new("c");
drop(b);
drop(a);
```

`b`, then `a`, then `c`.

The two explicit drops build the order you were asked for, and `c` is left to the default. Adding `drop(c)` as well would give the same answer but read worse: an explicit `drop` says "I had a particular reason here", and writing one for a value that is already released correctly is noise.

That `c` is the only variable still wearing an underscore isn't a coincidence — it's the only one never used in the code.

## What this lesson was really about

- **Cleanup is a call the compiler writes**, at the owner's closing brace, on every path out.
- **Bindings go in reverse; a collection's elements go first to last.** That's two rules, not one.
- **When cleanup happens *is* ownership.** Every move — into a function, into a `Vec`, into `drop` — moves the timing too.
- **`drop(value)` has no special privilege.** It's a function that takes ownership; `value.drop()` is refused because it would break "exactly once".
- **If your type has a destructor it isn't `Copy`** — the [1.2.3](../../03-clone-and-copy/README.md) table row you now have the reason for.

Next is [module 1.3](../../../03-borrowing-and-references/README.md): looking at a value instead of handing ownership over — which means the cleanup stays exactly where it was.
