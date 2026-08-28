# Solution

```rust
pub fn parse_watch_note(line: usize, text: &str) -> Result<WatchNote, WatchNoteError> {
    let (episode_part, note_part) = text
        .split_once(':')
        .ok_or(WatchNoteError::MissingSeparator { line })?;
    let episode_part = episode_part.trim();
    let episode: u32 = episode_part
        .parse()
        .map_err(|source| WatchNoteError::InvalidEpisode {
            line,
            episode: episode_part.to_string(),
            source,
        })?;
    Ok(WatchNote {
        episode,
        note: note_part.trim().to_string(),
    })
}
```

`.split_once(':')` is the same tool 2.5.2's `parse_config_str` used for `key=value` lines — it returns `Option<(&str, &str)>`, `None` becoming `MissingSeparator` via `.ok_or(...)?`. The `episode_part` half then has to parse as a `u32`; `.map_err(...)` is required here (not a bare `?`) for the same reason 2.5.2's `max_retries` field needed it and this lesson's `InvalidScore`/`InvalidEpisode` variants can't use `#[from]`: the variant needs `line` and `episode` on top of the `ParseIntError`, and `From::from` only ever receives the one value it's converting.

```rust
pub fn load_watch_notes(input: &str) -> anyhow::Result<Vec<WatchNote>> {
    input
        .lines()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| parse_watch_note(index + 1, line))
        .collect::<Result<Vec<_>, WatchNoteError>>()
        .context("failed to load watch notes")
}
```

`.enumerate()` runs *before* `.filter()`, which is why a blank line still consumes a line number — `index + 1` always matches the line's real position in `input`, blank lines included, exactly as documented. `.collect::<Result<Vec<_>, WatchNoteError>>()` is the short-circuiting `.collect()` from 2.2.3: the first `Err` becomes the whole result. `.context(...)` then converts that `Result<Vec<WatchNote>, WatchNoteError>` into `anyhow::Result<Vec<WatchNote>>`, wrapping rather than discarding the original error — which is why `err.source()` in the tests still finds the specific `WatchNoteError` underneath the fixed `"failed to load watch notes"` message.

This is the whole reason 2.5.1 and 2.5.2 bothered implementing `std::error::Error` properly (rather than just returning a `String`): `anyhow::Context`, and `?`/`.collect()` converting into `anyhow::Result` in general, only work for types that implement the real `Error` trait. A `String`-based error couldn't plug into this chain at all.
