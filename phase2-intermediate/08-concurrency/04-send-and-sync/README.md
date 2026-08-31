# 2.8.4 — `Send` and `Sync`: what they actually are, and why your type isn't `Send`

## At a glance

After this lesson you can:

- For any type — from the standard library or your own — say whether it's `Send` and whether it's `Sync`, just by looking at its fields.
- State the precise relationship between `Send` and `Sync` in one sentence, and use it to explain why `RefCell<T>` can be handed to another thread but can't be shared across several at once.
- Read an `E0277` error ("cannot be sent between threads safely") on your own code, say exactly what caused it, and fix it by choosing the right type — not by guessing.

**Time:** ~45 minutes · **Prerequisites:**
[2.8.3 — Channels and message passing](../03-channels-message-passing/README.md)

---

## Why this matters

From 2.8.1 to here, every time shared state between threads has come up, you've reached for `Arc<Mutex<T>>` — never `Rc<RefCell<T>>`. 2.8.3, right where it compared channels against that same pair, put the same choice in front of you again. By now you've accepted this as a rule: "threads involved means `Arc`/`Mutex`; leave `Rc`/`RefCell` alone." But nothing has actually said why yet — only that you should do it.

2.6.3 went further than that. Right where it introduced `Rc<T>`, it said outright that `Rc<T>` doesn't implement the `Send` marker trait, and that "the exact mechanics of crossing a thread boundary, and `Send` itself, belong to module 8." 2.6.5 took an even more concrete step: its optional challenge had you try putting an `Rc<RefCell<i32>>` inside `thread::spawn` yourself. It didn't compile. The compiler said exactly this: it "cannot be sent between threads safely" — and right there it said that same pattern, across several threads instead of one, is this module's subject.

Today both of those promises get paid off. `Send` and `Sync` are two formal traits Rust has for exactly this question. And they differ from every trait you've written so far in one specific way: you never `impl` them anywhere, and the compiler never waits for a `#[derive]` — it works it out itself, by looking at a type's own fields. After this lesson you no longer need to memorize "`Arc`/`Mutex`, not `Rc`/`RefCell`" — you can prove it yourself, on any new type you write.

---

## The concept

### `Send` — can ownership move to another thread?

Most types you've written so far can, right now, with no extra work at all, have their ownership move completely across a thread boundary:

```rust
use std::thread;

let name = String::from("senpai");
let numbers = vec![1, 2, 3, 4, 5];

let handle = thread::spawn(move || {
    let total: i32 = numbers.iter().sum();
    format!("{name} counted a total of {total}")
});
```

```rust
println!("{}", handle.join().unwrap());
```

```text
senpai counted a total of 15
```

Nothing special was written anywhere in that code — no `impl`, no marker, not even a request. `String` and `Vec<i32>` both simply allow their ownership to move completely. Rust calls this property **`Send`**: a type is `Send` if ownership of a value of that type can be safely handed to another thread. Almost every type you've built or used so far in this course — numeric types, `String`, `Vec<T>`, `HashMap`, your own structs and enums — is `Send`. The exceptions are rare, but today you meet exactly one of them, along with the reason.

### `Sync` — can `&T` be touched from several threads at once?

`Send` is about ownership moving. A completely separate question is: if ownership stays right here, but several threads hold a shared reference (`&T`) to the same value at the same time — is *that* safe? Rust calls this property **`Sync`**: a type is `Sync` if a `&T` of it can be safely shared across several threads.

These two traits are tied together closely enough that their relationship is one precise, quotable rule — the one that's worth keeping in your head from today on:

> `T` is `Sync` exactly when `&T` is itself `Send` — no more, no less.

This isn't just true on paper; the compiler checks it directly, and the next subsection shows you exactly that.

### A genuinely new category of trait: the auto trait

Every trait you've written so far in this course, you either `impl`ed yourself (`impl Display for ...`) or at least asked the compiler to write with `#[derive(...)]`. `Send` and `Sync` are neither — `impl Send for Ticket` isn't something you can write, and `#[derive(Send)]` means nothing; neither is even legal syntax. The compiler works it out entirely on its own, purely by looking at a type's fields: **a struct is `Send` exactly when every one of its fields is; the exact same rule holds for `Sync`.** Rust calls this category of trait — no methods, no manual `impl`, computed from composition — an **auto trait**. (These are also called **marker traits**, the same term 2.6.3 used for `Send` — "marker" because they carry no methods at all, they just flag a fact about the type.)

```rust
fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

// For any `T`: if `T` is `Sync`, `&T` is `Send`.
fn shared_ref_is_send_when_sync<T: Sync>() {
    assert_send::<&T>();
}

struct Ticket {
    id: u32,
    title: String,
}
```

```rust
let ticket = Ticket { id: 7, title: "senpai-api outage".to_string() };
assert_send::<Ticket>();
assert_sync::<Ticket>();
shared_ref_is_send_when_sync::<Ticket>();
println!("Ticket #{} ({}) is Send and Sync; so is &Ticket", ticket.id, ticket.title);
```

```text
Ticket #7 (senpai-api outage) is Send and Sync; so is &Ticket
```

Nothing in this code told the compiler what `Ticket` is with respect to `Send`/`Sync`. `id: u32` and `title: String` are both `Send` and `Sync`, so `Ticket` is too — with zero effort from you; and because `Ticket` is itself `Sync`, `shared_ref_is_send_when_sync` proved exactly the relationship you read above, on this very type. That's what "auto" in the name actually means.

### Why `Rc<T>` is neither `Send` nor `Sync`

2.6.3 showed you this: the count inside an `Rc<T>` is a completely ordinary integer. "Add one" to it is three separate steps — read, add one, write back — and if two threads did exactly those three steps on the same count at the same time, one increment could be lost entirely: a data race, on the count itself.

You can now say this in today's language. If `Rc<T>` were allowed to be `Send`, nothing would stop that exact race — two threads could each hold an `Rc` and clone it at the same time. Rust closes off that possibility at compile time: `Rc<T>` is neither `Send` nor `Sync` — you can't hand its ownership to another thread, and you can't share its `&Rc<T>` either, because both eventually reach that same unsafe count.

`Arc<T>` solves the exact same problem, the same way 2.6.3 showed you: its count moves up and down through **atomic** CPU instructions — a single, indivisible operation, even when several cores touch the same memory at once. No race on the count is possible, so there's no reason left for `Arc<T>` to be denied `Send`/`Sync` — and it isn't: `Arc<T>` is both `Send` and `Sync`, provided whatever it holds is itself both `Send` and `Sync`.

```senpai-visual
{"kind":"ownership","labels":["Rc: plain counter","thread boundary","compile error","Arc: atomic counter","crosses safely"]}
```

### `RefCell<T>`: fine to move, not fine to share

2.6.5 showed you how `RefCell<T>` moves the aliasing rule from compile time to run time: `.borrow()`/`.borrow_mut()` return guards, backed by a counter that tracks how many reads and which write are currently alive. That counter, too — just like `Rc<T>`'s — is a completely ordinary, non-atomic integer, nothing special.

Here's where it diverges from `Rc<T>`, though. If an entire `RefCell<T>` — with whatever it holds — moves to another thread, only one thread ever has access to it at a time; that counter is never touched from two places at once. That's why `RefCell<T>` — as long as `T` itself is `Send` — is `Send`. But if you tried to share a `&RefCell<T>` across several threads, several threads could call `.borrow_mut()` at the same time — each one, with no idea about the other, reading and writing that same non-atomic counter. Exactly the same race you saw on `Rc<T>`'s count, this time on the borrow count. That's why `RefCell<T>` is deliberately not `Sync` — no matter what `T` inside it happens to be.

### Why `Mutex<T>` is different

If you summed up the result above as a blanket rule — "anything written to through a shared reference isn't `Sync`" — you'd get the wrong answer, and `Mutex<T>` is exactly the exception that shows why. `Mutex<T>`, like `RefCell<T>`, also moves the aliasing rule to run time: `.lock()` stands in for `.borrow_mut()`. But instead of a plain counter, it uses a real, thread-safe lock — the same one 2.8.1 already had you using. That lock guarantees exactly one thread has access at any moment; the rest wait, instead of reaching in at the same time.

The result is a genuinely subtle difference: `Mutex<T>` is both `Send` and `Sync`, but only on the condition `T: Send` — not `T: Sync`! Because the lock rules out simultaneous access from the ground up, it no longer matters whether `T` itself could have survived simultaneous access. You don't have to take my word for it: even `Mutex<RefCell<i32>>`, even though `RefCell<i32>` itself is not `Sync`, is still `Sync` — for exactly that reason.

So this is exactly the same `Rc`-versus-`Arc` pattern, this time applied to interior mutability instead of a reference count: `RefCell<T>` moves the rule to run time with a simple, non-atomic tool — fine for one thread, unsafe for several; `Mutex<T>` does the same job with a thread-safe tool. `Arc<Mutex<T>>`, the combination you've been using without a full explanation since 2.8.1, is now clear from end to end: `Arc` makes shared ownership safe (an atomic count), `Mutex` makes shared mutation safe (a thread-safe lock). Swap either one out for `Rc`/`RefCell`, and you bring back exactly the unsafe piece you swapped it in to remove.

```senpai-visual
{"kind":"concurrency","labels":["RefCell: plain borrow flag","shared across threads","compile error","Mutex: real lock","safe to share"]}
```

### Raw pointers, and granting these traits by hand — briefly

Two short notes, just so neither surprises you later. Raw pointers — `*const T` and `*mut T` — are also neither `Send` nor `Sync` by default, for the same general reason: the compiler has no way to know whether whatever they point at stays safe under access from several threads, so it assumes the cautious answer. Real work with raw pointers is 2.10's job.

And: notice we never wrote `impl Send for X` ourselves anywhere — that's not an accident, but it isn't a dead end either. A type's author can hand it either trait by hand, with `unsafe impl Send for X {}` (or `Sync`), when they've personally verified it's genuinely safe. That's a real `unsafe` promise — exactly the kind of commitment 2.10 opens up properly; today you just need to know it exists, not how to write it.

---

## Hands on

```sh
cargo run -p p2-08-04-send-and-sync --example 01-ordinary-types-are-send
cargo run -p p2-08-04-send-and-sync --example 02-auto-trait-from-fields
```

Then the three broken ones:

```sh
cargo run -p p2-08-04-send-and-sync --example 03-struct-with-rc-not-send-broken --features broken
cargo run -p p2-08-04-send-and-sync --example 04-rc-refcell-thread-spawn-broken --features broken
cargo run -p p2-08-04-send-and-sync --example 05-arc-refcell-not-sync-broken --features broken
```

And now the actual fix:

```sh
cargo run -p p2-08-04-send-and-sync --example 06-arc-mutex-fix
```

Then try these:

1. In `02-auto-trait-from-fields.rs`, add a field `count: Rc<i32>` to `Ticket`. Three separate call sites in `main` now fail to compile — which three lines, and why each one?
2. In `06-arc-mutex-fix.rs`, raise the thread count from `4` to `8` and halve each thread's inner loop from `1000` to `500`. Does the final count still come out to exactly `4000`?
3. In `04-rc-refcell-thread-spawn-broken.rs`, change only `Rc::new` to `Arc::new` — leave `RefCell` untouched. Predict whether the error message changes, then run it and check. (Hint: this is exactly example 05.)

---

## Errors you will meet

### `E0277` (1) — one field is enough to disqualify the whole struct

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
  --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\03-struct-with-rc-not-send-broken.rs:19:19
   |
19 |     assert_send::<HoldsRc>();
   |                   ^^^^^^^ `Rc<i32>` cannot be sent between threads safely
   |
   = help: within `HoldsRc`, the trait `Send` is not implemented for `Rc<i32>`
note: required because it appears within the type `HoldsRc`
  --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\03-struct-with-rc-not-send-broken.rs:11:8
   |
11 | struct HoldsRc {
   |        ^^^^^^^
note: required by a bound in `assert_send`
  --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\03-struct-with-rc-not-send-broken.rs:16:19
   |
16 | fn assert_send<T: Send>() {}
   |                   ^^^^ required by this bound in `assert_send`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually objecting to:** `assert_send` requires `HoldsRc` to be `Send`. `HoldsRc` has two fields: `label: String` (`Send`) and `count: Rc<i32>` (not `Send`). A struct is `Send` exactly when **every one** of its fields is — one that isn't is enough to disqualify the whole struct, and that's exactly what the compiler says.

**The fix:** if `HoldsRc` genuinely needs to move between threads, its field needs to be `Arc<i32>`, not `Rc<i32>`:

```rust
struct HoldsRc {
    label: String,
    count: Arc<i32>,
}
```

**Why this is the fix:** `Arc<i32>` — unlike `Rc<i32>` — is itself `Send`, because its count is atomic. Once every field of `HoldsRc` is `Send`, `HoldsRc` itself is `Send` again, with no extra work.

### `E0277` (2) — `Rc<RefCell<i32>>` inside `thread::spawn`

```text
error[E0277]: `Rc<RefCell<i32>>` cannot be sent between threads safely
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\04-rc-refcell-thread-spawn-broken.rs:15:32
    |
 15 |       let handle = thread::spawn(move || {
    |                    ------------- ^------
    |                    |             |
    |  __________________|_____________within this `{closure@04-rc-refcell-thread-spawn-broken.rs:15:32}`
    | |                  |
    | |                  required by a bound introduced by this call
 16 | |         *shared.borrow_mut() += 1;
 17 | |     });
    | |_____^ `Rc<RefCell<i32>>` cannot be sent between threads safely
    |
    = help: within `{closure@04-rc-refcell-thread-spawn-broken.rs:15:32}`, the trait `Send` is not implemented for `Rc<RefCell<i32>>`
note: required because it's used within this closure
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\04-rc-refcell-thread-spawn-broken.rs:15:32
    |
 15 |     let handle = thread::spawn(move || {
    |                                ^^^^^^^
note: required by a bound in `spawn`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\thread\functions.rs:128:8
    |
125 | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    |        ----- required by a bound in this function
...
128 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\04_rc_refcell_thread_spawn_broken.long-type-11410446554853862890.txt'
    = note: consider using `--verbose` to print the full type name to the console

For more information about this error, try `rustc --explain E0277`.
```

Two lines of this output are specific to this one run, on this one machine, not part of the lesson itself: the standard library path (the `required by a bound in spawn` line), because it depends on where Rust is installed on your machine, and the hash inside the `long-type-...txt` filename, because it changes every time you compile this exact example — even on this same machine, without you touching the code. Everything else — the error code, the main message, the line and column numbers in the lesson's own file — stays exactly what you see here.

**What the compiler is actually objecting to:** the closure handed to `thread::spawn` takes full ownership of `shared: Rc<RefCell<i32>>` via `move`, and that closure itself must be `Send` — `spawn`'s own signature requires it: `F: Send + 'static`. Because `Rc<RefCell<i32>>` is neither `Send` nor `Sync` (see "Why `Rc<T>` is neither `Send` nor `Sync`" above), a closure holding it can't be `Send` either.

**The fix:** exactly what you've been using since 2.8.1 — `Arc<Mutex<T>>`:

```rust
let shared = Arc::new(Mutex::new(0));
let handle = thread::spawn(move || {
    *shared.lock().unwrap() += 1;
});
```

**Why this is the fix:** `Arc<Mutex<i32>>` is both `Send` (`Arc`'s count is atomic, and `Mutex<i32>` is itself `Send` because `i32: Send`) and `Sync` (because that same `Mutex<i32>: Send` is enough — see "Why `Mutex<T>` is different" above). A closure holding only this type is `Send` itself, and `thread::spawn` is satisfied.

### `E0277` (3) — swapping `Rc` for `Arc` alone isn't enough

```text
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\05-arc-refcell-not-sync-broken.rs:17:32
    |
 17 |       let handle = thread::spawn(move || {
    |  __________________-------------_^
    | |                  |
    | |                  required by a bound introduced by this call
 18 | |         *clone_for_thread.borrow_mut() += 1;
 19 | |     });
    | |_____^ `RefCell<i32>` cannot be shared between threads safely
    |
    = help: the trait `Sync` is not implemented for `RefCell<i32>`
    = note: if you want to do aliasing and mutation between multiple threads, use `std::sync::RwLock` instead
    = note: required for `Arc<RefCell<i32>>` to implement `Send`
note: required because it's used within this closure
   --> phase2-intermediate\08-concurrency\04-send-and-sync\examples\05-arc-refcell-not-sync-broken.rs:17:32
    |
 17 |     let handle = thread::spawn(move || {
    |                                ^^^^^^^
note: required by a bound in `spawn`
   --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\thread\functions.rs:128:8
    |
125 | pub fn spawn<F, T>(f: F) -> JoinHandle<T>
    |        ----- required by a bound in this function
...
128 |     F: Send + 'static,
    |        ^^^^ required by this bound in `spawn`

For more information about this error, try `rustc --explain E0277`.
```

(The same local-path note from the previous error applies here too, on the `required by a bound in spawn` line.)

**What the compiler is actually objecting to:** the message is different this time — "cannot be **shared** between threads safely," not "sent." Because it's `Arc::clone` going into the closure this time, not a bare `Rc`, the compiler goes straight for `Sync`: `Arc<T>` is only `Send` when `T` is both `Send` and `Sync` — and `RefCell<i32>` is exactly where that breaks: it's `Send`, not `Sync`. The message even suggests an alternative itself — `RwLock` (2.8.2's subject) — but this lesson stays with the `Mutex` you've already been using.

**The fix:** swap `RefCell` for `Mutex` — the same fix as the previous error, this time with an `Arc` that was already correct:

```rust
let shared = Arc::new(Mutex::new(0));
let clone_for_thread = Arc::clone(&shared);
let handle = thread::spawn(move || {
    *clone_for_thread.lock().unwrap() += 1;
});
```

**Why this is the fix:** `Mutex<i32>` — unlike `RefCell<i32>` — is both `Send` and `Sync` (see "Why `Mutex<T>` is different" above), so `Arc<Mutex<i32>>` has both conditions `Arc` requires: `T: Send + Sync`.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
let numbers = vec![1, 2, 3];
let handle = std::thread::spawn(move || numbers.iter().sum::<i32>());
handle.join().unwrap();
```

</details>

<details>
<summary>Answer</summary>

Yes. `Vec<i32>` is `Send`, like almost every other type you've written so far — nothing inside it stands in the way.

</details>

<details>
<summary>Does this compile?</summary>

```rust
use std::rc::Rc;

struct Session {
    user: String,
    hits: Rc<u32>,
}

fn assert_send<T: Send>() {}
assert_send::<Session>();
```

</details>

<details>
<summary>Answer</summary>

No — `E0277`. `Session` has two fields; `hits: Rc<u32>` isn't `Send`, and that alone is enough to disqualify all of `Session` from being `Send`.

</details>

<details>
<summary>True or false: a <code>RefCell&lt;i32&gt;</code> can be moved to another thread with <code>move</code>.</summary>

Write down your answer before checking.

</details>

<details>
<summary>Answer</summary>

True. `RefCell<i32>` is itself `Send` — only one thread will ever have access to it at a time, so its non-atomic borrow counter is never touched from two places at once. What you can't do is share a `&RefCell<i32>` across several threads.

</details>

<details>
<summary>True or false: <code>Mutex&lt;RefCell&lt;i32&gt;&gt;</code> is a combination you genuinely need to write when <code>RefCell</code> alone isn't enough.</summary>

Write down your answer before checking.

</details>

<details>
<summary>Answer</summary>

False, and you never need to. `Mutex<T>` already does exactly the job `RefCell<T>` does — mutation through a shared reference — just with a thread-safe tool. Swap `RefCell` for `Mutex`; don't nest one inside the other.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/03-struct-with-rc-not-send-broken.rs` by changing the `count` field from `Rc<i32>` to `Arc<i32>`.
2. Fix `examples/04-rc-refcell-thread-spawn-broken.rs` by changing `Rc<RefCell<i32>>` to `Arc<Mutex<i32>>` (and `.borrow_mut()` to `.lock().unwrap()`).
3. Fix `examples/05-arc-refcell-not-sync-broken.rs` by changing `RefCell` to `Mutex` — leave `Arc` untouched, since it was already correct.

### Implement

Implement all three in `src/lib.rs`:

- the `label_from_thread` function
- the `SharedCounter` struct and its three methods (`new`, `increment`, `value`)
- the `fan_out_increments` function

The exact specification for each is the doc comment right above it.

### Build

Design a small type, in a domain of your choosing, meant to be shared across several threads — a small cache, an event queue, a stats counter. Build it with `Arc<Mutex<T>>`. Then, in a comment, say which field would have failed first — and why — if you'd chosen `Rc<RefCell<T>>` instead.

### Challenge (optional)

Predict, then check: is `Mutex<Rc<i32>>` (not `Arc<Rc<i32>>` — `Mutex` itself) `Send`? Is it `Sync`? Build the check yourself with `assert_send`/`assert_sync` (like example 02) and see whether your prediction held. Write one sentence on why wrapping a type in `Mutex` alone doesn't guarantee anything about `T` itself being `Send`.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Send` | a type whose ownership can be safely handed to another thread | anything that goes into `thread::spawn` via `move` |
| `Sync` | a type whose `&T` can be safely shared across several threads | anything passed around threads behind an `Arc` |
| Auto trait | a trait with no methods, computed by the compiler straight from a type's fields | `Send`, `Sync` — no `impl`, no `derive` |
| Marker trait | a trait that only flags a fact about a type, carrying no methods at all | the same `Send`/`Sync` |
| `T: Sync` ⇔ `&T: Send` | the precise relationship between the two traits | understanding why `RefCell` can move but can't be shared |

### What you now know

- `Send` means ownership can safely move to another thread; `Sync` means `&T` can safely be shared across several — and the relationship between them is exact: `T` is `Sync` exactly when `&T` is itself `Send`.
- `Send`/`Sync` are auto traits: you never write an `impl`, never ask for a `derive` — the compiler works it out from your type's fields; one field that isn't `Send` is enough to make the whole struct not `Send`.
- `Rc<T>` is neither `Send` nor `Sync`, because its count is a plain, non-atomic integer — two threads operating on it at once would create a data race. `Arc<T>` solves the same problem with an atomic count.
- `RefCell<T>` can be handed to another thread outright (`Send`, if `T` is too) but can't have its `&RefCell<T>` shared across several threads (not `Sync`) — its borrow counter is non-atomic too.
- `Mutex<T>` is both `Send` and `Sync`, on the sole condition `T: Send` — because its lock, not a counter, is what guarantees safety under concurrent access. That's why `Arc<Mutex<T>>` works and `Rc<RefCell<T>>` doesn't.
- Raw pointers are neither `Send` nor `Sync` by default; and granting either trait to a type by hand, with `unsafe impl`, is possible but belongs to a different lesson.

### What comes back later

- **Real work with raw pointers, and the other `unsafe` operations this lesson has stayed clear of** — [2.10.4 — `unsafe`, for real](../../10-rust-toolbox/04-unsafe-for-real/README.md)

### Can you explain?

- Why are `Send` and `Sync` two separate traits, not one? What type do you know that has one but not the other?
- Why doesn't the compiler let you write `impl Send for MyType {}` yourself, the way you would for `Display` or `Clone`?
- Explain the relationship `T: Sync` ⇔ `&T: Send` in your own words, without the symbol.
- Why is `Rc<T>` neither `Send` nor `Sync`, while `RefCell<T>` is only missing one of the two? What exactly is the difference between them?
- How does `Mutex<T>` let a `T` that isn't itself `Sync` become `Sync`? If you had to explain this to a classmate, what would you say?

---

## Going further

- [The Rustonomicon — Send and Sync](https://doc.rust-lang.org/nomicon/send-and-sync.html) — the same ground, in more official detail, including how to write `unsafe impl` by hand.
- [`std::marker::Send` docs](https://doc.rust-lang.org/std/marker/trait.Send.html)
- [`std::marker::Sync` docs](https://doc.rust-lang.org/std/marker/trait.Sync.html)
