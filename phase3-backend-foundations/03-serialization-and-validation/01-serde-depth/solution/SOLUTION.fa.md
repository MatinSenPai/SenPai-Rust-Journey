# راه‌حل — ۳.۳.۱ عمقِ serde

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/src/lib.rs` و `solution/tests/build_test.rs` و `solution/tests/challenge_test.rs` را می‌گذراند.

## پیاده‌سازی ۱ تا ۴: attributeها، و یک خط برایِ هر تابع

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

`rename_all` کلیدهایِ camelCase را در هر دو جهت انجام می‌دهد. `default` رویِ `episode_count` کلیدِ غایب را `0` می‌کند. `skip_serializing_if` رتبه‌ی `None` را از خروجی حذف می‌کند. `default`ِ کنارش رویِ `rating` لازم نیست (یک `Option`ِ غایب از قبل `None` خوانده می‌شود)؛ آنجا است تا دو جهت جفت‌وجور خوانده شوند. تابع‌ها هم می‌شوند `serde_json::to_string(card).unwrap()` و `serde_json::from_str(json).map_err(|e| e.to_string())`. نوشتنِ این نوع‌ها نمی‌تواند شکست بخورد (نه نقشه‌ای با کلیدِ غیررشته، نه `Serialize`ِ دستی‌ای که خطا بدهد)، پس `unwrap` اینجا صادقانه است.

`AnimeDetail` رویِ `card` و `audit` هر دو `#[serde(flatten)]` می‌گیرد؛ `Audit` هم `rename_all = "camelCase"` می‌گیرد تا `createdAt` و `updatedAt` شوند. `flatten` ترتیبِ فیلدها را حفظ می‌کند، پس کلیدهایِ کارت اول می‌آیند. `skip_serializing_if`ِ خودِ کارت داخلِ شیءِ تخت هم هنوز اعمال می‌شود، و `detail_is_one_flat_object` به همین تکیه دارد.

`CreateAnime` یک `#[serde(deny_unknown_fields)]` می‌گیرد، و `#[serde(default)]` رویِ `status` (که از `WatchStatus::default()` یعنی `PlanToWatch` استفاده می‌کند) و رویِ `rating`. `AnimeEvent` یک `#[serde(tag = "type", rename_all = "snake_case")]` می‌گیرد: `rename_all` رویِ یک enum *گونه‌ها* را تغییرنام می‌دهد، پس `StatusChanged` می‌شود `status_changed`. هر گونه یک گونه‌ی struct است، که همان چیزی است که برچسبِ داخلی لازم دارد.

## پیاده‌سازی ۵: `Serialize` برایِ `Rating`

```rust
fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&format_args!("{}/10", self.0))
}
```

`collect_str` هر چیزی را که `Display` است (اینجا آرگومان‌هایِ قالب‌بندی‌شده) به‌صورتِ رشته می‌نویسد. `serializer.serialize_str(&format!("{}/10", self.0))` همان است با یک تخصیصِ بیشتر. اعتبارسنجی نیست: `Rating(0)` به‌صورتِ `"0/10"` نوشته می‌شود.

## بساز: `Deserialize` برایِ `Rating`

```rust
fn visit_u64<E: de::Error>(self, v: u64) -> Result<Rating, E> { check(v) }

fn visit_str<E: de::Error>(self, v: &str) -> Result<Rating, E> {
    let n = v.strip_suffix("/10")
        .and_then(|n| n.parse::<u64>().ok())
        .ok_or_else(|| E::invalid_value(de::Unexpected::Str(v), &self))?;
    check(n)
}
```

`check` تابعِ کوچکی است که هر دو متد با هم استفاده می‌کنند: یک `u64` را وقتی در `1..=10` است به `Rating` تبدیل می‌کند و در غیر این صورت به `E::custom("rating must be between 1 and 10, got N")`. تبدیل از راهِ `u8::try_from` است، پس `300` با `got 300` رد می‌شود و به `44` سرریز نمی‌کند. `deserialize` می‌شود `deserializer.deserialize_any(RatingVisitor)`. یک boolean، `null`، یک اعشاری، یا یک عددِ صحیحِ منفی به یک متدِ `visit_` می‌رسد که visitor ننوشته، و متدِ پیش‌فرض `invalid type: ..., expected <expecting>` گزارش می‌کند، و به همین دلیل `expecting` دقیقاً همان جمله‌ی خواسته‌شده است. رشته‌ای که `N/10` نیست (`"9"`، `"nine"`، `"9/11"`، `"/10"`، `"9/10 "`) در `strip_suffix` یا `parse` رد می‌شود و به یک خطایِ `invalid_value` تبدیل می‌شود. `"9/10 "` یک فاصله‌ی انتهایی دارد، پس به `/10` ختم نمی‌شود.

## چالش: `genre_list`

`serialize` می‌شود `serializer.serialize_str(&genres.join(","))`. `deserialize` با `String::deserialize(deserializer)?` یک `String` می‌خواند، بعد با ویرگول جدا می‌کند، هر تکه را trim می‌کند، تکه‌هایِ خالی را می‌اندازد و جمع می‌کند. ورودیِ آرایه خودبه‌خود `Err` است: `String::deserialize` یک دنباله را نمی‌پذیرد.
