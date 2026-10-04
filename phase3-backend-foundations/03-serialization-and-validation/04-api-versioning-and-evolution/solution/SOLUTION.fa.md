# راه‌حل — ۳.۳.۴ نسخه‌بندیِ API و تکاملش

کدِ کامل در `solution/src/lib.rs` است و همه‌ی تست‌هایِ `solution/tests/` را رد می‌کند، از جمله `build_test.rs` (پله‌ی «بساز») و `challenge_test.rs` (پله‌ی «چالش»).

## دو پیاده‌سازیِ `From`

```rust
impl From<&Anime> for AnimeV1 {
    fn from(a: &Anime) -> Self {
        AnimeV1 { id: a.id, title: a.title.clone(), status: a.status, rating: a.rating }
    }
}
```

`AnimeV2::from` همین است، با `watch_status: a.status` و `episodes: a.episodes`. گرفتنِ `&Anime` یعنی یک مقدارِ ذخیره‌شده را می‌شود بدونِ کپیِ کلِ struct در هر دو شکل نشان داد؛ فقط `title` کلون می‌شود. قاعده‌هایِ «تعدادِ قسمتِ نامعلوم» و «ratingِ null» اصلاً در کدِ `From` نیستند. رویِ خودِ نوع هستند: `episodes` در `AnimeV2` ویژگیِ `#[serde(default, skip_serializing_if = "Option::is_none")]` دارد، و `rating` هیچ ویژگی‌ای ندارد، پس `None` به‌صورتِ `null` نوشته می‌شود. این همان `v2_omits_unknown_episodes_but_keeps_a_null_rating` است.

## `classify` و `requires_new_version`

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

عمداً بازوی `_` ندارد: اگر واریانتِ چهاردهمی به `Change` اضافه کنی، تا وقتی تصمیم نگیری چیست کامپایل نمی‌شود. این همان `E0004`ای است که درس از طرفِ کلاینت نشان می‌دهد. `requires_new_version` می‌شود `changes.iter().any(|c| classify(*c) == Compat::Breaking)`؛ `any` رویِ برشِ خالی `false` است، که جوابِ لازم برایِ «هیچ تغییری» است.

## `deprecation_headers` و `version_from_accept`

```rust
vec![
    ("deprecation", format!("@{deprecated_at_unix}")),
    ("sunset", sunset.to_string()),
    ("link", format!("<{successor}>; rel=\"successor-version\"")),
]
```

ترتیب و حروفِ کوچکِ نام‌ها جزوِ مشخصات‌اند، پس تست می‌تواند کلِ `Vec` را مقایسه کند. برایِ `Accept`، هر ورودیِ جداشده با کاما در اولین `;` بریده می‌شود (با این کار `q=0.5` می‌افتد)، فاصله‌هایش گرفته می‌شود، و با `eq_ignore_ascii_case` مقایسه می‌شود. حلقه در اولین ورودیِ شناخته‌شده برمی‌گردد، پس «هر کدام زودتر آمد برنده است» از ترتیبِ حلقه درمی‌آید. بعد از حلقه، جوابِ همه‌چیزِ دیگر `V1` است، از جمله `*/*`.

## بساز: `app_after_sunset`

```rust
let v1 = Router::new().fallback(gone);
```

یک روترِ فرعی که فقط `fallback` دارد به هر مسیرِ زیرِ `/v1` جواب می‌دهد، حتی مسیرهایی که هیچ‌وقت وجود نداشتند (`/v1/anything/at/all`). `gone` توپلِ `410` را برمی‌گرداند: کدِ وضعیت، آرایه‌ی هدر با همان مقدارِ `link`، و بدنه‌ی JSON. `/v2` و مسیرهایِ بی‌نسخه از `app` کپی شده‌اند.

## چالش: `diff_shapes`

کلیدهایِ مرتب‌شده‌ی `old` با `new` سنجیده می‌شوند: نبودن یعنی `RemoveResponseField`؛ بودن با نوعِ JSONِ متفاوت (با `std::mem::discriminant` رویِ `Value` مقایسه می‌شود) یعنی `ChangeResponseFieldType`. یک عددِ دیگر همان نوع است، پس `{"a":1}` در برابرِ `{"a":2}` خالی است. کلیدهایی که فقط در `new` هستند هر کدام `AddResponseField` می‌دهند. تغییرِ نام یعنی یک حذف به‌علاوه‌ی یک اضافه، که `v1_to_v2_is_a_rename_plus_an_addition` همین را انتظار دارد. تابع نمی‌تواند تغییرِ نام را از دو فیلدِ بی‌ربط تشخیص بدهد، و درس هم این را می‌گوید: ابزار شکل‌ها را می‌بیند، آدم قصد را.
