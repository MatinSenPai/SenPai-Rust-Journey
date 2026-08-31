# Solution — 2.7.4 Property testing with `proptest`, snapshot testing with `insta`

```rust
pub fn encode_flags(flags: &[bool]) -> String {
    flags.iter().map(|&b| if b { '1' } else { '0' }).collect()
}

pub fn decode_flags(encoded: &str) -> Vec<bool> {
    encoded.chars().map(|c| c == '1').collect()
}

pub fn checklist(items: &[WatchItem]) -> String {
    let mut out = String::new();
    for item in items {
        let mark = if item.watched { "x" } else { " " };
        out.push_str(&format!("[{mark}] {}\n", item.title));
    }
    out
}
```

None of these three wanted anything beyond what you saw in "The concept" — the same two tools, on a slightly different domain.

## `encode_flags` / `decode_flags` — graded by a property, not an example

`encode_flags` maps each `bool` to its matching character with `.map()` and builds a `String` with `.collect()` — the exact same char-to-`String` collect pattern you already know from 2.2. `decode_flags` does exactly the reverse: it compares each `char` to `'1'` and produces a `bool`.

What matters isn't how these two functions happen to be written — it's that their hidden test (`flags_round_trip` in `src/lib.rs`) never checks one specific input. It only checks: `decode_flags(&encode_flags(&flags)) == flags`, across hundreds of random `Vec<bool>`s between zero and fifteen items long. If your implementation keeps that property true — however you happened to write it — the test goes green. proptest doesn't care whether you used `.map()`/`.collect()` or a hand-written loop; only the contract matters to it.

## `checklist` — graded by a snapshot

`checklist` isn't new either: the same pattern as `watch_digest` from "The concept," with a different output format. Its hidden test (`checklist_snapshot`) is different in kind, though — it's an `insta::assert_snapshot!`, not an `assert_eq!`. The file `src/snapshots/p2_07_04_property_and_snapshot_testing__tests__checklist_snapshot.snap` is already approved with this exact text:

```text
[x] Frieren
[x] Bocchi the Rock!
[ ] Dandadan
```

So any implementation that produces this *exact* text — against the same three sample entries the test builds — goes green. `checklist`'s doc comment already stated this same format up front: `[x] title` for watched, `[ ] title` for not, each line ending with `\n`.

## What this lesson was actually about

- **Write a property so the implementing code stays free.** The `flags_round_trip` test never says `encode_flags` has to use `.map()` or a loop — only the final behavior matters to it. That's exactly what separates proptest from a strict, hand-written test.
- **An approved snapshot doesn't change the output spec, it just records it.** The `.snap` above is exactly what `checklist`'s doc comment already said; the snapshot isn't a *second* source of truth, just a machine-readable copy of that same spec.
- **Both techniques worked on code with nothing special about it.** `encode_flags`/`decode_flags`/`checklist` are each a few lines long; what differs is how they're *tested*, not how complex the code itself is. That was exactly the lesson's point: you pick these two tools for the shape of the test, not the size of the code.
