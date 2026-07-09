# Checkpoint

1. `required_config` and `average_of_nonempty` both panic on invalid input,
   but for different *kinds* of reasons. Explain the difference between "a
   missing deployment config" and "an internal invariant violated by a
   caller bug" in your own words — why does panicking make sense for both,
   even though one is closer to "expected at the edges of the system" and
   the other is purely internal?
2. Imagine `parse_user_age` used `.unwrap()` internally instead of
   returning `Result`. If this were wired into a real web server (Phase 3),
   what would happen to a request that sent an invalid age? What about to
   *other, unrelated* requests being handled concurrently at the same time?
3. `#[should_panic(expected = "...")]` is a real, useful testing tool for
   code that's *supposed* to panic under specific conditions. Why is testing
   "this panics with a specific, informative message" better than just
   testing "this panics" (no `expected` string)?
4. Look back at every `.unwrap()` and `.expect()` you've written across
   Phase 0 and Phase 1's exercises so far. Pick one and decide: was `panic!`
   actually the right call there, or should it have been `Result`? Why?
