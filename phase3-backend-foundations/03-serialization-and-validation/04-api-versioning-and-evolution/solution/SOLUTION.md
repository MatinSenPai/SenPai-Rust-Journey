# Solution — 3.3.4 API versioning and evolution

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/`, including `build_test.rs` (Build rung) and `challenge_test.rs` (Challenge rung).

## The two `From` impls

```rust
impl From<&Anime> for AnimeV1 {
    fn from(a: &Anime) -> Self {
        AnimeV1 { id: a.id, title: a.title.clone(), status: a.status, rating: a.rating }
    }
}
```

`AnimeV2::from` is the same with `watch_status: a.status` and `episodes: a.episodes`. Taking `&Anime` means one stored value can be shown in both shapes without a clone of the whole struct; only the `title` is cloned. The "unknown episodes" and "null rating" rules are not in the `From` code at all. They live on the type: `AnimeV2` has `#[serde(default, skip_serializing_if = "Option::is_none")]` on `episodes`, and `rating` has no attribute, so `None` is written as `null`. That is `v2_omits_unknown_episodes_but_keeps_a_null_rating`.

## `classify` and `requires_new_version`

```rust
pub fn classify(change: Change) -> Compat {
    use Change::*;
    match change {
        RemoveResponseField | RenameResponseField | ChangeResponseFieldType
        | AddResponseEnumValue | AddRequiredRequestField
        | TightenRequestValidation | RemoveEndpoint => Compat::Breaking,
        AddResponseField | RemoveResponseEnumValue | AddOptionalRequestField
        | RemoveRequestField | LoosenRequestValidation | AddEndpoint => Compat::Compatible,
    }
}
```

There is no `_` arm, on purpose: add a fourteenth `Change` variant and this stops compiling until you decide what it is. That is the same `E0004` the lesson shows from the client's side. `requires_new_version` is `changes.iter().any(|c| classify(*c) == Compat::Breaking)`; `any` on an empty slice is `false`, which is the required answer for no changes.

## `deprecation_headers` and `version_from_accept`

```rust
vec![
    ("deprecation", format!("@{deprecated_at_unix}")),
    ("sunset", sunset.to_string()),
    ("link", format!("<{successor}>; rel=\"successor-version\"")),
]
```

Order and lowercase names are part of the spec, so the test can compare the whole `Vec`. For `Accept`, each comma-separated entry is cut at its first `;` (that drops `q=0.5`), trimmed, and compared with `eq_ignore_ascii_case`. The loop returns on the first entry that is a known vendor type, so "whichever comes first wins" falls out of the loop order. After the loop, `V1` is the answer for everything else, including `*/*`.

## Build: `app_after_sunset`

```rust
let v1 = Router::new().fallback(gone);
```

A sub-router with only a `fallback` answers every path under `/v1`, including ones that never existed (`/v1/anything/at/all`). `gone` returns the `410` tuple: status, a header array with the same `link` value, and the JSON body. `/v2` and the unversioned routes are copied from `app`.

## Challenge: `diff_shapes`

Sorted old keys are checked against `new`: absent gives `RemoveResponseField`; present with a different JSON kind (compared with `std::mem::discriminant` on the `Value`) gives `ChangeResponseFieldType`. A different number is the same kind, so `{"a":1}` against `{"a":2}` is empty. Keys only in `new` each give `AddResponseField`. A rename is a remove plus an add, which `v1_to_v2_is_a_rename_plus_an_addition` expects. The function cannot tell a rename from an unrelated pair, and the lesson says so: a tool sees shapes, a human sees intent.
