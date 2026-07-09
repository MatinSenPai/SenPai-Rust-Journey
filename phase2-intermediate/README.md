# Phase 2 — Intermediate & Idiomatic Rust

Phase 1 taught you to satisfy the compiler. Phase 2 teaches you to write
Rust the way experienced Rust engineers actually write it: generic,
trait-based, testable, and — by the end — concurrent and asynchronous. This
is also the phase where the pieces you'll need for a real backend
(Phase 3-4) start falling into place.

1. **Collections**
   - [01 — `Vec` and `HashMap`](01-collections/01-vec-and-hashmap/README.md)
   - [02 — `BTreeMap`, `HashSet`, `VecDeque`](01-collections/02-btreemap-hashset-vecdeque/README.md)
2. **Iterators & closures**
   - [01 — Closures and `Fn` traits](02-iterators-and-closures/01-closures-and-fn-traits/README.md)
   - [02 — Iterator adapters](02-iterators-and-closures/02-iterator-adapters/README.md)
3. **Generics & traits**
   - [01 — Generic functions and structs](03-generics-and-traits/01-generic-functions-and-structs/README.md)
   - [02 — Defining and implementing traits](03-generics-and-traits/02-defining-and-implementing-traits/README.md)
   - [03 — Trait objects vs. static dispatch](03-generics-and-traits/03-trait-objects-vs-static-dispatch/README.md)
4. **Error handling & lifetimes**
   - [01 — Custom error types](04-error-handling-and-lifetimes/01-custom-error-types/README.md)
   - [02 — `thiserror` and `anyhow`](04-error-handling-and-lifetimes/02-thiserror-and-anyhow/README.md)
   - [03 — Lifetime basics and elision](04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)
5. **Smart pointers**
   - [01 — `Box` and heap allocation](05-smart-pointers/01-box-and-heap-allocation/README.md)
   - [02 — `Rc` and `Arc`](05-smart-pointers/02-rc-and-arc/README.md)
   - [03 — `RefCell` and interior mutability](05-smart-pointers/03-refcell-and-interior-mutability/README.md)
6. **Project organization & testing**
   - [01 — Modules, visibility, workspaces](06-project-organization-and-testing/01-modules-visibility-workspaces/README.md)
   - [02 — Unit, integration & doc tests](06-project-organization-and-testing/02-unit-integration-doc-tests/README.md)
   - [03 — `unsafe` Rust overview](06-project-organization-and-testing/03-unsafe-rust-overview/README.md)
7. **Concurrency & async**
   - [01 — Threads, `Mutex`, `Arc`](07-concurrency-and-async/01-threads-mutex-arc/README.md)
   - [02 — Channels and message passing](07-concurrency-and-async/02-channels-message-passing/README.md)
   - [03 — Futures and runtimes](07-concurrency-and-async/03-futures-and-runtimes/README.md)
   - [04 — Tokio basics](07-concurrency-and-async/04-tokio-basics/README.md)

**Motivational checkpoint:** [Side-quest 2 — Telegram Quiz Bot](../side-quests/sq-02-telegram-quiz-bot/README.md)
— your first real async project.

When Phase 2 is fully checked off in [`PROGRESS.md`](../PROGRESS.md), move on
to [Phase 3](../phase3-backend-foundations/README.md), where all of this
starts turning into an actual backend.
