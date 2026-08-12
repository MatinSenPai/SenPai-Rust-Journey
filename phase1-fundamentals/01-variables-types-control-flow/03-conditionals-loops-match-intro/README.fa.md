# ۰۱.۳ — شرط، loop و آشنایی با `match`

## `if` یک expression است

```rust
let status = if score >= 90 { "A" } else if score >= 80 { "B" } else { "F" };
```

هر شاخه باید نوع سازگار بدهد. Rust ternary جدا ندارد؛ خود `if` مقدار تولید می‌کند.

- `loop` تا `break` ادامه دارد و `break value` می‌تواند خروجی loop باشد.
- `while condition` تا وقتی شرط برقرار است اجرا می‌شود.
- `for item in iterable` روی `IntoIterator` کار می‌کند؛ loop سبک C وجود ندارد و `for i in 0..n` جایگزین آن است.

```rust
let description = match n {
    0 => "zero",
    1 | 2 => "one or two",
    3..=9 => "single digit, at least three",
    _ => "big",
};
```

`match` exhaustive است؛ کامپایلر می‌خواهد تمام حالت‌ها پوشش داده شوند. در بک‌اند، match روی status یا enum وضعیت سفارش باعث می‌شود افزودن variant جدید محل‌های ناقص را آشکار کند.

تشبیه `match` به `switch` محدود است: patternها می‌توانند داده را destructure و شرط guard داشته باشند، نه فقط equality ساده.

```senpai-visual
{"kind":"concept","labels":["if value","loop value","match all"]}
```
