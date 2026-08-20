# Solution — Hello, Rust

```rust
pub fn describe_journey(language: &str, day: u32) -> String {
    format!("Day {day} of learning {language}: still compiling.")
}
```

`format!` works exactly like `println!` except it returns a `String`
instead of printing it — same macro-powered compile-time-checked
placeholders (`{day}`, `{language}` — this "capture the variable by name
directly in the braces" syntax is a modern convenience; you'll also see the
older `{}`-with-positional-args style, e.g. `format!("Day {} of learning
{}", day, language)`, which does the same thing and is still completely
idiomatic, especially when the value isn't a simple variable name).
