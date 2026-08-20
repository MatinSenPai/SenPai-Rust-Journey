# 03.2 — Defining and implementing traits

Lesson 01 used a *bound* (`T: PartialOrd`) — a trait someone else (the
standard library) already defined, that you leaned on. This lesson is about
writing your **own** trait: a contract you design, that your own structs (or
anyone else's) can promise to fulfill.

## Declaring a trait

```rust
pub trait Summarize {
    fn title(&self) -> String;

    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}
```

A trait is a set of method *signatures*. `title` has no body — every type
that implements `Summarize` **must** provide its own `title`. `summary`,
though, has a body: a **default implementation**. Any implementor gets
`summary` for free, exactly as written, unless it chooses to override it
with its own version.

Notice `summary`'s default body calls `self.title()` — a method that
doesn't exist yet at the point the trait is *defined*. This compiles because
the trait itself guarantees every implementor supplies a `title`; the
compiler doesn't need to know the concrete type to know that call is safe,
only that whatever type ends up here satisfies the trait.

## Implementing it

```rust
pub struct AnimeSeries {
    pub title: String,
    pub episodes: u32,
}

impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title, self.episodes)
    }
}
```

`AnimeSeries` supplies both `title` (required) and its own `summary`
(overriding the default). A different struct could implement only `title`
and inherit `summary` completely unchanged — you'll do exactly that with
`MangaVolume` in this lesson's exercise.

## The Python comparison

The closest thing you've likely used is an **ABC** (`abc.ABC` +
`@abstractmethod`) or a `Protocol`, optionally combined with a **mixin**:

```python
class Summarize(ABC):
    @abstractmethod
    def title(self) -> str: ...

    def summary(self) -> str:
        return f"{self.title()} (no summary available)"
```

Same idea — `title` is abstract (must be overridden), `summary` is a mixin
method with a default body that calls the abstract one. The difference is
*when* this gets checked. Python only discovers a missing `title` override
the first time you try to instantiate the class (or, with a plain
`Protocol`, potentially never — Python won't stop you from passing something
that's missing a method until the exact line that calls it blows up).
Rust's compiler checks every `impl Summarize for X` block at compile time:
if `X` is missing `title`, `cargo build` fails immediately, before any code
runs, naming the exact missing method.

## Your task

In `src/lib.rs`: finish the `Summarize` trait, implement it for
`AnimeSeries` (custom `summary`) and `MangaVolume` (default `summary`
only), and implement `print_all_summaries`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
