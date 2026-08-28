# Phase 2 — Intermediate & Idiomatic Rust

Phase 1 taught you to satisfy the compiler. Phase 2 teaches you to write
Rust the way experienced Rust engineers actually write it: iterators
instead of index loops, traits instead of duplicated functions, errors that
carry real information, and — by the end — the concurrency model that made
learning this language worth it in the first place.

Forty-eight lessons in ten modules, in the order they are meant to be read.

## 1. [Collections](01-collections/README.md)

What is actually inside `Vec` and `HashMap`, and what else is on the shelf.

1. [`Vec` in depth: capacity, `retain`, `drain`, `dedup`, `binary_search`](01-collections/01-vec-depth/README.md)
2. [`HashMap` in depth: the `entry` API, hashers, `&str` lookup](01-collections/02-hashmap-in-depth/README.md)
3. [`BTreeMap`, `HashSet`, `VecDeque`, `BinaryHeap`](01-collections/03-btreemap-hashset-vecdeque/README.md)
4. [Choosing a collection: a complexity table and when each wins](01-collections/04-choosing-a-collection/README.md)

## 2. [Iterators and closures](02-iterators-and-closures/README.md)

The biggest shift in how idiomatic Rust reads.

1. [Closures, `Fn`/`FnMut`/`FnOnce`, and `move`](02-iterators-and-closures/01-closures-and-fn-traits/README.md)
2. [Iterator adapters](02-iterators-and-closures/02-iterator-adapters/README.md)
3. [Consuming and collecting, including `Result<Vec<_>, E>`](02-iterators-and-closures/03-consuming-and-collecting/README.md)
4. [Implementing `Iterator` and `IntoIterator` for your own type](02-iterators-and-closures/04-implementing-iterator/README.md)
5. [Laziness and iterator performance](02-iterators-and-closures/05-laziness-and-performance/README.md)

## 3. [Traits and generics](03-traits-and-generics/README.md)

Polymorphism without inheritance, generic without paying for it at run time.

1. [Defining and implementing traits](03-traits-and-generics/01-defining-and-implementing-traits/README.md)
2. [Generic functions and structs, bounds, `where`](03-traits-and-generics/02-generic-functions-and-structs/README.md)
3. [`From`, `Into`, `TryFrom`, `TryInto`](03-traits-and-generics/03-from-into-tryfrom/README.md)
4. [The standard derives, implemented by hand](03-traits-and-generics/04-standard-derives-by-hand/README.md)
5. [Associated types versus generic parameters](03-traits-and-generics/05-associated-types/README.md)
6. [Supertraits, blanket impls, the orphan rule and the newtype escape hatch](03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md)
7. [Static versus dynamic dispatch, and object safety](03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)

## 4. [Lifetimes and conversion](04-lifetimes-and-conversion/README.md)

Naming how long a borrow lives, and what that buys you.

1. [Lifetime basics and elision](04-lifetimes-and-conversion/01-lifetime-basics-and-elision/README.md)
2. [Lifetimes in structs and methods](04-lifetimes-and-conversion/02-lifetimes-in-structs-and-methods/README.md)
3. [`Deref`, `AsRef`, `Borrow`, `ToOwned`](04-lifetimes-and-conversion/03-deref-asref-borrow/README.md)
4. [`Cow<'_, str>` and copy-on-write](04-lifetimes-and-conversion/04-cow-and-clone-on-write/README.md)

## 5. [Error handling](05-error-handling/README.md)

The error *type*, not just `Result` and `?`.

1. [Custom error types and `std::error::Error`](05-error-handling/01-custom-error-types/README.md)
2. [Source chains and `Box<dyn Error>`](05-error-handling/02-error-source-chains/README.md)
3. [`thiserror` versus `anyhow`, and the library/binary boundary](05-error-handling/03-thiserror-and-anyhow/README.md)
4. [Designing an error taxonomy for a service](05-error-handling/04-error-taxonomy-for-a-service/README.md)

## 6. [Smart pointers and shared state](06-smart-pointers/README.md)

Every sanctioned way around "one owner, always."

1. [`Box` and heap allocation](06-smart-pointers/01-box-and-heap-allocation/README.md)
2. [Recursive types and boxed trait objects](06-smart-pointers/02-recursive-types-and-trait-objects/README.md)
3. [`Rc` and `Arc`](06-smart-pointers/03-rc-and-arc/README.md)
4. [`Weak` and reference cycles](06-smart-pointers/04-weak-and-reference-cycles/README.md)
5. [`RefCell`, `Cell`, and the run-time panic trade](06-smart-pointers/05-refcell-and-interior-mutability/README.md)

## 7. [Project structure and testing](07-project-structure-and-testing/README.md)

What changes once a project outgrows one file.

1. [Modules, visibility, re-exports, workspaces](07-project-structure-and-testing/01-modules-visibility-workspaces/README.md)
2. [Unit, integration and doc tests](07-project-structure-and-testing/02-unit-integration-doc-tests/README.md)
3. [Test doubles in Rust, and why you rarely need a mocking framework](07-project-structure-and-testing/03-test-doubles-in-rust/README.md)
4. [Property testing with `proptest`, snapshot testing with `insta`](07-project-structure-and-testing/04-property-and-snapshot-testing/README.md)
5. [Benchmarking with `criterion`](07-project-structure-and-testing/05-benchmarking-with-criterion/README.md)

## 8. [Concurrency](08-concurrency/README.md)

The module the aliasing rule was for.

1. [Threads, `Mutex`, `Arc`](08-concurrency/01-threads-mutex-arc/README.md)
2. [`RwLock`, `Semaphore`, `OnceLock`/`LazyLock`, atomics](08-concurrency/02-rwlock-semaphore-oncelock-atomics/README.md)
3. [Channels and message passing](08-concurrency/03-channels-message-passing/README.md)
4. [`Send` and `Sync`: what they are and why your type isn't `Send`](08-concurrency/04-send-and-sync/README.md)
5. [Futures and runtimes: what `async fn` desugars to](08-concurrency/05-futures-and-runtimes/README.md)
6. [`tokio` basics](08-concurrency/06-tokio-basics/README.md)

## 9. [Async in practice](09-async-in-practice/README.md)

Building with `async fn`, not just watching it work.

1. [`spawn`, `JoinSet`, structured concurrency](09-async-in-practice/01-spawn-joinset-structured-concurrency/README.md)
2. [`select!` and cancellation safety](09-async-in-practice/02-select-and-cancellation-safety/README.md)
3. [Streams: `futures::Stream` and `tokio-stream`](09-async-in-practice/03-streams/README.md)
4. [Async traits and `spawn_blocking`](09-async-in-practice/04-async-traits-and-blocking/README.md)

## 10. [The Rust toolbox](10-rust-toolbox/README.md)

Four things you will reach for constantly, that belong to no earlier module.

1. [Pattern matching in depth](10-rust-toolbox/01-pattern-matching-depth/README.md)
2. [`macro_rules!` basics](10-rust-toolbox/02-macro-rules-basics/README.md)
3. [Cargo features and conditional compilation](10-rust-toolbox/03-cargo-features/README.md)
4. [`unsafe` for real: raw pointers, UB, upholding invariants](10-rust-toolbox/04-unsafe-for-real/README.md)

---

**Motivational recall questions:** [Side-quest 2 — Telegram Quiz Bot](../side-quests/sq-02-telegram-quiz-bot/README.md)
— your first real async project.

When Phase 2 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on
to [Phase 3](../phase3-backend-foundations/README.md), where all of this
starts turning into an actual backend.
