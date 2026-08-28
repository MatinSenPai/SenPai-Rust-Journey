# راه‌حل

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

`.split_once(':')` همان ابزاری است که `parse_config_str` در ۲.۵.۲ برایِ خط‌هایِ `key=value` استفاده کرد — یک `Option<(&str, &str)>` برمی‌گرداند، و `None`اش با `.ok_or(...)?` تبدیل می‌شود به `MissingSeparator`. نیمه‌ی `episode_part` بعدش باید به‌عنوانِ `u32` پارس شود؛ اینجا `.map_err(...)` لازم است (نه یک `?` خام)، دقیقاً به همان دلیلی که فیلدِ `max_retries` در ۲.۵.۲ به آن نیاز داشت و گونه‌هایِ `InvalidScore`/`InvalidEpisode` در این درس نمی‌توانند از `#[from]` استفاده کنند: گونه علاوه بر `ParseIntError`، به `line` و `episode` هم نیاز دارد، و `From::from` فقط همان یک مقداری را می‌گیرد که دارد تبدیلش می‌کند.

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

`.enumerate()` *قبل* از `.filter()` اجرا می‌شود، برایِ همین یک خطِ خالی هم یک شماره‌ی خط مصرف می‌کند — `index + 1` همیشه با جایگاهِ واقعیِ آن خط در `input` جور در می‌آید، خطِ خالی هم جزوش. `.collect::<Result<Vec<_>, WatchNoteError>>()` همان `collect()`ِ توقفِ زودهنگام (short-circuiting) از ۲.۲.۳ است: اولین `Err` کلِ نتیجه می‌شود. `.context(...)` بعدش آن `Result<Vec<WatchNote>, WatchNoteError>` را به `anyhow::Result<Vec<WatchNote>>` تبدیل می‌کند، با پیچیدنِ خطایِ اصلی، نه دور انداختنش — به همین خاطر `err.source()` در تست‌ها هنوز هم همان `WatchNoteError`ِ دقیق را زیرِ پیامِ ثابتِ `"failed to load watch notes"` پیدا می‌کند.

این دقیقاً همان دلیلی است که ۲.۵.۱ و ۲.۵.۲ زحمتِ پیاده‌سازیِ درستِ `std::error::Error` را به خودشان دادند (به‌جایِ اینکه فقط یک `String` برگردانند): `anyhow::Context`، و به‌طورِ کلی تبدیل‌شدنِ `?`/`collect()` به `anyhow::Result`، فقط برایِ نوع‌هایی کار می‌کند که trait واقعیِ `Error` را پیاده کرده باشند. یک خطایِ مبتنی‌بر `String` اصلاً نمی‌توانست به این زنجیره وصل شود.
