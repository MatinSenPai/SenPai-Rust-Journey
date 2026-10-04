# Solution — 3.3.1 Serde in depth

The full code is `solution/src/lib.rs`; it passes every test in `solution/src/lib.rs`, `solution/tests/build_test.rs` and `solution/tests/challenge_test.rs`.

## Implement 1 to 4: attributes, and one line per function

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeCard {
    pub id: u64,
    pub title: String,
    pub watch_status: WatchStatus,
    #[serde(default)]
    pub episode_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<u8>,
}
```

`rename_all` handles the camelCase keys in both directions. `default` on `episode_count` makes a missing key `0`. `skip_serializing_if` drops a `None` rating from the output. The `default` next to it on `rating` is not needed (a missing `Option` already reads as `None`); it is there so the two directions read as a pair. The functions are then `serde_json::to_string(card).unwrap()` and `serde_json::from_str(json).map_err(|e| e.to_string())`. Writing these types cannot fail (no map with non-string keys, no custom `Serialize` that errors), so `unwrap` is honest here.

`AnimeDetail` puts `#[serde(flatten)]` on both `card` and `audit`; `Audit` gets `rename_all = "camelCase"` for `createdAt` and `updatedAt`. Flattening keeps field order, so the card's keys come first. The card's own `skip_serializing_if` still applies inside the flat object, which `detail_is_one_flat_object` relies on.

`CreateAnime` gets `#[serde(deny_unknown_fields)]`, `#[serde(default)]` on `status` (which uses `WatchStatus::default()`, `PlanToWatch`), and on `rating`. `AnimeEvent` gets `#[serde(tag = "type", rename_all = "snake_case")]`: `rename_all` on an enum renames the *variants*, so `StatusChanged` becomes `status_changed`. Every variant is a struct variant, which is what an internal tag needs.

## Implement 5: `Serialize for Rating`

```rust
fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&format_args!("{}/10", self.0))
}
```

`collect_str` writes anything that is `Display` (here, the formatted arguments) as a string. `serializer.serialize_str(&format!("{}/10", self.0))` is the same thing with one more allocation. No validation: `Rating(0)` is written as `"0/10"`.

## Build: `Deserialize for Rating`

```rust
fn visit_u64<E: de::Error>(self, v: u64) -> Result<Rating, E> { check(v) }

fn visit_str<E: de::Error>(self, v: &str) -> Result<Rating, E> {
    let n = v.strip_suffix("/10")
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or_else(|| E::invalid_value(de::Unexpected::Str(v), &self))?;
    check(n)
}
```

`check` is a small function both methods share: it turns a `u64` into `Rating` when it is in `1..=10` and into `E::custom("rating must be between 1 and 10, got N")` otherwise. It converts through `u8::try_from`, so `300` is rejected with `got 300` and not wrapped to `44`. `deserialize` is `deserializer.deserialize_any(RatingVisitor)`. A boolean, `null`, a float, or a negative integer reaches a `visit_` method the visitor did not write, and the default one reports `invalid type: ..., expected <expecting>`, which is why `expecting` is exactly the required sentence. A string that is not `N/10` (`"9"`, `"nine"`, `"9/11"`, `"/10"`, `"9/10 "`) fails `strip_suffix` or `parse` and becomes an `invalid_value` error. `"9/10 "` has a trailing space, so it does not end with `/10`.

## Challenge: `genre_list`

`serialize` is `serializer.serialize_str(&genres.join(","))`. `deserialize` reads a `String` with `String::deserialize(deserializer)?`, then splits on `,`, trims each piece, drops empty ones, and collects. An array input is an `Err` for free: `String::deserialize` refuses a sequence.
