# Checkpoint

1. `echo_rejects_a_malformed_json_body_before_the_handler_runs` never calls
   your `echo` function's body at all when the request body isn't valid
   JSON — `axum` returns an error response on its own. In Django/DRF terms,
   what would the equivalent view code look like *without* this automatic
   behavior (i.e. what would you have to write yourself to get the same
   guarantee)?
2. `AppState` wraps the counter in `Arc<Mutex<i64>>`, not a plain `i64`.
   `State<AppState>` extraction gives each handler invocation its own
   *clone* of `AppState`. Why does cloning `AppState` still let two
   concurrent requests see (and safely mutate) the *same* counter, instead
   of each getting an independent copy starting back at whatever value it
   was when they were built?
3. Handlers are declared `async fn` even though none of them `.await`
   anything inside their bodies yet. What's a category of thing a handler
   *will* need to `.await` starting in module 4, and why couldn't a
   synchronous (non-`async`) function do that without blocking the whole
   `tokio` runtime thread it's running on?
4. `greet`'s signature is `Path(name): Path<String>` — a *pattern* in the
   parameter position, not just a type. What is `axum` actually handing
   the function here before that pattern destructures it (hint: it's not a
   bare `String`)?
5. `unknown_route_returns_404` gets a `404` with zero custom code from you
   — you never wrote a "no route matched" handler. Contrast this with
   Django's `urls.py`: does an unmatched URL there require you to write
   anything either, or is it also automatic? What *would* require custom
   code in both frameworks (hint: think about a route that exists but was
   hit with the wrong HTTP method).
