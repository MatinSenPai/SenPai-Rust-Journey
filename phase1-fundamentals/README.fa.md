# فاز یک — مبانی اصلی (Core Fundamentals)

این همون فازیه که واقعاً باعث می‌شه حس کنی Rust با پایتون فرق داره: مالکیت (ownership)، قرض‌گیری (borrowing) و یه سیستم نوع (type system) که بهت اجازه نمی‌ده نبودِ یه مقدار رو ماسمالی کنی. اینجا آروم پیش برو — هر چیزی که بعداً میاد (از traitها بگیر تا async و پروژه‌ی نهایی) به این بستگی داره که این مدل‌های ذهنی رو درست درک کرده باشی.

1. **متغیر، نوع و control flow**
   - [۰۱ — متغیر، mutability و shadowing](01-variables-types-control-flow/01-variables-mutability-shadowing/README.md)
   - [۰۲ — نوع‌های scalar و compound](01-variables-types-control-flow/02-scalar-and-compound-types/README.md)
   - [۰۳ — شرط، loop و آشنایی با match](01-variables-types-control-flow/03-conditionals-loops-match-intro/README.md)
2. **مالکیت و حافظه**
   - [۰۱ — معنای انتقال (Move semantics)](02-ownership-and-memory/01-move-semantics/README.md)
   - [۰۲ — `Clone` و `Copy`](02-ownership-and-memory/02-clone-and-copy/README.md)
   - [۰۳ — `Drop` و RAII](02-ownership-and-memory/03-drop-and-raii/README.md)
3. **قرض‌گیری و ارجاع**
   - [۰۱ — ارجاع اشتراکی و تغییرپذیر](03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
   - [۰۲ — قواعد borrow checker](03-borrowing-and-references/02-borrow-checker-rules/README.md)
4. **رشته و slice**
   - [۰۱ — `String` در برابر `&str`](04-strings-and-slices/01-string-vs-str/README.md)
   - [۰۲ — sliceها](04-strings-and-slices/02-slices/README.md)
5. **ساختارها (structs)، enum و pattern matching**
   - [۰۱ — struct و متد](05-structs-enums-pattern-matching/01-structs-and-methods/README.md)
   - [۰۲ — enum و match](05-structs-enums-pattern-matching/02-enums-and-match/README.md)
   - [۰۳ — `if let` و `while let`](05-structs-enums-pattern-matching/03-if-let-while-let/README.md)
6. **مبانی `Option`، `Result` و خطا**
   - [۰۱ — `Option` و ایمنی null](06-option-result-error-basics/01-option-and-null-safety/README.md)
   - [۰۲ — `Result` و operator `?`](06-option-result-error-basics/02-result-and-question-mark/README.md)
   - [۰۳ — `!panic` در برابر `Result`](06-option-result-error-basics/03-panic-vs-result/README.md)

**ایستگاه انگیزشی:** [ماموریت جانبی ۱ — CLI نقل‌قول انیمه](../side-quests/sq-01-anime-quote-cli/README.md)
— یه برنامه‌ی واقعی و کوچیک که از همه‌ی چیزای بالا استفاده می‌کنه.

وقتی تمام فاز یک تو [`PROGRESS.md`](../PROGRESS.md) تیک خورد، برو سراغ [فاز دو](../phase2-intermediate/README.md).