# ۰۸.۱ — pattern matching عمیق‌تر

## match guard

پس از pattern می‌توان `if` گذاشت: `Some(n) if n > 0 => ...`. guard فقط پس از matchشدن pattern ارزیابی می‌شود، اما compiler برای exhaustiveness به منطق دلخواه guard اعتماد نمی‌کند؛ پس همچنان شاخه‌ی پوششی لازم است.

## or-pattern و binding با `@`

`A | B` چند pattern را به یک بدنه وصل می‌کند؛ همه‌ی جایگزین‌ها باید bindingهای هم‌نام و هم‌نوع بسازند. `n @ 400..=499` هم مقدار را به `n` می‌بندد و هم range را بررسی می‌کند.

## destructuring تو‌در‌تو و slice pattern

می‌توان enum، struct، tuple و reference را در یک pattern باز کرد. slice patternهایی مانند `[]`، `[only]` و `[first, .., last]` طول‌های مختلف را به‌شکل exhaustively پوشش می‌دهند.

## binding mode

هنگام match روی `&LogEvent`، match ergonomics bindingها را خودکار reference می‌کند تا داده از پشت borrow move نشود؛ به همین دلیل معمولاً `ref` را دستی نمی‌نویسی.

```senpai-visual
{"kind":"concept","labels":["value","pattern","guard","binding @","arm"]}
```

مثل باجه‌ی دسته‌بندی مرسوله است، اما مرز تشبیه این است که pattern فقط شرط نیست؛ می‌تواند هم‌زمان ساختار را باز کند، داده bind کند و exhaustiveness را اثبات کند.

## تمرین تو

تابع‌های `src/lib.rs` را با pattern مناسب و بدون ifهای زائد کامل کن.
