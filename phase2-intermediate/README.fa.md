# فاز دو — Rust میانی و اصطلاحی

فاز یک به تو یاد داد کامپایلر را راضی کنی؛ فاز دو یاد می‌دهد مانند یک مهندس باتجربه Rust بنویسی: generic، مبتنی بر trait، testable و در پایان concurrent و async. قطعات لازم برای بک‌اند واقعی نیز اینجا کنار هم می‌آیند.

1. **مجموعه‌ها** — [`Vec` و `HashMap`](01-collections/01-vec-and-hashmap/README.md)، سپس [`BTreeMap`، `HashSet` و `VecDeque`](01-collections/02-btreemap-hashset-vecdeque/README.md)
2. **iterator و closure** — [closure و traitهای `Fn`](02-iterators-and-closures/01-closures-and-fn-traits/README.md)، [adapterهای iterator](02-iterators-and-closures/02-iterator-adapters/README.md)
3. **generic و trait** — [تابع/struct generic](03-generics-and-traits/01-generic-functions-and-structs/README.md)، [تعریف trait](03-generics-and-traits/02-defining-and-implementing-traits/README.md)، [dynamic/static dispatch](03-generics-and-traits/03-trait-objects-vs-static-dispatch/README.md)
4. **خطا و طول عمر** — [خطای سفارشی](04-error-handling-and-lifetimes/01-custom-error-types/README.md)، [`thiserror` و `anyhow`](04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.md)، [lifetime](04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)
5. **اشاره‌گر هوشمند** — [`Box`](05-smart-pointers/01-box-and-heap-allocation/README.md)، [`Rc` و `Arc`](05-smart-pointers/02-rc-and-arc/README.md)، [`RefCell`](05-smart-pointers/03-refcell-and-interior-mutability/README.md)
6. **ساختار پروژه و آزمون** — [module/workspace](06-project-organization-and-testing/01-modules-visibility-workspaces/README.md)، [گونه‌های آزمون](06-project-organization-and-testing/02-unit-integration-doc-tests/README.md)، [`unsafe`](06-project-organization-and-testing/03-unsafe-rust-overview/README.md)
7. **concurrency و async** — [thread](07-concurrency-and-async/01-threads-mutex-arc/README.md)، [channel](07-concurrency-and-async/02-channels-message-passing/README.md)، [Future/runtime](07-concurrency-and-async/03-futures-and-runtimes/README.md)، [Tokio](07-concurrency-and-async/04-tokio-basics/README.md)
8. **جعبه‌ابزار Rust** — [نمای کلی](08-rust-toolbox/README.md)

پس از تکمیل فاز، [ربات مسابقه‌ی تلگرام](../side-quests/sq-02-telegram-quiz-bot/README.md) را بساز و وارد [فاز سه](../phase3-backend-foundations/README.md) شو.

```senpai-visual
{"kind":"roadmap","labels":["collections","traits","lifetimes","async"]}
```
