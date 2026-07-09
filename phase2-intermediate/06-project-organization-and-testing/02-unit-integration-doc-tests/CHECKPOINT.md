# Checkpoint

1. Add a line to `tests/conversion_test.rs` calling
   `round_to_one_decimal(1.0)` directly. What's the exact compiler error?
   Does it say "private," or something else — and either way, what is it
   actually telling you?
2. Run `cargo test -p p2-06-02-unit-integration-doc-tests` and find the
   "Doc-tests" section in the output. How many tests ran there, and where
   did they come from — you never wrote a `#[test]` function for them
   directly.
3. Delete the `# ` before the `use` line in the doc comment's example (so
   it reads `use p2_06_02_unit_integration_doc_tests::celsius_to_fahrenheit;`
   without the hash) and regenerate docs with `cargo doc -p
   p2-06-02-unit-integration-doc-tests --open` (or just reason about it if
   you can't open a browser here). What's different about what a reader of
   the generated documentation sees?
4. Why do integration tests (`tests/`) matter *in addition to* unit tests,
   given that unit tests can already call every public function? What
   category of bug would only an integration test (not a unit test) catch?
