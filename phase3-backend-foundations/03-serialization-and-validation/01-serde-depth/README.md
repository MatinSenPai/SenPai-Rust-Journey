# 3.3.1 — Serde in depth

## At a glance

After this lesson you can:

- Shape the JSON of a struct without touching its fields: rename keys, leave a key out, fill in a missing one, merge two structs into one flat object, and reject keys you did not expect.
- Pick between the four JSON representations of an enum, and spot the one that quietly picks the wrong variant.
- Write `Serialize` by hand, and `Deserialize` with a `Visitor`, for a type whose JSON does not look like its Rust.
- Read the `E0277` for a missing `Deserialize`, the `E0277` for a `#[serde(default)]` on a type with no `Default`, and the run-time error for an internal tag next to a bare integer.

**Time:** ~110 minutes · **Prerequisites:**
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[2.3.1 — Defining and implementing traits](../../../phase2-intermediate/03-traits-and-generics/01-defining-and-implementing-traits/README.md),
[2.3.4 — The standard derives, implemented by hand](../../../phase2-intermediate/03-traits-and-generics/04-standard-derives-by-hand/README.md)

---

## Why this matters

In 3.2.3 you wrote `Json<CreateAnime>` and `Json<Anime>` and it just worked. Behind that were two `derive`s, `Serialize` and `Deserialize`, and one default: the JSON looks exactly like the struct. That default is right until the first real client. The mobile app wants `watchStatus`, not `watch_status`. The list screen does not want `"rating":null` on every row. The create endpoint should reject a typo instead of dropping it. An event feed needs one array holding three different shapes. Each of those is a one-line attribute, and not knowing the attribute is how people end up writing a second struct just to rename a field.

If you come from Django, this is the job of a DRF `Serializer`: declare the fields, say which are optional, rename with `source=`, compute a value with `SerializerMethodField`. The difference is where it lives. A DRF serializer is a separate class that you write next to the model. Here the shape is *attributes on the type itself*, checked at compile time, and the same type serves both directions: writing JSON and reading it. Where the analogy stops is the other half of a serializer's job. A DRF serializer also validates (`validate_rating`, `max_value=10`). `serde` only describes the *shape*; "a rating is 1 to 10" is [3.3.2 — Validation](../02-validation/README.md). One section here (the `Rating` hand-written `Deserialize`) touches the border on purpose, so you can see exactly where shape ends and rules begin.

There is no HTTP in this lesson at all, just `serde` and `serde_json`, so every example is a function from a value to text and back. The same attributes work unchanged inside `Json<T>`. [3.3.3 — API contracts and OpenAPI](../03-api-contracts-and-openapi/README.md) and [3.3.4 — API versioning and evolution](../04-api-versioning-and-evolution/README.md) follow in this module.

---

## The concept

### What `derive(Serialize)` really gives you

`serde` splits the job in two on purpose. Your type implements a trait that says *what its data is* (a struct with these named fields, an enum variant, a string). A *format* crate, here `serde_json`, decides how that looks as text. Neither knows the other.

```senpai-visual
{"kind":"concept","labels":["your type","Serialize: says what the data is","Serializer: serde_json says how it looks","JSON text","Deserialize + Visitor: reads it back"]}
```

`#[derive(Serialize, Deserialize)]` writes both impls for you, field by field. Everything below is either an attribute that changes what the derive writes, or a hand-written impl that replaces it. The two directions are separate, which is the root of most surprises: `skip_serializing_if` is about writing only, `default` about reading only.

### Renaming, leaving out, filling in

Take the card shape a list screen wants. Keys in camelCase, `rating` left out when there is none, and `episodeCount` optional on the way in:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Card {
    id: u64,
    title: String,
    watch_status: WatchStatus,
    #[serde(default)]
    episode_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    rating: Option<u8>,
}
```

`examples/01-rename-skip-default.rs` writes it twice (with and without a rating) and then reads two inputs:

```text
{"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28,"rating":9}
{"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28}
Ok(Card { id: 2, title: "Mushishi", watch_status: Completed, episode_count: 0, rating: None })
Err(Error("missing field `watchStatus`", line: 1, column: 54))
```

Reading it line by line: keys come out in the order the fields are declared. With `rating: None` the key is gone, not `null`. A missing `episodeCount` became `0`, because `default` calls `Default::default()` for a `u32`. A missing `rating` became `None` *without* any attribute, since a derived `Deserialize` treats a missing `Option` field as `None`. And a snake_case `watch_status` key was not understood: `rename_all` renames in *both* directions, so the Rust name is no longer a valid key. The error says `missing field` and not "unknown key" because, by default, an unknown key is simply ignored; the key `watch_status` was dropped and then `watchStatus` was found missing.

*Show, then name:* `rename_all` (a container attribute, applies to every field), `default` and `skip_serializing_if` (field attributes). This is the DRF `required=False` plus `default=` plus "do not include this key" in one place each. `rename_all` also takes `"snake_case"`, `"SCREAMING_SNAKE_CASE"`, `"kebab-case"` and others; a single field can override with `#[serde(rename = "...")]`.

### Merging structs: `flatten`

A detail screen wants the card's fields and the audit timestamps in **one** flat object, not `{"card":{...},"audit":{...}}`. Keep two structs in Rust (they are reused elsewhere) and ask for one object on the wire:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Detail {
    #[serde(flatten)]
    card: Card,
    #[serde(flatten)]
    audit: Audit,
}

#[derive(Debug, Serialize, Deserialize)]
struct WithExtras {
    id: u64,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}
```

`examples/02-flatten.rs` writes a `Detail`, reads it back, and then reads an object with keys `WithExtras` does not know:

```text
{"id":1,"title":"Frieren","created_at":"2026-01-05"}
Detail { card: Card { id: 1, title: "Frieren" }, audit: Audit { created_at: "2026-01-05" } }
WithExtras { id: 7, extra: {"studio": String("Madhouse"), "year": Number(2023)} }
```

`flatten` on a struct field splices that struct's keys into the parent, in both directions. `flatten` on a map is a catch-all: every key no named field claimed lands in the map. That is how you keep unknown data instead of dropping it, for example a proxy that must pass through fields it does not understand. (It has a cost: to sort keys out `serde` has to buffer the whole object while reading. It is a tool for shapes, not for hot loops.)

### Rejecting keys you did not expect: `deny_unknown_fields`

By default an unknown key is ignored. For a *response* you are reading from someone else that is exactly what you want: they may add fields tomorrow. For a *request* your own API receives, it hides bugs, because `{"title":"Frieren","ratng":9}` creates a show with no rating and reports success. A typo is dropped in silence.

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Strict {
    title: String,
    rating: Option<u8>,
}
```

`examples/06-deny-unknown-fields.rs` feeds the typo to a `Lenient` struct (same fields, no attribute), to `Strict`, and to a third struct that combines `deny_unknown_fields` with a `flatten` catch-all map:

```text
Ok(Lenient { title: "Frieren", rating: None })
Err(Error("unknown field `ratng`, expected `title` or `rating`", line: 1, column: 26))
Err(Error("unknown field `ratng`", line: 1, column: 29))
```

`Lenient` accepts it and `rating` is `None`, no complaint. `Strict` refuses and names the unknown key *and* lists what it expected; this is the message you want to hand back to an API client. The third line is the trap: `serde`'s documentation says `deny_unknown_fields` is not supported together with `flatten`. It does reject here, but with the shorter message that has no list of expected keys, and a catch-all map next to a "deny" attribute says two opposite things about the same key. Do not combine them; pick "keep unknown keys" or "refuse unknown keys".

### Enums: four ways to write the same data

An enum is where JSON and Rust disagree most, because JSON has no "one of these shapes". `serde` gives you four representations. Same enum, four attributes (`examples/03-tagged-enums.rs`):

```rust
#[derive(Serialize, Deserialize)]
enum Event {
    Added { id: u64, title: String },
    Removed { id: u64 },
}
// plus #[serde(tag = "type")], or
// #[serde(tag = "type", content = "data")], or
// #[serde(untagged)] on copies of the same enum
```

The example prints `Event::Added { id: 1, title: "Frieren" }` in each representation, and then reads three inputs into the internally tagged one and one into the untagged one:

```text
{"Added":{"id":1,"title":"Frieren"}}
{"type":"Added","id":1,"title":"Frieren"}
{"type":"Added","data":{"id":1,"title":"Frieren"}}
{"id":1,"title":"Frieren"}
Ok(Removed { id: 1 })
Err(Error("unknown variant `Renamed`, expected `Added` or `Removed`", line: 1, column: 17))
Err(Error("missing field `type`", line: 1, column: 8))
Err(Error("data did not match any variant of untagged enum Untagged", line: 0, column: 0))
```

| Attribute | JSON for `Added` | When |
|---|---|---|
| none (externally tagged) | `{"Added":{"id":1,"title":"Frieren"}}` | the default; the variant name is the only key |
| `tag = "type"` (internally tagged) | `{"type":"Added","id":1,"title":"Frieren"}` | what most JSON APIs and event feeds use |
| `tag = "type", content = "data"` (adjacently tagged) | `{"type":"Added","data":{"id":1,"title":"Frieren"}}` | when the payload must stay in its own key |
| `untagged` | `{"id":1,"title":"Frieren"}` | when the data itself tells the variants apart |

For an event feed the internally tagged form is the one to reach for: a client can switch on `type` without looking inside. It also gives good errors, as the transcript shows: an unknown `type` names the variant and lists the valid ones, and a missing `type` says ``missing field `type` ``. The last line of the transcript is the contrast. `untagged` has no tag to complain about, so the best it can say is `data did not match any variant`, with no hint which key was wrong and no position. That poor error is the price of untagged; the next paragraph is the other.

`untagged` tries the variants **in order, top to bottom, and the first that fits wins**. If two variants can both accept the same JSON, the earlier one takes it and the later one is dead. Nothing warns you. "Errors you will meet" runs exactly that case.

### Writing `Serialize` by hand

Sometimes the JSON is not a field-by-field copy of the struct. Here a response has a computed `label` that is stored nowhere, and a `rating` that is omitted when `None`. You could add a second struct; or implement the trait:

```rust
impl Serialize for Anime {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let len = if self.rating.is_some() { 4 } else { 3 };
        let mut s = serializer.serialize_struct("Anime", len)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("title", &self.title)?;
        if let Some(r) = self.rating {
            s.serialize_field("rating", &r)?;
        }
        s.serialize_field("label", &format!("#{} {}", self.id, self.title))?;
        s.end()
    }
}
```

`examples/04-hand-serialize-struct.rs` writes one anime with a rating and one without:

```text
{"id":1,"title":"Frieren","rating":9,"label":"#1 Frieren"}
{"id":2,"title":"Mushishi","label":"#2 Mushishi"}
```

The shape of every manual struct impl: ask the serializer for a "struct writer" (`serialize_struct`, with the type's name and the number of fields you are *about to write*; `serde_json` ignores the number, but other formats rely on it, so keep it honest), push fields one at a time, and finish with `.end()`. The `?`s propagate the format's own error type. `S` is whatever format is writing (`serde_json` here); your code never mentions JSON. For a type that is a single value, like the `Rating` you implement below, there is no struct writer: you call one method, such as `serialize_str`, and you are done.

### Reading by hand: `Deserialize` and a `Visitor`

Reading is harder than writing, for a reason worth seeing. When *writing*, you know your value. When *reading*, you do not know what the JSON holds: a string, a number, an object. So `serde` flips the control. You tell the deserializer "I can accept these shapes", by implementing a `Visitor` with one method per shape, and the deserializer calls the method that matches what it finds.

`examples/05-visitor-deserialize.rs` reads a `Minutes` from either `24` or `"24m"`. The visitor is:

```rust
impl<'de> Visitor<'de> for MinutesVisitor {
    type Value = Minutes;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a number of minutes like 24, or a string like \"24m\"")
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Minutes, E> {
        u32::try_from(v).map(Minutes).map_err(E::custom)
    }
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Minutes, E> {
        let digits = v.strip_suffix('m').ok_or_else(|| E::custom("missing `m`"))?;
        digits.parse().map(Minutes).map_err(E::custom)
    }
}
```

The `Deserialize` impl itself is one line, `d.deserialize_any(MinutesVisitor)`: "look at the input and call whichever `visit_` fits". Five inputs:

```text
    24 -> Ok(Minutes(24))
 "24m" -> Ok(Minutes(24))
  "24" -> Err(Error("missing `m`", line: 1, column: 4))
  true -> Err(Error("invalid type: boolean `true`, expected a number of minutes like 24, or a string like \"24m\"", line: 1, column: 4))
    -3 -> Err(Error("invalid type: integer `-3`, expected a number of minutes like 24, or a string like \"24m\"", line: 1, column: 2))
```

Two numbers and a string with the suffix work. `"24"` reaches `visit_str` and fails with *your* message. `true` and `-3` reach no method you wrote, so the default method of the trait fails for you, and the message is built from `expecting`: `invalid type: boolean `true`, expected a number of minutes like 24, or a string like "24m"`. That sentence is why `expecting` matters: write it as the end of "expected ...". `E::custom` builds an error of whatever type the format uses, so your visitor does not depend on `serde_json`.

Where DRF fits: this is `to_internal_value` and `to_representation` for one field, plus a bit of `validate_<field>`. The difference is that the visitor is typed. There is no `data` dict of `Any` to poke at; each shape arrives as a Rust value in the method made for it.

---

## Hands on

There is no server in this lesson: every command is a program that prints a few lines. Run the six that compile and compare with the transcripts above:

```sh
cargo run -p p3-03-01-serde-depth --example 01-rename-skip-default
cargo run -p p3-03-01-serde-depth --example 02-flatten
cargo run -p p3-03-01-serde-depth --example 03-tagged-enums
cargo run -p p3-03-01-serde-depth --example 04-hand-serialize-struct
cargo run -p p3-03-01-serde-depth --example 05-visitor-deserialize
cargo run -p p3-03-01-serde-depth --example 06-deny-unknown-fields
```

(While `src/lib.rs` is unfinished, `cargo` prints a dozen `unused variable` warnings from your `todo!()` skeleton before each run. They go away as you implement the functions. The transcripts in this lesson leave them out and show only the program's output or the error that matters.) Examples `07`, `08` and `10` are broken on purpose and sit behind the `broken` feature; `09` runs and is simply wrong. "Errors you will meet" covers all four, and `11` and `12` are their fixes.

The tests start out red. `src/lib.rs` has the whole skeleton and every function is a `todo!()` with a doc comment that says exactly what it must do and the exact JSON it works with:

```sh
cargo test -p p3-03-01-serde-depth --lib 2>&1 | grep 'test result'
```

```text
test result: FAILED. 1 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The one test that passes is the already-complete `Anime` from 3.2.3.) The ladder has three more test files: `tests/build_test.rs` for the Build rung and `tests/challenge_test.rs` for the Challenge. These are the totals from running them against the finished code in `solution/`:

```sh
cd phase3-backend-foundations/03-serialization-and-validation/01-serde-depth/solution
cargo test 2>&1 | grep -E 'Running|test result'
```

```text
     Running unittests src\lib.rs (target\debug\deps\p3_03_01_serde_depth_solution-bc5380be2b442f1b.exe)
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\build_test.rs (target\debug\deps\build_test-b9c8d9ef9621fcc1.exe)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests\challenge_test.rs (target\debug\deps\challenge_test-176306e8bdab8d94.exe)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Then try these:

1. In `examples/01-rename-skip-default.rs` add `#[serde(rename = "eps")]` to `episode_count`. What does the written JSON look like, and does the input `{"episodeCount":3,...}` still read?
2. In `examples/02-flatten.rs`, give `Card` and `Audit` a field with the same name (`id`, say). What does the written JSON contain? Does it still read back?
3. In `examples/03-tagged-enums.rs`, send `{"type":"added","id":1,"title":"x"}` (lower-case) to the `Internal` enum. Which message do you get, and which attribute would make it succeed?

---

## Errors you will meet

(The `todo!()` warnings from `src/lib.rs` are cut from the transcripts below, so each starts at the real error.)

### `E0277` — the trait bound `Rating: Deserialize<'de>` is not satisfied

`Review` derives `Deserialize`, but its field type `Rating` derives only `Serialize`. `examples/07-missing-deserialize-bound-broken.rs`:

```text
error[E0277]: the trait bound `Rating: serde::Deserialize<'de>` is not satisfied
    --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\07-missing-deserialize-bound-broken.rs:13:13
     |
  13 |     rating: Rating,
     |             ^^^^^^ unsatisfied trait bound
     |
help: the trait `Deserialize<'_>` is not implemented for `Rating`
    --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\07-missing-deserialize-bound-broken.rs:8:1
     |
   8 | struct Rating(u8);
     | ^^^^^^^^^^^^^
     = note: for local types consider adding `#[derive(serde::Deserialize)]` to your `Rating` type
     = note: for types from other crates check whether the crate offers a `serde` feature flag
     = help: the following other types implement trait `Deserialize<'de>`:
               &'a Path
               &'a [u8]
               &'a str
               ()
               (T,)
               (T0, T1)
               (T0, T1, T2)
               (T0, T1, T2, T3)
             and 143 others
note: required by a bound in `next_element`
    --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\serde_core-1.0.228\src\de\mod.rs:1771:12
     |
1769 |     fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
     |        ------------ required by a bound in this associated function
1770 |     where
1771 |         T: Deserialize<'de>,
     |            ^^^^^^^^^^^^^^^^ required by this bound in `SeqAccess::next_element`

error[E0277]: the trait bound `Rating: serde::Deserialize<'de>` is not satisfied
    --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\07-missing-deserialize-bound-broken.rs:13:13
     |
  13 |     rating: Rating,
     |             ^^^^^^ unsatisfied trait bound
     |
help: the trait `Deserialize<'_>` is not implemented for `Rating`
    --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\07-missing-deserialize-bound-broken.rs:8:1
     |
   8 | struct Rating(u8);
     | ^^^^^^^^^^^^^
     = note: for local types consider adding `#[derive(serde::Deserialize)]` to your `Rating` type
     = note: for types from other crates check whether the crate offers a `serde` feature flag
     = help: the following other types implement trait `Deserialize<'de>`:
               &'a Path
               &'a [u8]
               &'a str
               ()
               (T,)
               (T0, T1)
               (T0, T1, T2)
               (T0, T1, T2, T3)
             and 143 others
note: required by a bound in `next_value`
    --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\serde_core-1.0.228\src\de\mod.rs:1916:12
     |
1914 |     fn next_value<V>(&mut self) -> Result<V, Self::Error>
     |        ---------- required by a bound in this associated function
1915 |     where
1916 |         V: Deserialize<'de>,
     |            ^^^^^^^^^^^^^^^^ required by this bound in `MapAccess::next_value`

error[E0277]: the trait bound `Rating: serde::Deserialize<'de>` is not satisfied
  --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\07-missing-deserialize-bound-broken.rs:10:28
   |
10 | #[derive(Debug, Serialize, Deserialize)]
   |                            ^^^^^^^^^^^ unsatisfied trait bound
   |
help: the trait `Deserialize<'_>` is not implemented for `Rating`
  --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\07-missing-deserialize-bound-broken.rs:8:1
   |
 8 | struct Rating(u8);
   | ^^^^^^^^^^^^^
   = note: for local types consider adding `#[derive(serde::Deserialize)]` to your `Rating` type
   = note: for types from other crates check whether the crate offers a `serde` feature flag
   = help: the following other types implement trait `Deserialize<'de>`:
             &'a Path
             &'a [u8]
             &'a str
             ()
             (T,)
             (T0, T1)
             (T0, T1, T2)
             (T0, T1, T2, T3)
           and 143 others
note: required by a bound in `_::_serde::__private228::de::missing_field`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\serde-1.0.228\src\private\de.rs:26:8
   |
24 | pub fn missing_field<'de, V, E>(field: &'static str) -> Result<V, E>
   |        ------------- required by a bound in this function
25 | where
26 |     V: Deserialize<'de>,
   |        ^^^^^^^^^^^^^^^^ required by this bound in `missing_field`
   = note: this error originates in the derive macro `Deserialize` (in Nightly builds, run with -Z macro-backtrace for more info)

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-03-01-serde-depth` (example "07-missing-deserialize-bound-broken") due to 3 previous errors
```

**What the compiler is objecting to:** the derive for `Review` writes a `deserialize` that has to read a `Rating` for the `rating` key, and that needs `Rating: Deserialize`. `Rating` has no such impl. The message is long because the derive expands to three places that need it (reading a field in sequence form, reading it in map form, and the "missing field" fallback), and the compiler reports each. The first line, the missing impl for `Rating`, is the whole story; the other two are the same fact from the other call sites. `note: for local types consider adding #[derive(serde::Deserialize)]` is the fix, written for you.

**The fix:** derive it, or write it by hand when the JSON is not a plain copy of the struct, as you do for `Rating` in the Build rung:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Rating(u8);
```

**Why this is the fix:** a derived `Deserialize` is only as capable as its fields. Every type nested inside needs both directions you use it in. It is the same rule as `#[derive(Default)]` needing `Default` on every field (2.3.4).

### `E0277` — `#[serde(default)]` on a type with no `Default`

`examples/08-default-without-default-broken.rs` puts `#[serde(default)]` on a field whose type is an enum with no `Default`:

```text
error[E0277]: the trait bound `Status: Default` is not satisfied
  --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\08-default-without-default-broken.rs:16:5
   |
16 |     #[serde(default)]
   |     ^ the trait `Default` is not implemented for `Status`
   |
help: consider annotating `Status` with `#[derive(Default)]`
   |
 8 + #[derive(Default)]
 9 | enum Status {
   |

error[E0277]: the trait bound `Status: Default` is not satisfied
  --> phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\08-default-without-default-broken.rs:13:17
   |
13 | #[derive(Debug, Deserialize)]
   |                 ^^^^^^^^^^^ the trait `Default` is not implemented for `Status`
   |
   = note: this error originates in the derive macro `Deserialize` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider annotating `Status` with `#[derive(Default)]`
   |
 8 + #[derive(Default)]
 9 | enum Status {
   |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-03-01-serde-depth` (example "08-default-without-default-broken") due to 2 previous errors
```

**What the compiler is objecting to:** `#[serde(default)]` means "when the key is missing, use `Default::default()`". The generated code calls that for `Status`, and `Status` does not implement the trait. The first error points at the attribute, the second at the derive that expanded it.

**The fix:** say which variant is the default. The standard library has an attribute for it:

```rust
#[derive(Debug, Deserialize, Default)]
enum Status {
    Watching,
    #[default]
    Dropped,
}
```

**Why this is the fix:** `#[derive(Default)]` on an enum needs one variant marked `#[default]` (the `WatchStatus` in `src/lib.rs` does exactly this for `PlanToWatch`). When the default should *not* be the type's own `Default`, `#[serde(default = "path::to::function")]` calls a function of yours instead, so a missing key can mean something the type's own `Default` would not.

### A run-time `Err`: an internal tag next to a bare integer

Nothing here stops the compiler. `examples/10-tagged-newtype-panic-broken.rs` has an internally tagged enum with a variant `ById(u64)`:

```text
{"type":"ByTitle","title":"Frieren"}

thread 'main' (42116) panicked at phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\10-tagged-newtype-panic-broken.rs:20:55:
called `Result::unwrap()` on an `Err` value: Error("cannot serialize tagged newtype variant Lookup::ById containing an integer", line: 0, column: 0)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The thread id in parentheses changes on every run.) The first line printed fine: the struct variant `ByTitle` has keys, so the `type` key can sit beside them. Then `Lookup::ById(1)` failed. The panic here is only our `.unwrap()`; the real failure is the `Err` it unwrapped.

**What is actually broken:** an internal tag is an extra key *inside the variant's object*. A struct variant is an object, so it works. `ById(1)` would have to be `{"type":"ById", ???}`: a bare number has no keys to put the tag beside, and `serde` can only find that out while it is writing. The derive compiled; the `Err` arrives on the first value that hits the bad variant. Worse, `ByTitle` worked, so a quick test of the happy path would have passed.

**The fix:** make every variant of an internally tagged enum a struct variant (or a tuple variant holding a struct). `examples/12-tagged-struct-variant-fix.rs`:

```rust
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Lookup {
    ById { id: u64 },
    ByTitle { title: String },
}
```

```text
{"type":"by_id","id":1}
{"type":"by_title","title":"Frieren"}
```

**Why this is the fix:** now both variants are objects with named keys and the tag has somewhere to go. If you truly need a bare payload, use adjacent tagging (`tag` plus `content`, from the table above) and the payload gets its own key.

### No error at all: `untagged` picks the wrong variant

`examples/09-untagged-wrong-variant-trap.rs` has a draft form (rating optional) and a full form (rating and status required) under `#[serde(untagged)]`, and reads a body that is clearly the full one:

```rust
#[serde(untagged)]
enum Input {
    Draft { title: String, rating: Option<u8> },
    Full { title: String, rating: u8, status: String },
}
```

```text
Draft { title: "Frieren", rating: Some(9) }
```

**What's actually broken:** it compiles, runs, and returns `Ok`. But the body had a `status`, and the result is a `Draft`, so the `status` was thrown away. `Draft` is tried first; it needs only `title` and an optional `rating`, the body has both, and an unknown key (`status`) is ignored by default. First fit wins, `Full` is never reached, and no test that only checks "did it parse" notices.

**The fix:** put the most specific variant first, `examples/11-untagged-specific-first-fix.rs`:

```text
Full { title: "Frieren", rating: 9, status: "watching" }
Draft { title: "Frieren", rating: None }
```

**Why this is the fix:** `untagged` is "try each in order", so order is the rule. The better fix is usually to stop being `untagged`: a `tag` makes the choice explicit, and so would moving `Draft`'s fields into their own struct marked `deny_unknown_fields`, which makes the first attempt fail for a body with a `status` key. The lesson to keep is that `untagged` hides decisions; use it only when the shapes truly cannot overlap.

---

## Exercises

### Warm up

<details>
<summary>A struct has <code>#[serde(rename_all = "camelCase")]</code> and a field <code>watch_status</code>. A client sends the key <code>watch_status</code>. What happens?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

It is not accepted. `rename_all` renames in both directions, so the only key that reads into the field is `watchStatus`. The unknown `watch_status` key is ignored (no `deny_unknown_fields`), and the result is ``missing field `watchStatus` ``. Example 01 shows this exact error.

</details>

<details>
<summary>Does <code>#[serde(skip_serializing_if = "Option::is_none")]</code> change how a missing <code>rating</code> key is <em>read</em>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No, it only affects writing. A missing `Option` field already reads as `None` in a derived `Deserialize`, with or without any attribute. This is why a field often carries both `default` and `skip_serializing_if`: one per direction.

</details>

<details>
<summary>In <code>#[serde(untagged)] enum E { A { x: Option&lt;u8&gt; }, B { x: u8, y: u8 } }</code>, which variant reads <code>{"x":1,"y":2}</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`A`. It is tried first, `x` fits, and the unknown `y` is ignored. `B` is never reached for any input that `A` accepts. Swapping the order, or tagging the enum, fixes it.

</details>

<details>
<summary>Will an internally tagged enum with a variant <code>ById(u64)</code> compile? When does the problem show up?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

It compiles. The problem shows up at run time, as an `Err` when a `ById` value is *written*: there is no key in a bare number to put the `type` beside. Struct variants never have the problem, which is why a test that only writes the other variants misses it.

</details>

<details>
<summary>Why does <code>CreateAnime</code> get <code>deny_unknown_fields</code> but a struct you read from a third-party API usually should not?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

For your own request body, an unknown key is almost always a client bug (a typo), and refusing it tells them. For someone else's response, an unknown key is most likely a field they added after you wrote the code, and refusing it turns their harmless change into your outage.

</details>

### Repair

Fix all four, and check each with the command in its file header:

1. `examples/07-missing-deserialize-bound-broken.rs` compiles (build it with `--features broken`).
2. `examples/08-default-without-default-broken.rs` compiles, and reading `{"title":"Frieren"}` gives `Dropped`.
3. `examples/10-tagged-newtype-panic-broken.rs` runs without panicking and prints two JSON lines. Do not just delete the `ById` variant.
4. `examples/09-untagged-wrong-variant-trap.rs` prints a `Full`. Find two different fixes.

### Implement

Everything in `src/lib.rs` marked `Implement 1` to `Implement 5` (the Build and Challenge `todo!()`s come in their own rungs), with the `#[serde(...)]` attributes that go on the types. Each doc comment spells out the exact JSON (key order, which keys are left out, which are optional), so you should never need the tests to know what to build. Five pieces, one per idea from "The concept":

1. `AnimeCard`: the camelCase card (`rename_all`, `default`, `skip_serializing_if`), plus `card_to_json` and `card_from_json`.
2. `AnimeDetail`: the card and `Audit` as one flat object (`flatten`), plus `detail_to_json` and `detail_from_json`.
3. `CreateAnime`: a request body that rejects typos and has a default status (`deny_unknown_fields`, `default`), plus `decode_create_anime`.
4. `AnimeEvent`: the internally tagged event feed, plus `events_to_json` and `events_from_json`.
5. `Rating`: the hand-written `Serialize` that writes `"9/10"`.

```sh
cargo test -p p3-03-01-serde-depth --lib
```

The 25 tests are in `src/lib.rs`, grouped by piece. Note what the functions return on failure: the *error text from `serde_json`*, unchanged. That is how the tests check that a typo is really named in the message.

### Build

Write `Deserialize` for `Rating` with a `Visitor`, in the `impl` already waiting in `src/lib.rs`. It reads either the string `"9/10"` or the bare integer `9`, accepts only 1 to 10, and fails with an exact message otherwise. This is the border with [3.3.2](../02-validation/README.md): reading is where "shape" and "rules" meet, and the doc comment says exactly where each error comes from. Write it so that a value that is not a string or non-negative integer reports your `expecting` text.

```sh
cargo test -p p3-03-01-serde-depth --test build_test
```

`tests/build_test.rs` has 8 tests, including a write-then-read test for every rating from 1 to 10.

### Challenge (optional)

`AnimeGenres` stores its genres as a `Vec<String>` but the wire format is one comma-separated string, via `#[serde(with = "genre_list")]`. Implement the two functions in the `genre_list` module. `with` is the third way to customize serde, between an attribute and a whole hand-written impl: you write only the field's two functions and the derive does the rest. The file is `tests/challenge_test.rs` (5 tests). It stays small on purpose; the next lesson, [3.3.2](../02-validation/README.md), is where "this list must not be empty" goes.

```sh
cargo test -p p3-03-01-serde-depth --test challenge_test
```

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `rename_all` / `rename` | change the JSON key names, in both directions | every API whose clients are not Rust |
| `default` | the value to use when a key is missing while reading | optional request fields, additive API changes |
| `skip_serializing_if` | leave a key out of the written JSON when a predicate is true | `Option` fields, empty lists |
| `flatten` | splice a nested struct's keys into the parent, or collect leftover keys into a map | shared "audit" or "pagination" blocks |
| `deny_unknown_fields` | reading fails on a key the struct does not know | request bodies, config files |
| tagged / untagged enum | how an enum's variant is written: external, internal (`tag`), adjacent (`tag` + `content`), or not at all | event feeds, polymorphic payloads |
| `Visitor` | an object that says which JSON shapes a type can be read from, with one method per shape | any type whose JSON is not a copy of its fields |
| `serde(with = "...")` | send one field through your own pair of functions | a field with its own wire format |

### What you now know

- Attributes change the shape of the JSON without changing the type, and each one works in one direction or both: `skip_serializing_if` writes, `default` reads, `rename_all` does both.
- An unknown key is silently dropped by default; `deny_unknown_fields` turns that into an error that names the key, and it does not mix with a catch-all `flatten`.
- Enums have four JSON representations; internal tagging is the usual API choice, needs every variant to be a struct, and `untagged` always means "first variant that fits".
- `Serialize` is written with a struct writer and `Deserialize` with a `Visitor`; your impls never mention `serde_json`, only the format-neutral traits.
- A derive needs the trait on every field type, and `#[serde(default)]` needs `Default` on the field.

### What comes back later

- **Rules on top of the shape (`1..=10`, non-empty, an email)**: [3.3.2 — Validation](../02-validation/README.md)
- **API contracts and OpenAPI**: [3.3.3 — API contracts and OpenAPI](../03-api-contracts-and-openapi/README.md)
- **API versioning and evolution**: [3.3.4 — API versioning and evolution](../04-api-versioning-and-evolution/README.md)
- **Consistent error envelopes**: [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **How the `Json<T>` extractor reports a failed read as `422`**: [3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md)

### Can you explain?

- Which of `rename_all`, `default` and `skip_serializing_if` affects writing, which reading, and which both?
- Why is `{"title":"Frieren","ratng":9}` a successful request without `deny_unknown_fields`, and why is that dangerous?
- What do the four enum representations look like on the wire, and which one would you give an event feed?
- Why does `untagged` sometimes return the wrong variant with no error, and how do you stop it?
- Why does reading need a `Visitor` when writing did not need anything like it?
- Where does a DRF serializer's job end and `serde`'s begin?

---

## Going further

- [Serde attributes](https://serde.rs/attributes.html): the complete list: container, variant and field attributes, including every `rename_all` style.
- [Enum representations](https://serde.rs/enum-representations.html): the four forms side by side, with the same example data as above.
- [Implementing `Deserialize`](https://serde.rs/impl-deserialize.html) and [Implementing `Serialize`](https://serde.rs/impl-serialize.html): the official walk-throughs of the two hand-written impls.
- [Custom serialization with `with`](https://serde.rs/field-attrs.html#with): the field attribute behind the Challenge.
- [`serde_json` documentation](https://docs.rs/serde_json/1.0.150/serde_json/): `from_str`, `to_string`, `Value`, and the error type you printed.
- [3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md): the `CreateAnime` and `Anime` types this lesson takes apart.
