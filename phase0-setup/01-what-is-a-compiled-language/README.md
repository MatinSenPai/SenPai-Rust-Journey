# 01 — What is a compiled language?

## At a glance

After this lesson you can:

- Explain what actually happens when you run `python manage.py runserver`, and what happens differently when you run `cargo run`.
- Say what a "binary" is, where it sits on your disk, and why you can copy that one file to an empty server and run it.
- Predict which classes of bug Rust catches before your program runs and Python doesn't — and say what you give up for that.

**Time:** ~25 minutes · **Prerequisites:** none; this is the first lesson of the course. Nothing needs to be installed yet.

---

## Why this matters

You're coming from Python. Python gave you a mental habit you've probably never examined: **write the code, run it, and if it's broken you'll see the error.** Running it *is* how you find out whether it's right.

That habit doesn't work in Rust — and that isn't the bad news, it's the good news. But until you understand *why* it's different, you'll spend your first months feeling like the compiler is your opponent, inventing obstacles.

So this lesson has no code. It builds the mental model everything else sits on: the difference between translating as you go and translating once, up front.

---

## The concept

### Python: a translator who follows you around

When you run `python app.py`, here's what happens:

A program called `python3` — itself a compiled program, though you never think about that — reads your `.py` file **as text**. Then it walks it line by line: translating each line into an action and performing it immediately. Then it moves to the next line.

The key part: **every time you run the program, that translation happens again**, from the same text, from scratch.

Three everyday consequences you've lived with:

```python
def send_invoice(user):
    total = calculate_totl(user)   # typo
    return total
```

That file imports without complaint. The program starts. Your tests may well pass. The typo in `calculate_totl` only shows up when a real user clicks "generate invoice" — and then you get a `NameError`, in production, at 3am.

Python never looked at that line. It couldn't: until it gets there, it's just text.

The other two consequences:

- **You have no build step.** `python foo.py` just works. That is genuinely convenient.
- **Shipping your program to someone** means either handing over the `.py` files and making sure they have a compatible Python, or bundling a whole interpreter alongside it with something like PyInstaller.

### Rust: a pedantic editor, and then a standalone file

Rust is **compiled ahead of time**. When you run `cargo build`, the Rust compiler — `rustc` — runs **once**, reads your **entire** program, checks it exhaustively, and translates it into a **binary**.

After that, running the program involves no translation at all. That work is already done.

The recipe analogy — and I'll tell you where it breaks:

- **Python** is like handing someone a recipe in your own language plus a translator who reads it aloud step by step, every single time they cook. The translator can only catch a nonsensical instruction when they reach it out loud.
- **Rust** is like handing that recipe to an extremely pedantic editor who reads **all** of it first and refuses to hand it back until every instruction is unambiguous and physically possible. Only then do you get a clean, standalone card you can cook from as often as you like, with no editor present.

**Where the analogy breaks:** the editor can't tell you whether the recipe makes food that tastes *good*. Neither can the compiler. It won't tell you your business logic is right; it tells you your code is coherent about types and memory. A logic bug is still entirely possible. Compiling means "this isn't nonsense", not "this is correct".

### What is a binary, exactly?

It isn't an abstraction — it's a real file on your disk. After `cargo build` you'll see:

```text
target/debug/my-program.exe        (on Windows)
target/debug/my-program            (on Linux and macOS)
```

That file is not text. It's a sequence of **machine instructions** — numbers your CPU understands directly. No interpreter in the middle.

Three consequences you'll live with for months:

**1. Errors are caught before it runs, anywhere in the program.** That typo? It doesn't compile in Rust. It doesn't matter that the function is never called:

```text
error[E0425]: cannot find function `calculate_totl` in this scope
 --> src/main.rs:2:13
  |
2 |     let total = calculate_totl(user);
  |                 ^^^^^^^^^^^^^^ help: a function with a similar name exists: `calculate_total`
```

That isn't an error blocking you. That's the 3am `NameError`, moved to 2pm today, on your own laptop, with the correct name suggested next to it.

**2. Running is instant and needs nothing installed.** You can copy that file under `target/debug/` to another Linux machine (same architecture) with no Rust toolchain on it at all, and it runs. One file. If you've ever fought `requirements.txt` against the Python version on a server, you'll feel this difference.

**3. Compiling takes real time.** Seconds to minutes, growing with project size. `cargo run` does both steps for convenience, but it's still "compile, then execute", not "execute directly".

That's the trade you're making: **more friction before running, in exchange for entire categories of bug becoming compile errors instead of production incidents.**

```senpai-visual
{"kind":"concept","labels":["source text","compiler","binary","run"]}
```

### What does the compiler actually catch?

Not everything, but far more than Python leads you to expect:

| What | Python | Rust |
|---|---|---|
| Typo in a function or variable name | At run time, if you reach that line | At compile time, always |
| Passing the wrong type (`"3"` instead of `3`) | At run time (or never, until it breaks) | At compile time |
| Forgetting the "there is no value" case | It's `None` until an `AttributeError` | At compile time — Phase 1 |
| Using memory that was already freed | Impossible in Python (it has a GC) | At compile time — the borrow checker |
| A data race between two threads | At run time, nondeterministically, a nightmare | At compile time — Phase 2 |
| Wrong business logic | Never | Never |

That last row matters. The compiler is not magic. But the five rows above it are exactly the ones that cost time and credibility in a real service.

### The genuinely unique part of Rust

That pedantic editor has one part no other mainstream language has: the **borrow checker**.

Its job is to prove *at compile time* that your program never touches memory that is no longer valid. Other languages went two other ways:

- **Python, Java, Go** ship a **garbage collector**: at run time it continuously works out what's still in use and frees the rest. Safe, but it costs — extra memory, and pauses you don't control.
- **C and C++** do neither. You free memory yourself. Get it wrong and the program either crashes or — worse — quietly does the wrong thing and becomes a security hole.

Rust takes a third route: **rules the compiler verifies before the program runs.** No garbage collector, no manual freeing. That's what the whole of Phase 1 is about, and it's why Rust feels hardest at the start. For now, just know it exists and why.

### What "zero-cost abstraction" means

You'll hear this phrase constantly in Rust circles.

It means: you can write high-level, readable code — and the resulting binary is as fast as if you'd written the same thing by hand at a low level. The abstraction disappears **at compile time** rather than costing you at run time.

That's only possible *because* there is a compile step. The compiler has time to read your code, understand it, and rewrite it. A language translating as it goes has no such opportunity.

---

## Exercises

This lesson has no code. Its exercises are thinking — and each one has an answer so you can check yourself.

### Warm up

<details>
<summary>You have a Python file with a function that is never called and contains a typo. You run the program. What happens?</summary>

Nothing. The program runs normally. That function is never reached, so the typo never becomes an error. The equivalent Rust code would not compile at all.

</details>

<details>
<summary>You copy your compiled binary to an empty Linux server with no Rust installed. Does it work?</summary>

Yes, given the same architecture (say both x86-64) and the system libraries it needs. This is exactly what makes deploying Rust so much simpler than deploying Python: packaging means "move that one file".

</details>

<details>
<summary>Does compiling successfully mean your program is correct?</summary>

No. It means your program is coherent about types and memory. If you calculate tax at the wrong rate, the compiler will cheerfully compile it. The compiler removes a class of bugs, not the need to think.

</details>

### Build

Recall a bug you actually shipped to production — a real one, not a hypothetical. Write down:

1. What it was.
2. When it surfaced (while writing? in tests? at run time? on a real user?).
3. Given the table above, could Rust have caught it before the program ran?

Sometimes the answer to (3) is "no". That's a good answer too — better than "yes", because it stops you expecting the compiler to solve everything.

### Challenge (optional)

Search for *"Rust has no runtime"* and read why that's slightly misleading. Hint: Rust has a very small runtime; what it lacks is a garbage collector and a virtual machine. That distinction becomes useful in Phase 4 when we talk about deployment and container size.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| interpreter | a program that reads your code and runs it as it goes | your mental model of Python |
| compiler | a program that translates all of your code into a binary, once | `rustc`, which `cargo` invokes |
| binary | a file of machine instructions the CPU executes directly | `target/debug/…` |
| compile time / run time | before the program starts / while it's running | any discussion of *when* an error is caught |
| garbage collector | automatic memory reclamation at run time | what Rust doesn't have, and Phase 1 explains why |
| zero-cost abstraction | high-level code that costs nothing extra once compiled | why many Rust APIs are shaped the way they are |

### What you now know

- Python translates from text on every run; Rust translates once into a binary and then simply runs it.
- A binary is a real file you can move and execute without Rust installed.
- Rust catches typos, type errors, memory errors and data races before running — and does not catch logic errors.
- You pay for it in compile time, and that pedantic editor is what makes the first month uncomfortable.

### What comes back later

- **`cargo` and what the compiler really does** — [03 — Cargo basics](../04-cargo-basics/README.md)
- **Reading compiler errors** (we'll take that `error[E0425]` apart properly) — [05 — Reading compiler errors](../05-reading-compiler-errors/README.md)
- **The first program you compile yourself** — [06 — Hello, Rust](../03-hello-rust/README.md)
- **The borrow checker and life without a garbage collector** — [Phase 1 — Language foundations](../../phase1-fundamentals/README.md)
- **Small binaries and real deployment** — [Phase 4 — Deployment and operations](../../phase4-backend-advanced/07-deployment-and-operations/README.md)

### Can you explain?

If a Python colleague were sitting across from you, could you say these out loud?

- When you run `python app.py`, what exactly reads the file and what does it do with it?
- Why does Rust find a typo in a function that is never called, and Python doesn't?
- What is a "binary" and where do you find it in your project?
- What does Rust take from you in exchange for catching those errors?
- Name one thing the compiler **cannot** prevent.

---

## Going further

- [The Rust Programming Language — Foreword & Introduction](https://doc.rust-lang.org/book/foreword.html) — the same idea in the Rust team's own words.
- [`rustc` book — What is rustc?](https://doc.rust-lang.org/rustc/what-is-rustc.html) — one page, specifically about the compiler itself.
- [Rust Error Index](https://doc.rust-lang.org/error_codes/error-index.html) — every error code, `E0425` included. You don't need to read it now; just know it exists.
