# Solution — 3.4.2 Application state and dependency wiring

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/`, including `build_test.rs` for the Build rung.

## Repair

- `04`: `#[derive(Clone)]` on `AppState`.
- `05`: `trait Clock: Send + Sync`, so every implementor is checked at its `impl`.
- `06`: `#[derive(Clone, FromRef)]` on `AppState` (import `axum::extract::FromRef`). The handler stays as it was.
- `07`: one newtype per counter (`struct Hits(Arc<AtomicU32>)`, `struct Misses(Arc<AtomicU32>)`) and the `hits` handler takes `State<Hits>`.

## The clocks

```rust
impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}
```

`duration_since` fails only when the system clock is before 1970, and the spec says that answers `0`, so the error is mapped away instead of unwrapped.

```rust
impl FakeClock {
    pub fn at(secs: u64) -> Self { Self { secs: AtomicU64::new(secs) } }
    pub fn advance(&self, secs: u64) { self.secs.fetch_add(secs, Ordering::SeqCst); }
}
impl Clock for FakeClock {
    fn now(&self) -> u64 { self.secs.load(Ordering::SeqCst) }
}
```

An atomic is interior mutability that needs no lock, so `advance` can take `&self`. That is the whole point of the design: the test keeps an `Arc<FakeClock>`, the app holds a clone of it, and `advance` on one is seen by the other (`every_holder_of_the_same_fake_clock_sees_the_same_time`). A `Cell` would not do: it is not `Sync`, so the fake would not satisfy `Clock: Send + Sync`.

## The store

```rust
pub fn add(&self, title: &str, at: u64) -> Watch {
    let mut entries = self.entries.lock().unwrap();
    let watch = Watch { id: entries.len() as u64 + 1, title: title.to_string(), watched_at: at };
    entries.push(watch.clone());
    watch
}
```

The id comes from the length, read under the same lock as the push, so two concurrent adds cannot get the same id. That is only safe because entries are never removed; with a `DELETE` you would need a separate counter, as 3.2.3's store has. `since` filters with `watched_at >= cutoff` (the boundary is inclusive, `since_keeps_entries_at_or_after_the_cutoff`) and keeps insertion order, which is not time order (`since_keeps_the_order_entries_were_added_in`). `count` is `lock().unwrap().len()`.

## The state and the handlers

```rust
impl AppState {
    pub fn new(clock: Arc<dyn Clock>, config: Config) -> Self {
        Self { clock, store: Arc::new(WatchStore::default()), config }
    }
}
```

`new` creates the store itself: callers choose the clock and the settings, which are the things that differ between `main` and a test.

```rust
let title = input.title.trim();
if title.is_empty() || title.chars().count() > config.max_title_len {
    return Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid title"));
}
Ok((StatusCode::CREATED, Json(store.add(title, clock.now()))))
```

The title is trimmed first and the *trimmed* one is stored (`the_title_is_trimmed_before_it_is_stored`). The limit is counted in characters, not bytes: `title.len()` would reject `"é é é"` at `max_title_len = 5` (`the_title_limit_counts_characters_not_bytes`). The failure is `422`, not `400`: the JSON was fine and the content broke a rule (3.1.3). The handler never touches `AppState`; it takes three `State`s, each produced by the derived `FromRef`.

```rust
let cutoff = clock.now().saturating_sub(config.recent_window_secs);
Json(store.since(cutoff))
```

`saturating_sub` is the one trap: a fake clock at `10` with a window of `3600` would underflow a `u64` (a panic in debug builds, a huge cutoff that hides everything in release). Saturating to `0` means "everything counts" (`a_clock_smaller_than_the_window_does_not_underflow`). Because the cutoff is `now - window` and the filter is `>=`, an entry exactly one window old still counts (`an_entry_exactly_one_window_old_still_counts`).

```rust
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/watch", post(add_watch))
        .route("/watch/recent", get(recent_watches))
        .with_state(state)
}
```

A method nobody registered, such as `DELETE /watch`, answers `405` on its own.

## Build: a second dependency

```rust
pub trait Notifier: Send + Sync {
    fn notify(&self, message: &str);
}

pub fn with_notifier(mut self, notifier: Arc<dyn Notifier>) -> Self {
    self.notifier = notifier;
    self
}
```

`AppState::new` fills the new field with `NullNotifier`, so every test written before the Build rung keeps compiling, and `with_notifier` swaps it. The field is another `Arc<dyn ...>`, so the derive gives handlers `State<Arc<dyn Notifier>>` for free: no other wiring changes. `add_watch` calls `notifier.notify(&format!("watched: {}", watch.title))` after the store call and only on the success path, so a rejected title (which returns before reaching it) notifies nobody. The test double is a `Mutex<Vec<String>>` behind the trait: a spy in the glossary's terms.

## Challenge: a generic clock

There is no single answer, but expect these findings. `AppState<C>` needs `C: Clock` wherever it is used: `app<C>`, every handler, and any extractor that names the state. `#[derive(FromRef)]` does not work on it at all (the macro answers "`#[derive(FromRef)]` doesn't support generics"), so each `impl<C: Clock> FromRef<AppState<C>> for ...` is written by hand. Handlers that take `State<Arc<C>>` have to be generic too. `#[derive(Clone)]` on `AppState<C>` adds `C: Clone`, which `Arc<C>` never needed, so you bound or implement `Clone` by hand. The payoff is that `now()` is a direct, inlinable call; for a clock read once per request that saves nothing measurable, which is the case for keeping `Arc<dyn Clock>` here.
