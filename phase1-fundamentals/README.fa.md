# فاز ۱ — مبانیِ اصلی

این همان فازی است که Rust را از پایتون جدا می‌کند: مالکیت، قرض گرفتن، و یک سیستمِ نوع که نمی‌گذارد نبودنِ یک مقدار را زیرِ فرش پنهان کنی. اینجا آهسته برو. هرچه بعداً می‌آید — صفت‌ها، async، پروژه‌ی نهایی — روی درست جا افتادنِ همین مدل‌های ذهنی سوار است.

سی‌ویک درس در هفت ماژول، به همان ترتیبی که قرار است خوانده شوند. هر درس پیش‌نیازش را می‌گوید، و هر درس می‌گوید کدام درسِ بعدی چیزهایی را که فقط شروع کرده کامل می‌کند.

## ۱. [پایه‌ها](01-foundations/README.fa.md)

لایه‌ی نحو، و تصمیم‌های کوچکی که Rust مجبورت می‌کند خودت بگیری.

۱. [متغیرها، تغییرپذیری و سایه‌اندازی](01-foundations/01-variables-mutability-shadowing/README.fa.md)
۲. [انواعِ اسکالر و سرریز](01-foundations/02-scalar-types-and-overflow/README.fa.md)
۳. [انواعِ مرکب و واکافت](01-foundations/03-compound-types-and-destructuring/README.fa.md)
۴. [توابع و عبارت‌ها](01-foundations/04-functions-and-expressions/README.fa.md)
۵. [جریانِ کنترل](01-foundations/05-control-flow/README.fa.md)
۶. [مقدماتِ `Vec` و `String`](01-foundations/06-vec-and-string-basics/README.fa.md)

## ۲. [مالکیت و حافظه](02-ownership-and-memory/README.fa.md)

قلبِ زبان، و دلیلِ وجودِ هرچه بالای آن است.

۱. [پشته و هیپ](02-ownership-and-memory/01-stack-and-heap/README.fa.md)
۲. [معناشناسیِ حرکت](02-ownership-and-memory/02-move-semantics/README.fa.md)
۳. [`Clone` و `Copy`](02-ownership-and-memory/03-clone-and-copy/README.fa.md)
۴. [مالکیت در عبور از توابع](02-ownership-and-memory/04-ownership-across-functions/README.fa.md)
۵. [`Drop` و RAII](02-ownership-and-memory/05-drop-and-raii/README.fa.md)

## ۳. [قرض گرفتن و ارجاع‌ها](03-borrowing-and-references/README.fa.md)

چطور از یک مقدار استفاده کنی بدونِ اینکه بگیری‌اش.

۱. [ارجاع‌های اشتراکی و تغییرپذیر](03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md)
۲. [قواعدِ بررسی‌کننده‌ی قرض](03-borrowing-and-references/02-borrow-checker-rules/README.fa.md)
۳. [دامنه‌ی قرض و NLL](03-borrowing-and-references/03-borrow-scopes-and-nll/README.fa.md)
۴. [برش‌ها](03-borrowing-and-references/04-slices/README.fa.md)

## ۴. [متن و رشته‌ها](04-text-and-strings/README.fa.md)

جایی که Rust سخت‌گیرتر از انتظارت است، و جایی که این سخت‌گیری برای کسی که با فارسی کار می‌کند ارزشش را ثابت می‌کند.

۱. [`String` در برابرِ `&str`](04-text-and-strings/01-string-vs-str/README.fa.md)
۲. [یوتی‌اف-۸: بایت، کاراکتر، گرافیم](04-text-and-strings/02-utf8-bytes-chars-graphemes/README.fa.md)
۳. [ساختن و دگرگون کردنِ رشته‌ها](04-text-and-strings/03-building-and-transforming-strings/README.fa.md)
۴. [برش زدنِ امنِ متن](04-text-and-strings/04-slicing-text-safely/README.fa.md)

## ۵. [نوع‌های خودت](05-your-own-types/README.fa.md)

کاری کن کامپایلر مسئله‌ات را بفهمد، نه فقط داده‌ات را.

۱. [ساختارها و متدها](05-your-own-types/01-structs-and-methods/README.fa.md)
۲. [ساختارهای تاپلی و الگوی newtype](05-your-own-types/02-tuple-structs-and-newtype/README.fa.md)
۳. [enum‌ها به‌عنوانِ داده](05-your-own-types/03-enums-as-data/README.fa.md)
۴. [`match` از نزدیک](05-your-own-types/04-match-in-depth/README.fa.md)
۵. [`if let`، `while let`، `let else`](05-your-own-types/05-if-let-while-let-let-else/README.fa.md)

## ۶. [نبودن و شکست](06-absence-and-failure/README.fa.md)

نه null، نه استثنا. به‌جایش دو enum و یک علامتِ سؤال.

۱. [`Option` و ایمنی در برابرِ نبودن](06-absence-and-failure/01-option-and-null-safety/README.fa.md)
۲. [ترکیب‌گرهای `Option`](06-absence-and-failure/02-option-combinators/README.fa.md)
۳. [`Result` و علامتِ سؤال](06-absence-and-failure/03-result-and-question-mark/README.fa.md)
۴. [پنیک در برابرِ `Result`](06-absence-and-failure/04-panic-vs-result/README.fa.md)
۵. [`From` و تبدیلِ خطا](06-absence-and-failure/05-from-and-error-conversion/README.fa.md)

## ۷. [کنارِ هم گذاشتنش](07-putting-it-together/README.fa.md)

۱. [پروژه‌ی کوچکِ راهنمایی‌شده](07-putting-it-together/01-guided-mini-project/README.fa.md)
۲. [مرورِ فاز](07-putting-it-together/02-phase-review/README.fa.md)

---

**یک برنامه‌ی واقعی که همه‌اش را به کار می‌گیرد:** [ساید-کوئست ۱ — CLI جملات انیمه](../side-quests/sq-01-anime-quote-cli/README.fa.md)

وقتی فاز ۱ در [`PROGRESS.fa.md`](../PROGRESS.fa.md) تیک خورد، برو سراغِ [فاز ۲](../phase2-intermediate/README.fa.md).
