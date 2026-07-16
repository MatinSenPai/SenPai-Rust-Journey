# 08.2 — `macro_rules!` basics

You've been *calling* macros since your first `println!` — the `!` is the
tell. This lesson is about writing your own with `macro_rules!`, Rust's
**declarative** macro system (procedural macros like `#[derive(Serialize)]`
are a different, heavier mechanism — Phase 3 material at the earliest).

Python has no real equivalent. Decorators and metaclasses transform
*objects at runtime*; a macro transforms *source code at compile time* —
it receives the tokens you wrote and expands into new code before the
compiler proper ever sees them. The reason Rust needs this and Python
doesn't: Python functions take `*args` and any type, so `max(a, b, c)`
is just a function. Rust functions have fixed arity and fixed types, so
anything variadic — `println!`, `vec!`, `format!` — *must* be a macro.

## Matcher and transcriber

A `macro_rules!` definition is a list of arms, each `(matcher) =>
{ transcriber }`. The matcher is a pattern over *tokens* (like `match`,
but for source code); the transcriber is a template the tokens get pasted
into:

```rust
macro_rules! square {
    ($x:expr) => {
        $x * $x // careful — see "evaluate once" below
    };
}
```

`$x:expr` is a **fragment specifier**: capture one full *expression* and
call it `$x`. The ones you'll actually use: `expr` (expression), `ident`
(a name — for generating variables/functions), `ty` (a type), `pat` (a
pattern), `literal`, and `tt` (a single raw token tree — the escape hatch).
A captured `expr` is atomic in the expansion: `square!(1 + 2)` expands to
`(1 + 2) * (1 + 2)` conceptually, not `1 + 2 * 1 + 2` — the capture keeps
its grouping. But notice it now evaluates `1 + 2` **twice** — for real work,
bind it first: `{ let x = $x; x * x }`. The test suite for `timed!` below
checks exactly this.

## Repetition

`$( ... ),*` matches something comma-separated, zero or more times (`+`
for one-or-more, `?` for zero-or-one). The same syntax replays captures
in the transcriber:

```rust
macro_rules! string_vec {
    ( $( $s:expr ),* $(,)? ) => {{
        let mut v = Vec::new();
        $( v.push($s.to_string()); )*
        v
    }};
}
```

The trailing `$(,)?` is idiomatic politeness — it accepts an optional
trailing comma, like every builtin macro does. Multiple arms are tried
top to bottom (first matcher that fits wins), and an arm may invoke the
macro recursively — that's how `max_of!` below reduces N arguments to a
chain of two-argument `std::cmp::max` calls, with a single-argument arm
as the base case.

## Hygiene, in one paragraph

Names introduced *inside* a macro live in their own scope: if `timed!`'s
transcriber declares `let start = Instant::now();`, and the caller already
has a variable named `start`, they do not collide — the macro's `start`
and the caller's `start` are different names as far as the compiler is
concerned. This is called hygiene, and it's why Rust macros don't suffer
the classic C `#define` disasters. The flip side: a macro *can't* quietly
introduce a variable for the caller to use — which is a feature.

## When NOT to write a macro (honest guidance)

Reach for a macro **last**. A function is checked as written, shows real
types in errors, and gets full rust-analyzer support; a macro is checked
per-expansion, produces errors pointing at generated code, and autocomplete
inside one is rough. Generics already cover "same logic, many types."
The legitimate triggers are narrow: variadic arguments (`max_of!`),
syntax a function can't accept (`"k" => "v"` isn't a valid expression, so
`string_map!` *has* to be a macro), wrapping an expression with
before/after code without forcing a closure (`timed!`), and generating
repetitive items. If a plain `fn` compiles for your problem, the macro
version is worse. Full stop.

## Your task — and how this skeleton works

Macros can't contain `todo!()` the way function bodies can (the macro
body is just tokens; it only becomes code where it's *used* — in the
tests). So each transcriber currently expands to a call to
`unsolved(...)`, a helper that panics: the crate **builds** fine, and
`cargo test` **fails** until you replace each placeholder with a real
transcriber. Delete the `unsolved` calls as you go; when all three macros
are done, nothing calls it anymore.

- `string_map! { "k" => "v", ... }` — builds a `HashMap<String, String>`.
  Repetition + the `=>` token in a matcher.
- `max_of!(a, b, c, ...)` — variadic max via recursion + `std::cmp::max`.
- `timed!(expr)` — evaluates `expr` **once**, returns `(result, elapsed)`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
