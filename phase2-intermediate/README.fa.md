# فاز ۲ — سطح متوسط و استاندارد (Intermediate & Idiomatic Rust)

تو فاز ۱ یاد گرفتی چطوری کامپایلر رو راضی کنی. فاز ۲ بهت یاد میده چطوری به روش برنامه‌نویس‌های باتجربه‌ی Rust کد بنویسی: استفاده از genericها، بر پایه traitها، قابل‌تست بودن و — در نهایت — همروند (concurrent) و ناهمگام (asynchronous). اینجا همون فازیه که قطعاتی که برای یه بک‌اند واقعی (تو فاز ۳ و ۴) نیاز داری، یکی‌یکی شروع می‌کنن به جا افتادن.

1. **کالکشن‌ها (Collections)**
   - [۰۱ — `Vec` و `HashMap`](01-collections/01-vec-and-hashmap/README.md)
   - [۰۲ — `BTreeMap`، `HashSet` و `VecDeque`](01-collections/02-btreemap-hashset-vecdeque/README.md)
2. **پیمایشگرها و کلوژرها (Iterators & closures)**
   - [۰۱ — کلوژرها و traitهای `Fn`](02-iterators-and-closures/01-closures-and-fn-traits/README.md)
   - [۰۲ — آداپتورهای Iterator](02-iterators-and-closures/02-iterator-adapters/README.md)
3. **نوع‌های عمومی و خصیصه‌ها (Generics & traits)**
   - [۰۱ — توابع و ساختارهای Generic](03-generics-and-traits/01-generic-functions-and-structs/README.md)
   - [۰۲ — تعریف و پیاده‌سازی traitها](03-generics-and-traits/02-defining-and-implementing-traits/README.md)
   - [۰۳ — اشیای Trait در برابر تخصیص استاتیک (static dispatch)](03-generics-and-traits/03-trait-objects-vs-static-dispatch/README.md)
4. **مدیریت خطا و طول‌عمرها (Error handling & lifetimes)**
   - [۰۱ — نوع‌های خطای سفارشی](04-error-handling-and-lifetimes/01-custom-error-types/README.md)
   - [۰۲ — استفاده از `thiserror` و `anyhow`](04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.md)
   - [۰۳ — مبانی طول‌عمر (Lifetime) و elision](04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)
5. **اشاره‌گرهای هوشمند (Smart pointers)**
   - [۰۱ — `Box` و تخصیص روی heap](05-smart-pointers/01-box-and-heap-allocation/README.md)
   - [۰۲ — `Rc` و `Arc`](05-smart-pointers/02-rc-and-arc/README.md)
   - [۰۳ — `RefCell` و تغییرپذیری داخلی (interior mutability)](05-smart-pointers/03-refcell-and-interior-mutability/README.md)
6. **سازمان‌دهی پروژه و تست‌نویسی**
   - [۰۱ — ماژول‌ها، سطح دسترسی (visibility) و workspaceها](06-project-organization-and-testing/01-modules-visibility-workspaces/README.md)
   - [۰۲ — تست‌های واحد، یکپارچگی و doc](06-project-organization-and-testing/02-unit-integration-doc-tests/README.md)
   - [۰۳ — مرور کلی بر Rustِ ناایمن (`unsafe`)](06-project-organization-and-testing/03-unsafe-rust-overview/README.md)
7. **همروندی و Async**
   - [۰۱ — Threadها، `Mutex` و `Arc`](07-concurrency-and-async/01-threads-mutex-arc/README.md)
   - [۰۲ — کانال‌ها و پیام‌رسانی (message passing)](07-concurrency-and-async/02-channels-message-passing/README.md)
   - [۰۳ — مفهوم Future و runtimeها](07-concurrency-and-async/03-futures-and-runtimes/README.md)
   - [۰۴ — مبانی Tokio](07-concurrency-and-async/04-tokio-basics/README.md)
8. **جعبه‌ابزار Rust** — [بررسی کلی ماژول](08-rust-toolbox/README.md)
   - [۰۱ — مبحث Pattern matching به صورت عمیق](08-rust-toolbox/01-pattern-matching-depth/README.md)
   - [۰۲ — مبانی `!macro_rules`](08-rust-toolbox/02-macro-rules-basics/README.md)
   - [۰۳ — Featureهای Cargo](08-rust-toolbox/03-cargo-features/README.md)
   - [۰۴ — مفهوم `TryFrom` و تبدیل‌های خطاپذیر (fallible conversions)](08-rust-toolbox/04-tryfrom-fallible-conversions/README.md)

**ایستگاه انگیزشی:** [ماموریت جانبی ۲ — ربات تلگرامی مسابقه/کوییز](../side-quests/sq-02-telegram-quiz-bot/README.md)
— اولین پروژه‌ی واقعیِ async تو.

وقتی که تک‌تک تیک‌های فاز ۲ رو تو فایل [`PROGRESS.md`](../PROGRESS.md) زدی، برو سراغ [فاز ۳](../phase3-backend-foundations/README.md)، جایی که تمام این مفاهیم شروع می‌کنن به تبدیل شدن به یه بک‌اند واقعی.