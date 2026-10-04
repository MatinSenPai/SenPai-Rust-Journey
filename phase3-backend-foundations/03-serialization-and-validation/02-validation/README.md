# 3.3.2 — Validation

## At a glance

After this lesson you can:

- Put rules on a type with `#[derive(Validate)]` (length, range, email, your own function, nested structs and lists) and read the `ValidationErrors` tree that `validate()` returns.
- Flatten that tree into a field-keyed JSON body (`{"errors": {"reviewer.email": [...]}}`) and answer it as a `422`, so a client can show each message next to its input.
- Say which of the two passes rejects a request: the shape pass (`Json<T>`, `serde`) or the rules pass (`validate()`), and which status each one produces.
- Spot the three mistakes this crate makes easy: a missing derive (`E0599`), a custom rule with the wrong signature (`E0308`), and a nested struct whose rules silently never run.

**Time:** ~90 minutes · **Prerequisites:**
[3.3.1 — Serde in depth](../01-serde-depth/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

In 3.2.3 a rating of `15` came back as a `422`. The rule behind it was one hand-written function in the store, `validate_rating`, called from two methods. That works for one field. A real request body has a dozen fields, some of them nested (a reviewer inside a review, a list of episode notes inside it), and every rule you write by hand is a rule you can forget to call.

In DRF you never wrote that function: `serializers.IntegerField(min_value=1, max_value=10)` is the rule, `serializer.is_valid()` runs all of them, and `serializer.errors` is a dict of `{"field": ["message", ...]}` that goes straight into a `400`. This lesson builds the same three pieces in Rust: rules declared on the type, one call that runs all of them, and an error value that you turn into a response. The Rust version is a separate step from parsing (3.3.1 owns the parsing half), and that separation is the main thing to understand.

The crate is `validator` 0.18. Everything below was run against the resolved version, `0.18.1`.

---

## The concept

### Two passes, two different failures

A request body goes through two checks, and they fail differently:

```senpai-visual
{"kind":"result","labels":["request body","Json of T: is it the right shape?","400 or 422, axum's plain text","validate: do the values obey the rules?","422, your errors object","handler runs"]}
```

The first pass is `Json<T>` from 3.2.1, which uses `serde` (3.3.1): is this JSON, and does it have the fields `T` needs, with the right types? If not, `axum` answers by itself (`400` for broken JSON, `422` for the wrong shape, `415` for a missing `Content-Type`) and your code never runs. The second pass is new: the body already is a `T`, so every type is right, but `rating: 15` is a perfectly good `u8` that still breaks a rule. A type cannot say "a `u8` from 1 to 10", so a second, explicit step does.

DRF does both inside `is_valid()`. Where the analogy stops: here you choose, in your own code, where the second step runs and what its failure looks like on the wire. Nothing happens automatically.

### Rules live on the type: `#[derive(Validate)]`

Show the rules first:

```rust
#[derive(Debug, Validate)]
struct Rating {
    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    score: u8,
    #[validate(length(min = 1, max = 20))]
    title: String,
    #[validate(email)]
    contact: String,
    #[validate(length(max = 5))]
    note: Option<String>,
}
```

`#[derive(Validate)]` generates one method, `validate(&self) -> Result<(), ValidationErrors>`. It runs *every* rule, not just the first that fails. `examples/01-derive-validate.rs` builds a good `Rating`, one whose `note` is too long, and one that breaks three rules at once:

```text
good: Ok(())
long note: true
contact: code=email message=None
score: code=range message=Some("rating must be between 1 and 10")
title: code=length message=None
```

Two things to read off that output. A rule without a `message` still has a `code` (`email`, `length`, `range`): the name of the rule that failed. And `note: Option<String>` passed when it was `None`: a rule on an `Option` runs only when there is a value, which is exactly DRF's `required=False`.

The rules you will use most:

| Rule | On | Example |
|---|---|---|
| `length` | `String`, `Vec`, ... | `length(min = 1, max = 100)` |
| `range` | numbers | `range(min = 1, max = 10)` |
| `email`, `url` | strings | `email` |
| `custom` | anything | `custom(function = "my_rule")` |
| `nested` | a struct, or a `Vec` of them | `nested` |

Every rule takes an optional `message = "..."`. The attribute names are the ones the 0.18 docs list; `custom(function = "...")` takes the function's name *as a string* in this version.

### What `validate()` returns: a tree, not a list

A flat struct gives a flat result. A struct with another struct inside, or a `Vec` of them, gives a tree, and `ValidationErrors` has one node type for each of the three shapes:

```rust
pub enum ValidationErrorsKind {
    Struct(Box<ValidationErrors>),
    List(BTreeMap<usize, Box<ValidationErrors>>),
    Field(Vec<ValidationError>),
}
```

`ValidationErrors` itself wraps a map from field name to one of these. `Field` is the leaf: the list of rules that failed on that field. `Struct` is a nested struct's own `ValidationErrors`. `List` is one `ValidationErrors` per failing item, keyed by the item's index (passing items are simply absent). `examples/02-nested-and-lists.rs` validates a review whose `first_note` and second `notes` item are empty, and prints the tree:

```text
first_note: Struct
  text: Field(1)
notes: List
  [1]
    text: Field(1)
title: Field(1)
```

This is why `errors.field_errors()` is not enough once you nest: it returns only the `Field` entries and silently leaves out every `Struct` and `List` node. To report `first_note` and `notes[1]` you must walk the tree yourself, and that walk is your first Implement exercise.

### Your own rule: a function returning `Result<(), ValidationError>`

When no built-in fits, a rule is a plain function. It takes a reference to the field's value and returns `Ok(())` or an error you build, with a `code` and, if you want one, a `message`:

```rust
fn no_spaces(value: &str) -> Result<(), ValidationError> {
    if value.contains(' ') {
        return Err(ValidationError::new("no_spaces").with_message("no spaces allowed".into()));
    }
    Ok(())
}
// on the field: #[validate(length(min = 3), custom(function = "no_spaces"))]
```

`examples/03-custom-rule.rs` tries three names. The last one, `"m "`, breaks both rules on the same field, so that field gets two errors, and `Debug` shows the `params` each rule recorded:

```text
"matin" -> Ok(())
"ma tin" -> Err(ValidationErrors({"name": Field([ValidationError { code: "no_spaces", message: Some("no spaces allowed"), params: {"value": String("ma tin")} }])}))
"m " -> Err(ValidationErrors({"name": Field([ValidationError { code: "length", message: None, params: {"min": Number(3), "value": String("m ")} }, ValidationError { code: "no_spaces", message: Some("no spaces allowed"), params: {"value": String("m ")} }])}))
```

### From tree to a `422` with a field-keyed body

A client wants to put each message next to the right input, so the body is keyed by field. That is the same shape as DRF's `serializer.errors`, with one choice to make for nested data: here a dot joins a parent to a child (`reviewer.email`) and brackets index a list (`notes[1].text`) (shortened from a real reply you will see in "Hands on"):

```json
{"errors":{"reviewer.email":["email"],"notes[1].text":["note must be 1 to 200 characters"]}}
```

Each field maps to a *list* of messages (one per failed rule), and a message is the rule's `message` if it has one, otherwise its `code`. The status is `422`, by the rule 3.1.3 gave and 3.2.3 applied to `InvalidRating`: the request parsed fine, the content broke a rule.

Turning it into a response is the same move as 3.2.3's `AnimeError`: a small error type that implements `IntoResponse`, plus a `From<ValidationErrors>` so `?` converts for you. This is `examples/07-rating-server.rs`'s handler:

```rust
async fn create(Json(input): Json<NewRating>) -> Result<(StatusCode, String), ApiError> {
    input.validate()?;
    Ok((StatusCode::CREATED, format!("saved {}", input.score)))
}
```

`input.validate()?` returns early with an `ApiError` if any rule failed, and the handler body after it can trust the data. The type system does not know that: `NewRating` is the same type before and after the call, so forgetting the line compiles fine. Build's `ValidatedJson<T>` removes that risk by moving the call into an extractor, so a handler that asks for one cannot be reached with unchecked data.

---

## Hands on

Three of the examples are run in "The concept" above (`01`, `02`, `03`). `06` runs normally and is simply wrong; `04` and `05` are broken on purpose and sit behind the `broken` feature (see "Errors you will meet"). Run the silent one:

```sh
cargo run -p p3-03-02-validation --example 06-forgot-nested-trap
```

```text
outer validate: Ok(())
inner has errors: true
```

`Reviewer` has an `email` rule and the email is not one, yet the outer `validate()` says `Ok`. We come back to this below.

`07` is a real server on `127.0.0.1:3110`. Start it (`cargo run -p p3-03-02-validation --example 07-rating-server`, in its own terminal) and send it four bodies. `-w` prints the status code and content type after each reply:

```sh
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"score":9}' 127.0.0.1:3110/ratings
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"score":15}' 127.0.0.1:3110/ratings
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"score":"nine"}' 127.0.0.1:3110/ratings
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{' 127.0.0.1:3110/ratings
```

```text
saved 9
201 text/plain; charset=utf-8
{"errors":{"score":["score must be between 1 and 10"]}}
422 application/json
Failed to deserialize the JSON body into the target type: score: invalid type: string "nine", expected u8 at line 1 column 15
422 text/plain; charset=utf-8
Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
400 text/plain; charset=utf-8
```

Both `15` and `"nine"` answer `422`, but they are different passes. The first is *your* `ApiError` (JSON, field-keyed); the second is `axum`'s own shape rejection (plain text), and `validate()` never ran. A client cannot tell those two apart by status alone, which is one of the things 3.8.1 fixes. Stop the server when you are done.

The exercises start red. `src/lib.rs` has the whole skeleton, with a `todo!()` and a doc comment that is the complete specification for each function:

```sh
cargo test -p p3-03-02-validation --test validate_test 2>&1 | grep 'test result'
```

```text
test result: FAILED. 0 passed; 10 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

When all tests are green, the finished review API (`POST /reviews` and `GET /reviews`) is in `solution/`, which is its own crate with its own `main`. Run it (`cd solution && cargo run` in its own terminal; it listens on `127.0.0.1:3111`) and send a good review, a review with one broken rule, a review that breaks five rules at once, and one with a field missing:

```sh
U=127.0.0.1:3111/reviews; C='content-type: application/json'
curl -s -X POST -H "$C" -d '{"title":"Frieren","rating":9,"reviewer":{"handle":"matin_01","email":"matin@example.com"}}' $U
curl -s -X POST -H "$C" -d '{"title":"Frieren","rating":15,"reviewer":{"handle":"matin_01","email":"matin@example.com"}}' $U
curl -s -X POST -H "$C" -d '{"title":"","rating":0,"reviewer":{"handle":"a!","email":"nope"},"notes":[{"episode":1,"text":"ok"},{"episode":0,"text":"x"}]}' $U
curl -s -X POST -H "$C" -d '{"title":"x","rating":5}' $U
```

```text
{"id":1,"title":"Frieren","rating":9,"reviewer":"matin_01"}
{"errors":{"rating":["rating must be between 1 and 10"]}}
{"errors":{"notes[1].episode":["range"],"rating":["rating must be between 1 and 10"],"reviewer.email":["email"],"reviewer.handle":["handle may only contain letters, digits and underscores","handle must be 3 to 20 characters"],"title":["title must be 1 to 100 characters"]}}
Failed to deserialize the JSON body into the target type: missing field `reviewer` at line 1 column 24
```

The third reply is the point of the whole lesson: one request, every broken rule reported at once, each under the path of the input that caused it. Notice `reviewer.handle` has two messages, sorted, and `notes[1].episode` says `range` because that rule has no message. Stop the server, then try these:

1. Send a rating of `10` and then `11`. Which one is accepted, and where in the source is that written?
2. Send `"notes": []`. Is it valid? What would make it invalid?
3. Send a good review, then one with a bad title, then another good one. What id does the third request get? (The `a_rejected_review_is_not_stored_and_uses_up_no_id` test checks the idea.)

---

## Errors you will meet

### `E0599` — `validate` on a type that never derived it

```rust
use validator::Validate;

struct Rating {
    score: u8,
}
```

`examples/04-missing-derive-broken.rs` adds a `main` that calls `r.validate()`. Run it with the feature:

```text
error[E0599]: no method named `validate` found for struct `Rating` in the current scope
  --> phase3-backend-foundations\03-serialization-and-validation\02-validation\examples\04-missing-derive-broken.rs:13:24
   |
 7 | struct Rating {
   | ------------- method `validate` not found for this struct
...
13 |     println!("{:?}", r.validate());
   |                        ^^^^^^^^ method not found in `Rating`
   |
   = help: items from traits can only be used if the trait is implemented and in scope
   = note: the following trait defines an item `validate`, perhaps you need to implement it:
           candidate #1: `Validate`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p3-03-02-validation` (example "04-missing-derive-broken") due to 1 previous error
```

**What the compiler is objecting to:** `validate` is a method of the trait `Validate`, and `Rating` does not implement it. Importing the trait (`use validator::Validate;`) is not enough; something must implement it for `Rating`. The `help` and `note` lines are the compiler listing the one trait it knows that has a method with this name.

**The fix:** derive it:

```rust
#[derive(Validate)]
struct Rating {
    #[validate(range(min = 1, max = 10))]
    score: u8,
}
```

**Why this is the fix:** the derive is what writes the `impl Validate for Rating`, including the code for each `#[validate(...)]` attribute.

### `E0308` — a custom rule that does not return a `Result`

```rust
fn no_spaces(value: &str) -> bool {
    !value.contains(' ')
}
// on the field: #[validate(custom(function = "no_spaces"))]
```

`examples/05-custom-wrong-signature-broken.rs` has the field attribute on a `Handle` struct:

```text
error[E0308]: mismatched types
  --> phase3-backend-foundations\03-serialization-and-validation\02-validation\examples\05-custom-wrong-signature-broken.rs:11:10
   |
11 | #[derive(Validate)]
   |          ^^^^^^^^ expected `bool`, found `Result<_, _>`
   |
   = note: expected type `bool`
              found enum `Result<_, _>`
   = note: this error originates in the derive macro `Validate` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-03-02-validation` (example "05-custom-wrong-signature-broken") due to 1 previous error
```

**What the compiler is objecting to:** the span points at `#[derive(Validate)]`, not at your function, because the derive is what wrote the code that calls `no_spaces`, and that code needs a `Result<(), ValidationError>` back. Do not over-read "expected `bool`": the point is only that the *signature* of the function named in the attribute is wrong, and the error is reported at the derive.

**The fix:** return `Result<(), ValidationError>`, as in "The concept": `Ok(())` for a pass, `Err(ValidationError::new("code"))` for a failure.

**Why this is the fix:** a rule is not a predicate; it must be able to say *why* it failed. That `code` (and optional `message`) is what ends up in the error tree and, later, in the JSON body.

### No error at all: a nested struct whose rules never run

```text
outer validate: Ok(())
inner has errors: true
```

**What's actually broken:** `examples/06-forgot-nested-trap.rs` compiles and runs. `Review` has a field `reviewer: Reviewer`, and `Reviewer` has a rule, but the field is not marked `#[validate(nested)]`. The derive on `Review` only runs the rules it can see on `Review`'s own fields; it does not reach into other types unless you say so. The bad email is accepted, silently.

**The fix:**

```rust
#[derive(Validate)]
struct Review {
    #[validate(length(min = 1))]
    title: String,
    #[validate(nested)]
    reviewer: Reviewer,
}
```

**Why this is the fix:** `nested` tells the derive to call `reviewer.validate()` and store a failure under the key `reviewer` (a `Struct` node). The compiler cannot warn you, because "no rules on this field" is a legal state; only a test that sends a bad nested value and expects a `422` catches it. That is why this lesson's `tests/api_test.rs` has one.

---

## Exercises

### Warm up

<details>
<summary>A <code>POST</code> body is valid JSON with <code>"rating": 15</code>. Which pass rejects it, and what answers?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

The second pass. `15` is a fine `u8`, so `Json<T>` accepts the shape; `validate()` then fails the `range` rule, and your `ApiError` answers `422` with the field-keyed `errors` object.

</details>

<details>
<summary>The body has <code>"rating": "nine"</code>. Does your <code>validate()</code> run?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. `"nine"` is not a `u8`, so `Json<T>` rejects the body in the first pass, with `axum`'s own plain-text `422`. Your rules only ever see values that already have the right types.

</details>

<details>
<summary>A review's <code>notes</code> list has two items and only the second has an empty <code>text</code>. Under which key does the message appear?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`notes[1].text`. Indexes start at `0`, the list adds `[1]`, and the field inside the item is joined with a dot. The first item passes and is absent from the tree.

</details>

<details>
<summary>A <code>note: Option&lt;String&gt;</code> field has <code>length(max = 5)</code>. Is <code>None</code> an error?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. A rule on an `Option` runs only when there is a `Some`. If a value must be present, that is a shape question: make the field a plain `String`, and a missing key fails in the first pass.

</details>

<details>
<summary>Why does <code>errors.field_errors()</code> not give you a nested struct's problems?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

It returns only the `Field` entries of the top level. A nested struct is a `Struct` node and a list of structs is a `List` node, and both are filtered out. You walk `errors.errors()` instead.

</details>

### Repair

Fix all three examples:

1. `examples/04-missing-derive-broken.rs` compiles with `--features broken`. Give `Rating` a rule so that the `15` it holds is rejected.
2. `examples/05-custom-wrong-signature-broken.rs` compiles with `--features broken`, and `"ma tin"` fails with the code `no_spaces`.
3. `examples/06-forgot-nested-trap.rs` prints `outer validate: Err(...)`. Print the error and read which key the failure sits under.

### Implement

Everything in `src/lib.rs` that is a `todo!()`: `validate_handle`, `flatten_errors`, `ApiError::into_response`, `create_review`, `list_reviews`, and `app`. The rules on the types (`Reviewer`, `EpisodeNote`, `NewReview`) are already written, and `ReviewStore` is given. Each doc comment is the complete specification (the key format, the message order, the status, the exact body), so you should never need to open the tests. Work in this order: `validate_handle`, then `flatten_errors`, then the HTTP edge.

```sh
cargo test -p p3-03-02-validation --test validate_test
cargo test -p p3-03-02-validation --test api_test
```

`tests/validate_test.rs` (10 tests) checks the custom rule and the flattening with plain calls: no `axum`, no runtime, no requests. `tests/api_test.rs` (6 tests) checks the whole stack through `oneshot`, the way 3.2.1 did.

### Build

Write `ValidatedJson<T>`, an extractor that reads the body exactly like `Json<T>` does and then runs `T`'s rules, so a handler that asks for `ValidatedJson<NewReview>` can never see unchecked data. The signature is in `src/lib.rs` (the body is a `todo!()` that says what, in the doc comment above it). It must work for *any* `T`, not just `NewReview`, and it must not change the rejections `Json<T>` already makes: a broken body is still `400`, a missing `Content-Type` still `415`. When `tests/build_test.rs` is green, change `create_review` to take `ValidatedJson<NewReview>` and see that `tests/api_test.rs` still passes with the explicit `validate()?` line gone.

```sh
cargo test -p p3-03-02-validation --test build_test
```

### Challenge (optional)

Rules that look at two fields at once. Add this rule to `NewReview`: a rating of `1` or `10` must come with a `body` of at least 20 characters ("extreme ratings need a reason"). A single field attribute cannot see the other field, so use `validator`'s struct-level rule, `#[validate(schema(function = "..."))]` on the struct, with a function that takes `&NewReview` and returns `Result<(), ValidationError>`. Run it once and print what `flatten_errors` gives for a failure: in 0.18.1 a struct-level error is filed under the key `__all__`, which is not a field name, so decide what the client should see instead. There are no provided tests: write your own.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Validation rule | a constraint a well-typed value must still satisfy (`1..=10`, a length, an email) | every request body and every config |
| `#[derive(Validate)]` | generates `validate(&self) -> Result<(), ValidationErrors>` from `#[validate(...)]` attributes | request types |
| `ValidationErrors` tree | a map of field name to `Field`, `Struct` or `List` nodes | turning failures into a response |
| Custom rule | a function `fn(&T) -> Result<(), ValidationError>` named in `custom(function = "...")` | anything a built-in rule cannot say |
| `#[validate(nested)]` | opt-in: also run the rules of a struct, or of every item of a `Vec` | nested bodies |
| Field-keyed error body | `{"errors": {"path": ["message"]}}` with dots for nesting and `[i]` for list items | every `422` the API sends |
| `ValidatedJson<T>` | an extractor that parses and validates, so handlers get checked data only | any handler that takes a body |

### What you now know

- Parsing and validating are separate passes: `Json<T>` decides the shape (`400`/`415`/`422`, plain text, before your code), `validate()` decides the rules (`422`, in a body you design).
- `#[derive(Validate)]` runs every rule and reports all failures at once; a rule on an `Option` only runs on `Some`.
- The result is a tree with three node kinds, and `field_errors()` only shows the leaves, so nested data needs your own walk.
- A custom rule is a function returning `Result<(), ValidationError>`, and `nested` must be written for a struct's rules to run through a parent.
- A field-keyed `422` body is built once, in one `IntoResponse` impl, and reached with `?` from any handler.

### What comes back later

- **Putting these same constraints into a generated API document, so clients and docs cannot drift**: [3.3.3 — API contracts and OpenAPI (`utoipa`)](../03-api-contracts-and-openapi/README.md)
- **One error format for every failure, including the plain-text `axum` rejections you saw today**: [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **Storing the validated review somewhere that survives a restart**: [3.5.3 — Anime catalog, Postgres-backed](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
- **Changing a rule without breaking every client that already sends the old shape**: [3.3.4 — API versioning and evolution](../04-api-versioning-and-evolution/README.md)
- **The extractor traits `ValidatedJson<T>` implements, one level up**: [3.2.2 — Writing your own extractor](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md)

### Can you explain?

- Why is `{"rating": 15}` rejected by a different part of the program than `{"rating": "nine"}`, and what does each answer?
- What does `#[derive(Validate)]` generate, and why does it report every failed rule instead of stopping at the first?
- Why is `errors.field_errors()` not enough for a review with a `reviewer` inside it?
- Why can a field's rule on an `Option` pass for `None`, and where do you say "this field is required" instead?
- What goes wrong when you forget `#[validate(nested)]`, and why can the compiler not warn you?
- What does `ValidatedJson<T>` buy a handler that `input.validate()?` does not?

---

## Going further

- [`validator` 0.18.1 on docs.rs](https://docs.rs/validator/0.18.1/validator/): the full rule list (`contains`, `must_match`, `regex`, ...) and the `ValidationErrors` API.
- [`Json` in the `axum` 0.8.9 docs](https://docs.rs/axum/0.8.9/axum/struct.Json.html): the rejection table that your validation sits behind.
- [RFC 9110 §15.5.21 — 422 Unprocessable Content](https://www.rfc-editor.org/rfc/rfc9110.html#name-422-unprocessable-content): the status, in the standard's words.
- [DRF — Serializer validation](https://www.django-rest-framework.org/api-guide/serializers/#validation): `validate_<field>()` and `validate()`, the Python versions of a custom rule and a struct-level rule.
