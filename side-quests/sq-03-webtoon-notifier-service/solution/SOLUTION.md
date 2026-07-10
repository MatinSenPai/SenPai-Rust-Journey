# Solution

## `check_all_webtoons` — the actual notifier logic

```rust
pub async fn check_all_webtoons(
    store: &WebtoonStore,
    checker: &dyn ChapterChecker,
) -> Vec<NewChapterEvent> {
    let mut events = Vec::new();
    for webtoon in store.list() {
        if let Some(new_chapter) = checker.check(&webtoon).await {
            if new_chapter > webtoon.current_chapter
                && store.update_chapter(webtoon.id, new_chapter).is_some()
            {
                tracing::info!(
                    webtoon_id = webtoon.id,
                    title = %webtoon.title,
                    old_chapter = webtoon.current_chapter,
                    new_chapter,
                    "new chapter detected"
                );
                events.push(NewChapterEvent {
                    id: webtoon.id,
                    title: webtoon.title.clone(),
                    old_chapter: webtoon.current_chapter,
                    new_chapter,
                });
            }
        }
    }
    events
}
```

Three things matter here, each pinned by a test in `notifier_test.rs`:

1. **`store.list()` is snapshotted once, up front**, not re-read inside the
   loop — each `webtoon` here is a plain owned value, not a live view into
   the store, so `webtoon.current_chapter` in the comparison below is
   "what we knew when this round started," which is exactly what we want
   to compare a fresh check result against.
2. **`new_chapter > webtoon.current_chapter`, not `Some(_)`.** A checker
   answering "yes, chapter 42 is out" when we're already on chapter 42
   (`SameChapterChecker` in the tests) isn't news — treating any `Some` as
   "new" would re-log and "re-detect" the same chapter forever if a
   checker ever repeats an answer, which a real API-backed checker
   absolutely could do (e.g. a cached/stale response from the upstream
   source).
3. **`store.update_chapter(...).is_some()` gates whether an event is
   recorded.** Between listing the webtoon and finishing its check, it's
   possible (in a concurrent, multi-request world — not exercised by the
   single-threaded tests here, but true in principle) that the webtoon was
   un-followed. `update_chapter` returning `None` means there's nothing
   left to update, so nothing gets logged or reported for it.

## `check_webtoon` — the on-demand handler

```rust
pub async fn check_webtoon(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<CheckResult>, WebtoonError> {
    let webtoon = state.store.get(id).ok_or(WebtoonError::NotFound)?;
    let found = state.checker.check(&webtoon).await;

    match found {
        Some(new_chapter) if new_chapter > webtoon.current_chapter => {
            let updated = state
                .store
                .update_chapter(id, new_chapter)
                .expect("just fetched this webtoon, it must still exist");
            Ok(Json(CheckResult { webtoon: updated, new_chapter_found: true }))
        }
        _ => Ok(Json(CheckResult { webtoon, new_chapter_found: false })),
    }
}
```

Same "strictly greater" rule as `check_all_webtoons`, applied to one
webtoon instead of every followed one — deliberately duplicated logic
rather than routing this handler through `check_all_webtoons` itself,
because `check_all_webtoons` always checks *every* followed webtoon and
this endpoint's whole point is checking exactly *one*, on demand, without
touching the others.

## `follow_webtoon` and `app`

```rust
pub async fn follow_webtoon(
    State(state): State<AppState>,
    Json(input): Json<FollowWebtoon>,
) -> (StatusCode, Json<Webtoon>) {
    let webtoon = state.store.follow(input);
    (StatusCode::CREATED, Json(webtoon))
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/webtoons", get(list_webtoons).post(follow_webtoon))
        .route("/webtoons/{id}/check", post(check_webtoon))
        .with_state(state)
}
```

Nothing new here — the same "201 on a successful create, `MethodRouter`
chaining for a shared path" pattern as Phase 3's anime catalog CRUD lesson.

## Why `#[async_trait]` here, unlike the cache-aside lesson's `Cache` trait

The cache-aside lesson's `Cache` trait uses native `async fn` in traits
(`fn get(&self, ...) -> impl Future<Output = ...>`) and explicitly avoids
`#[async_trait]`, because every caller there is generic over `C: Cache` —
nobody ever needs to store "some `Cache` implementation, I don't know
which one, at runtime" behind a single concrete type.

`ChapterChecker` is different: `AppState` needs to hold **one** checker
that's shared between the axum handlers (`check_webtoon`) and the
background job (`spawn_notifier`), and `main.rs` picks which concrete
checker to use once, at startup (`RandomChapterChecker` for the real run,
a deterministic one in tests) — that's exactly "I need a trait object,"
i.e. `Arc<dyn ChapterChecker>`. Native RPITIT (`impl Future<Output = ...>`
as a trait method's return type) isn't `dyn`-safe — the compiler can't
build a vtable for a method whose return type's size isn't known until
monomorphization — so `#[async_trait]` (which desugars `async fn` into a
`fn(...) -> Pin<Box<dyn Future<Output = ...> + Send>>` under the hood,
a size the vtable *can* describe) is what makes `dyn ChapterChecker`
possible at all.

## Why the background loop calls `check_all_webtoons` instead of embedding its logic

```rust
pub fn spawn_notifier(
    store: Arc<WebtoonStore>,
    checker: Arc<dyn ChapterChecker>,
    period: Duration,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(period);
        loop {
            interval.tick().await;
            let events = check_all_webtoons(&store, checker.as_ref()).await;
            // ...log each event...
        }
    })
}
```

If the comparison-and-update logic lived directly inside this `loop`
block, testing it would mean either (a) actually waiting through real
`tokio::time::interval` ticks in a test — slow, and the first tick doesn't
fire until `period` has elapsed, so even a "does this loop work at all"
test would need a real sleep — or (b) reaching for `tokio::time::pause()`
and manually advancing virtual time, which works but adds ceremony for no
benefit here. Factoring the interesting part out as `check_all_webtoons`,
a plain `async fn` that takes `&WebtoonStore` and `&dyn ChapterChecker` and
returns a `Vec<NewChapterEvent>`, means `notifier_test.rs` can call it
directly, `.await` it once, and assert on the result immediately — no
timer involved at all, because the timer was never the thing worth
testing in the first place. `spawn_notifier` itself is *only* timer
plumbing and logging by the time this split is done, which is exactly why
it's given to you fully implemented instead of being an exercise.

## `WebtoonStore` staying in-memory

The Phase 3 README blurb name-drops Postgres for this project, and by the
time you've finished `04-postgres-and-sqlx/03-anime-catalog-postgres-backed`
you'll have built the real, durable version of exactly this pattern —
`AnimeStore` rebuilt against `sqlx::PgPool` instead of a `Mutex<HashMap>`.
This side-quest deliberately stays in-memory, for the same reason the
cache-aside lesson's tests never touch a live Redis: a side-quest's `cargo
test` (and a learner's first `cargo run`) should never depend on an
external service being up. If you do want to harden this into something
that survives a restart, the shape of the change is small precisely
*because* `WebtoonStore`'s public methods (`follow`, `get`, `list`,
`update_chapter`) are the only thing anything else in this crate talks to
— swap the `Mutex<HashMap<...>>` internals for `sqlx` queries against a
`webtoons` table and nothing in `lib.rs` outside `WebtoonStore` itself, or
in `tests/api_test.rs`/`tests/notifier_test.rs`, needs to change at all.
