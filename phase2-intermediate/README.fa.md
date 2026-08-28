# فاز ۲ — سطح متوسط و اصطلاحی Rust

فاز ۱ به تو یاد داد چطور کامپایلر را راضی کنی. فاز ۲ به تو یاد می‌دهد Rust را
همان‌طور بنویسی که مهندس‌های باتجربه‌ی Rust می‌نویسند: ایتریتور به‌جای حلقه‌ی
اندیسی، صفت به‌جای توابعِ تکراری، خطاهایی که اطلاعاتِ واقعی حمل می‌کنند، و در
پایان همان مدلِ هم‌روندی که اصلاً یادگیریِ این زبان را ارزشمند کرد.

چهل‌وهشت درس در ده ماژول، به همان ترتیبی که قرار است خوانده شوند.

## ۱. [مجموعه‌ها](01-collections/README.fa.md)

واقعاً داخلِ `Vec` و `HashMap` چه خبر است، و چه چیزِ دیگری روی قفسه هست.

۱. [`Vec` از نزدیک: ظرفیت، `retain`، `drain`، `dedup`، `binary_search`](01-collections/01-vec-depth/README.fa.md)
۲. [`HashMap` از نزدیک: `entry`، هشرها، جست‌وجو با `&str`](01-collections/02-hashmap-in-depth/README.fa.md)
۳. [`BTreeMap`، `HashSet`، `VecDeque`، `BinaryHeap`](01-collections/03-btreemap-hashset-vecdeque/README.fa.md)
۴. [انتخابِ مجموعه: جدولِ پیچیدگی و اینکه هرکدام کِی برنده است](01-collections/04-choosing-a-collection/README.fa.md)

## ۲. [ایتریتورها و کلوژرها](02-iterators-and-closures/README.fa.md)

بزرگ‌ترین تغییر در شکلِ نوشتنِ Rustِ اصطلاحی.

۱. [کلوژرها، `Fn`/`FnMut`/`FnOnce` و `move`](02-iterators-and-closures/01-closures-and-fn-traits/README.fa.md)
۲. [ترکیب‌گرهای ایتریتور](02-iterators-and-closures/02-iterator-adapters/README.fa.md)
۳. [مصرف و جمع‌آوری، از جمله `Result<Vec<_>, E>`](02-iterators-and-closures/03-consuming-and-collecting/README.fa.md)
۴. [پیاده‌سازیِ `Iterator` و `IntoIterator` برای نوعِ خودت](02-iterators-and-closures/04-implementing-iterator/README.fa.md)
۵. [تنبلی و کاراییِ ایتریتورها](02-iterators-and-closures/05-laziness-and-performance/README.fa.md)

## ۳. [صفت‌ها و جنریک‌ها](03-traits-and-generics/README.fa.md)

چندریختی بدونِ ارث‌بری، جنریک بدونِ هزینه در زمانِ اجرا.

۱. [تعریف و پیاده‌سازیِ صفت‌ها](03-traits-and-generics/01-defining-and-implementing-traits/README.fa.md)
۲. [توابع و ساختارهای جنریک، کران‌ها، `where`](03-traits-and-generics/02-generic-functions-and-structs/README.fa.md)
۳. [`From`، `Into`، `TryFrom`، `TryInto`](03-traits-and-generics/03-from-into-tryfrom/README.fa.md)
۴. [مشتق‌های استاندارد، دستی پیاده‌سازی‌شده](03-traits-and-generics/04-standard-derives-by-hand/README.fa.md)
۵. [نوع‌های وابسته در برابرِ پارامترهای جنریک](03-traits-and-generics/05-associated-types/README.fa.md)
۶. [ابرصفت‌ها، پیاده‌سازیِ فراگیر، قاعده‌ی یتیم و راهِ فرارِ newtype](03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.fa.md)
۷. [ارسالِ ایستا در برابرِ پویا، و ایمنیِ شیء](03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)

## ۴. [طول‌عمرها و تبدیل](04-lifetimes-and-conversion/README.fa.md)

اسم گذاشتن روی مدتِ زندگیِ یک قرض، و اینکه این اسم چه چیزی نصیبت می‌کند.

۱. [مبانیِ طول‌عمر و حذفِ آن](04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.fa.md)
۲. [طول‌عمر در ساختارها و متدها](04-lifetimes-and-conversion/02-lifetimes-in-structs-and-methods/README.fa.md)
۳. [`Deref`، `AsRef`، `Borrow`، `ToOwned`](04-lifetimes-and-conversion/03-deref-asref-borrow/README.fa.md)
۴. [`Cow<'_, str>` و کپی‌هنگامِ‌نوشتن](04-lifetimes-and-conversion/04-cow-and-clone-on-write/README.fa.md)

## ۵. [مدیریتِ خطا](05-error-handling/README.fa.md)

خودِ *نوعِ* خطا، نه فقط `Result` و `?`.

۱. [نوع‌های خطای سفارشی و `std::error::Error`](05-error-handling/01-custom-error-types/README.fa.md)
۲. [زنجیره‌ی منشأ و `Box<dyn Error>`](05-error-handling/02-error-source-chains/README.fa.md)
۳. [`thiserror` در برابرِ `anyhow` و مرزِ کتابخانه/باینری](05-error-handling/03-thiserror-and-anyhow/README.fa.md)
۴. [طراحیِ رده‌بندیِ خطا برای یک سرویس](05-error-handling/04-error-taxonomy-for-a-service/README.fa.md)

## ۶. [اشاره‌گرهای هوشمند و وضعیتِ اشتراکی](06-smart-pointers/README.fa.md)

هر راهِ مجازِ دور زدنِ «همیشه یک مالک».

۱. [`Box` و تخصیصِ هیپ](06-smart-pointers/01-box-and-heap-allocation/README.fa.md)
۲. [نوع‌های بازگشتی و شیء‌های صفتِ جعبه‌ای](06-smart-pointers/02-recursive-types-and-trait-objects/README.fa.md)
۳. [`Rc` و `Arc`](06-smart-pointers/03-rc-and-arc/README.fa.md)
۴. [`Weak` و چرخه‌های ارجاع](06-smart-pointers/04-weak-and-reference-cycles/README.fa.md)
۵. [`RefCell`، `Cell` و معامله‌ی پنیکِ زمانِ اجرا](06-smart-pointers/05-refcell-and-interior-mutability/README.fa.md)

## ۷. [ساختارِ پروژه و تست](07-project-structure-and-testing/README.fa.md)

همان چیزی که وقتی پروژه از یک فایل بزرگ‌تر می‌شود، عوض می‌شود.

۱. [ماژول‌ها، دیدپذیری، صادرِ دوباره، ورک‌اسپیس‌ها](07-project-structure-and-testing/01-modules-visibility-workspaces/README.fa.md)
۲. [تستِ واحد، یکپارچه و مستندات](07-project-structure-and-testing/02-unit-integration-doc-tests/README.fa.md)
۳. [بدل‌های تست در Rust، و چرا به‌ندرت به فریم‌ورکِ mock احتیاج داری](07-project-structure-and-testing/03-test-doubles-in-rust/README.fa.md)
۴. [تستِ خاصیت‌محور با `proptest`، تستِ عکس‌برداری با `insta`](07-project-structure-and-testing/04-property-and-snapshot-testing/README.fa.md)
۵. [سنجشِ کارایی با `criterion`](07-project-structure-and-testing/05-benchmarking-with-criterion/README.fa.md)

## ۸. [هم‌روندی](08-concurrency/README.fa.md)

همان ماژولی که قاعده‌ی هم‌نامی برایش بود.

۱. [ریسه‌ها، `Mutex`، `Arc`](08-concurrency/01-threads-mutex-arc/README.fa.md)
۲. [`RwLock`، `Semaphore`، `OnceLock`/`LazyLock`، اتمیک‌ها](08-concurrency/02-rwlock-semaphore-oncelock-atomics/README.fa.md)
۳. [کانال‌ها و پیام‌رسانی](08-concurrency/03-channels-message-passing/README.fa.md)
۴. [`Send` و `Sync`: چه هستند و چرا نوعِ تو `Send` نیست](08-concurrency/04-send-and-sync/README.fa.md)
۵. [فیوچرها و رانتایم‌ها: `async fn` به چه تبدیل می‌شود](08-concurrency/05-futures-and-runtimes/README.fa.md)
۶. [مقدماتِ `tokio`](08-concurrency/06-tokio-basics/README.fa.md)

## ۹. [async در عمل](09-async-in-practice/README.fa.md)

ساختن با `async fn`، نه فقط تماشای کارکردنش.

۱. [`spawn`، `JoinSet`، هم‌روندیِ ساخت‌یافته](09-async-in-practice/01-spawn-joinset-structured-concurrency/README.fa.md)
۲. [`select!` و ایمنیِ لغو](09-async-in-practice/02-select-and-cancellation-safety/README.fa.md)
۳. [استریم‌ها: `futures::Stream` و `tokio-stream`](09-async-in-practice/03-streams/README.fa.md)
۴. [صفت‌های async و `spawn_blocking`](09-async-in-practice/04-async-traits-and-blocking/README.fa.md)

## ۱۰. [جعبه‌ابزارِ Rust](10-rust-toolbox/README.fa.md)

چهار چیز که مدام سراغشان می‌روی، و به هیچ ماژولِ قبلی تنها تعلق ندارند.

۱. [تطبیقِ الگو از نزدیک](10-rust-toolbox/01-pattern-matching-depth/README.fa.md)
۲. [مقدماتِ `macro_rules!`](10-rust-toolbox/02-macro-rules-basics/README.fa.md)
۳. [ویژگی‌های cargo و کامپایلِ شرطی](10-rust-toolbox/03-cargo-features/README.fa.md)
۴. [`unsafe` به‌طورِ جدی: اشاره‌گرِ خام، UB، حفظِ ناوردا](10-rust-toolbox/04-unsafe-for-real/README.fa.md)

---

**ایستگاه انگیزشی:** [مأموریت جانبی ۲ — ربات تلگرامی کوییز](../side-quests/sq-02-telegram-quiz-bot/README.fa.md)
— اولین پروژه‌ی واقعیِ async تو.

وقتی فاز ۲ در [`PROGRESS.fa.md`](../PROGRESS.fa.md) تیک خورد، برو سراغِ [فاز ۳](../phase3-backend-foundations/README.fa.md).
