# ۳.۳.۱ — عمقِ serde

## در یک نگاه

بعد از این درس می‌توانی:

- شکلِ JSONِ یک struct را بدونِ دست‌زدن به فیلدهایش تعیین کنی: کلیدها را تغییرنام بدهی، یک کلید را از خروجی حذف کنی، کلیدِ غایب را پر کنی، دو struct را در یک شیءِ تخت ادغام کنی، و کلیدهایی را که انتظارشان را نداشتی رد کنی.
- بینِ چهار نمایشِ JSON برایِ یک enum انتخاب کنی، و آن یکی را که بی‌سروصدا گونه‌ی اشتباه را برمی‌دارد تشخیص بدهی.
- برایِ نوعی که JSONاش شبیهِ Rustش نیست، `Serialize` را دستی بنویسی و `Deserialize` را با یک `Visitor`.
- خطایِ `E0277`ِ نبودنِ `Deserialize`، خطایِ `E0277`ِ `#[serde(default)]` رویِ نوعی که `Default` ندارد، و خطایِ زمانِ اجرایِ برچسبِ داخلی کنارِ یک عددِ لخت را بخوانی.

**زمان:** حدود ۱۱۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)،
[۲.۳.۱ — تعریف و پیاده‌سازیِ صفت‌ها](../../../phase2-intermediate/03-traits-and-generics/01-defining-and-implementing-traits/README.fa.md)،
[۲.۳.۴ — مشتق‌های استاندارد، دستی پیاده‌سازی‌شده](../../../phase2-intermediate/03-traits-and-generics/04-standard-derives-by-hand/README.fa.md)

---

## چرا اهمیت دارد

در ۳.۲.۳ `Json<CreateAnime>` و `Json<Anime>` نوشتی و کار کرد. پشتش دو `derive` بود، `Serialize` و `Deserialize`، و یک پیش‌فرض: JSON دقیقاً شبیهِ struct است. این پیش‌فرض تا اولین کلاینتِ واقعی درست است. اپِ موبایل `watchStatus` می‌خواهد، نه `watch_status`. صفحه‌ی فهرست نمی‌خواهد در هر ردیف `"rating":null` ببیند. endpointِ ساخت باید یک غلطِ تایپی را رد کند، نه اینکه بیندازدش دور. یک فیدِ رویداد به یک آرایه نیاز دارد که سه شکلِ متفاوت در آن باشد. هرکدامِ این‌ها یک attributeِ یک‌خطی است، و ندانستنِ آن attribute همان چیزی است که آدم را به نوشتنِ یک struct دومِ فقط برایِ تغییرنامِ یک فیلد می‌کشاند.

اگر از Django می‌آیی، این کارِ `Serializer`ِ DRF است: فیلدها را اعلام کن، بگو کدام اختیاری است، با `source=` تغییرنام بده، با `SerializerMethodField` یک مقدار حساب کن. فرق در این است که کجا زندگی می‌کند. سریالایزرِ DRF یک کلاسِ جدا است که کنارِ مدل می‌نویسی. اینجا شکل *attributeهایِ خودِ نوع* است، در زمانِ کامپایل بررسی می‌شود، و همان نوع هر دو جهت را پوشش می‌دهد: نوشتنِ JSON و خواندنش. تشبیه جایی می‌شکند که نیمه‌ی دیگرِ کارِ یک سریالایزر شروع می‌شود. سریالایزرِ DRF اعتبارسنجی هم می‌کند (`validate_rating`، `max_value=10`). `serde` فقط *شکل* را توصیف می‌کند؛ «رتبه ۱ تا ۱۰ است» کارِ [۳.۳.۲ — اعتبارسنجی](../02-validation/README.fa.md) است. یک بخشِ این درس (`Deserialize`ِ دستیِ `Rating`) عمداً به مرز می‌زند، تا دقیقاً ببینی شکل کجا تمام می‌شود و قاعده کجا شروع.

در این درس اصلاً HTTP نیست، فقط `serde` و `serde_json`، پس هر مثال تابعی است از یک مقدار به متن و برعکس. همین attributeها بدونِ تغییر داخلِ `Json<T>` هم کار می‌کنند. [۳.۳.۳ — قراردادهایِ API و OpenAPI (`utoipa`)](../03-api-contracts-and-openapi/README.fa.md) و [۳.۳.۴ — نسخه‌بندیِ API و تکاملش](../04-api-versioning-and-evolution/README.fa.md) در ادامه‌ی همین ماژول می‌آیند.

---

## مفهوم

### `derive(Serialize)` واقعاً چه به تو می‌دهد

`serde` کار را عمداً دو نیم کرده است. نوعِ تو صفتی را پیاده می‌کند که می‌گوید *داده‌اش چیست* (یک struct با این فیلدهایِ نام‌دار، یک گونه‌ی enum، یک رشته). یک crateِ *قالب* (format)، اینجا `serde_json`، تصمیم می‌گیرد آن داده به‌صورتِ متن چه شکلی باشد. هیچ‌کدام دیگری را نمی‌شناسد.

```senpai-visual
{"kind":"concept","labels":["نوعِ تو","Serialize: می‌گوید داده چیست","Serializer: serde_json می‌گوید چه شکلی است","متنِ JSON","Deserialize و Visitor: دوباره می‌خواندش"]}
```

`#[derive(Serialize, Deserialize)]` هر دو impl را برایت فیلد به فیلد می‌نویسد. هرچه در ادامه می‌آید یا یک attribute است که چیزی را که derive می‌نویسد عوض می‌کند، یا یک implِ دستی که جایش را می‌گیرد. دو جهت از هم جدایند، و ریشه‌ی بیشترِ غافلگیری‌ها همین است: `skip_serializing_if` فقط به نوشتن مربوط است و `default` فقط به خواندن.

### تغییرنام، حذف، پرکردن

شکلِ کارتی را بگیر که صفحه‌ی فهرست می‌خواهد. کلیدها camelCase، `rating` وقتی نیست از خروجی حذف شود، و `episodeCount` در ورودی اختیاری باشد:

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

`examples/01-rename-skip-default.rs` آن را دو بار می‌نویسد (با رتبه و بی‌رتبه) و بعد دو ورودی می‌خواند:

```text
{"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28,"rating":9}
{"id":1,"title":"Frieren","watchStatus":"watching","episodeCount":28}
Ok(Card { id: 2, title: "Mushishi", watch_status: Completed, episode_count: 0, rating: None })
Err(Error("missing field `watchStatus`", line: 1, column: 54))
```

خط‌به‌خط: کلیدها به ترتیبِ تعریفِ فیلدها بیرون می‌آیند. با `rating: None` کلید حذف شده، نه `null`. `episodeCount`ِ غایب شد `0`، چون `default` برایِ یک `u32` همان `Default::default()` را صدا می‌زند. `rating`ِ غایب بدونِ هیچ attributeی شد `None`، چون `Deserialize`ِ derive‌شده یک فیلدِ `Option`ِ غایب را `None` می‌گیرد. و کلیدِ snake_caseِ `watch_status` فهمیده نشد: `rename_all` در *هر دو* جهت تغییرنام می‌دهد، پس نامِ Rust دیگر کلیدِ معتبری نیست. خطا `missing field` می‌گوید و نه «کلیدِ ناشناخته»، چون به‌طورِ پیش‌فرض کلیدِ ناشناخته ساده نادیده گرفته می‌شود؛ کلیدِ `watch_status` دور انداخته شد و بعد `watchStatus` غایب بود.

*اول نشان بده، بعد نام بگذار:* `rename_all` (attributeِ container، رویِ همه‌ی فیلدها اثر دارد)، `default` و `skip_serializing_if` (attributeهایِ فیلد). این همان `required=False` و `default=` و «این کلید را نگذار»ِ DRF است، هرکدام در یک جا. `rename_all` مقدارهایِ `"snake_case"`، `"SCREAMING_SNAKE_CASE"`، `"kebab-case"` و چندتایِ دیگر را هم می‌گیرد؛ یک فیلد با `#[serde(rename = "...")]` می‌تواند خودش را استثنا کند.

### ادغامِ struct‌ها: `flatten`

صفحه‌ی جزئیات فیلدهایِ کارت و زمان‌هایِ audit را در **یک** شیءِ تخت می‌خواهد، نه `{"card":{...},"audit":{...}}`. در Rust دو struct نگه می‌داری (جاهایِ دیگر هم به کار می‌روند) و در سیم یک شیء می‌خواهی:

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

`examples/02-flatten.rs` یک `Detail` می‌نویسد، برش می‌گرداند، و بعد شیئی می‌خواند که کلیدهایی دارد که `WithExtras` نمی‌شناسد:

```text
{"id":1,"title":"Frieren","created_at":"2026-01-05"}
Detail { card: Card { id: 1, title: "Frieren" }, audit: Audit { created_at: "2026-01-05" } }
WithExtras { id: 7, extra: {"studio": String("Madhouse"), "year": Number(2023)} }
```

`flatten` رویِ یک فیلدِ struct، کلیدهایِ آن struct را در والد می‌پاشد، در هر دو جهت. `flatten` رویِ یک نقشه (map) یک جمع‌کننده است: هر کلیدی که هیچ فیلدِ نام‌داری ادعایش نکرده در نقشه می‌افتد. این‌طور داده‌ی ناشناخته را نگه می‌داری به‌جایِ دورریختنش، مثلاً در پراکسی‌ای که باید فیلدهایی را که نمی‌فهمد عبور بدهد. (هزینه‌ای دارد: برایِ جداکردنِ کلیدها `serde` باید هنگامِ خواندن کلِ شیء را بافر کند. ابزارِ «شکل» است، نه حلقه‌ی داغ.)

### ردِ کلیدهایِ ناخواسته: `deny_unknown_fields`

به‌طورِ پیش‌فرض کلیدِ ناشناخته نادیده گرفته می‌شود. برایِ یک *پاسخ* که از دیگری می‌خوانی دقیقاً همین را می‌خواهی: او ممکن است فردا فیلد اضافه کند. برایِ یک *درخواست* که API خودت می‌گیرد، این باگ را پنهان می‌کند، چون `{"title":"Frieren","ratng":9}` یک انیمه بدونِ رتبه می‌سازد و موفقیت گزارش می‌کند. غلطِ تایپی در سکوت دور ریخته می‌شود.

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Strict {
    title: String,
    rating: Option<u8>,
}
```

`examples/06-deny-unknown-fields.rs` غلطِ تایپی را به یک struct به نامِ `Lenient` (همان فیلدها، بدونِ attribute)، به `Strict`، و به structِ سومی می‌دهد که `deny_unknown_fields` را با یک نقشه‌ی جمع‌کننده‌ی `flatten` ترکیب کرده:

```text
Ok(Lenient { title: "Frieren", rating: None })
Err(Error("unknown field `ratng`, expected `title` or `rating`", line: 1, column: 26))
Err(Error("unknown field `ratng`", line: 1, column: 29))
```

`Lenient` می‌پذیرد و `rating` می‌شود `None`، بدونِ هیچ شکایتی. `Strict` رد می‌کند و کلیدِ ناشناخته را نام می‌برد *و* فهرستِ کلیدهایِ مورد انتظار را می‌دهد؛ این همان پیامی است که می‌خواهی به کلاینتِ API برگردانی. خطِ سوم تله است: مستنداتِ `serde` می‌گوید `deny_unknown_fields` همراهِ `flatten` پشتیبانی نمی‌شود. اینجا رد می‌کند، ولی با پیامِ کوتاه‌تر که فهرستِ کلیدهایِ مورد انتظار را ندارد، و نقشه‌ی جمع‌کننده کنارِ یک attributeِ «ردکن» دو حرفِ مخالف را درباره‌ی یک کلید می‌زند. ترکیبشان نکن؛ یا «کلیدهایِ ناشناخته را نگه دار» یا «کلیدهایِ ناشناخته را رد کن».

### enumها: چهار راه برایِ نوشتنِ یک داده

enum جایی است که JSON و Rust بیشترین اختلاف را دارند، چون JSON «یکی از این شکل‌ها» ندارد. `serde` چهار نمایش می‌دهد. یک enum، چهار attribute (`examples/03-tagged-enums.rs`):

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

مثال `Event::Added { id: 1, title: "Frieren" }` را در هر نمایش چاپ می‌کند، و بعد سه ورودی را به نسخه‌ی برچسبِ داخلی و یکی را به نسخه‌ی بی‌برچسب می‌خواند:

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

| Attribute | JSON برایِ `Added` | کی |
|---|---|---|
| هیچ (برچسبِ بیرونی) | `{"Added":{"id":1,"title":"Frieren"}}` | پیش‌فرض؛ نامِ گونه تنها کلید است |
| `tag = "type"` (برچسبِ داخلی) | `{"type":"Added","id":1,"title":"Frieren"}` | چیزی که بیشترِ APIهایِ JSON و فیدهایِ رویداد به کار می‌برند |
| `tag = "type", content = "data"` (برچسبِ مجاور) | `{"type":"Added","data":{"id":1,"title":"Frieren"}}` | وقتی بار باید در کلیدِ خودش بماند |
| `untagged` | `{"id":1,"title":"Frieren"}` | وقتی خودِ داده گونه‌ها را از هم جدا می‌کند |

برایِ فیدِ رویداد، شکلِ برچسبِ داخلی همان است که باید سراغش بروی: کلاینت می‌تواند بدونِ نگاه‌کردن به داخل رویِ `type` شاخه شود. خطایِ خوب هم می‌دهد، همان‌طور که رونوشت نشان می‌دهد: یک `type`ِ ناشناخته گونه را نام می‌برد و گونه‌هایِ معتبر را فهرست می‌کند، و `type`ِ غایب می‌گوید ``missing field `type` ``. خطِ آخرِ رونوشت تضاد است. `untagged` برچسبی ندارد که از آن شکایت کند، پس بهترین چیزی که می‌تواند بگوید `data did not match any variant` است، بدونِ اینکه بگوید کدام کلید غلط بود و بدونِ موقعیت. این خطایِ ضعیف بهای بی‌برچسب‌بودن است؛ بهایِ دیگرش پاراگرافِ بعد است.

`untagged` گونه‌ها را **به ترتیب، از بالا به پایین امتحان می‌کند و اولینی که جور شود برنده است**. اگر دو گونه هر دو بتوانند یک JSON را بپذیرند، اولی آن را می‌برد و دومی هرگز نمی‌رسد. هیچ‌چیز هشدار نمی‌دهد. «خطاهایی که خواهی دید» دقیقاً همین حالت را اجرا می‌کند.

### نوشتنِ دستیِ `Serialize`

گاهی JSON کپیِ فیلد به فیلدِ struct نیست. اینجا پاسخ یک `label`ِ محاسبه‌شده دارد که هیچ‌جا ذخیره نشده، و `rating`ای که وقتی `None` است حذف می‌شود. می‌توانستی struct دومی اضافه کنی؛ یا صفت را پیاده کنی:

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

`examples/04-hand-serialize-struct.rs` یک انیمه با رتبه و یکی بی‌رتبه می‌نویسد:

```text
{"id":1,"title":"Frieren","rating":9,"label":"#1 Frieren"}
{"id":2,"title":"Mushishi","label":"#2 Mushishi"}
```

شکلِ هر implِ دستیِ struct: از serializer یک «نویسنده‌ی struct» بخواه (`serialize_struct`، با نامِ نوع و تعدادِ فیلدهایی که *می‌خواهی بنویسی*؛ `serde_json` این عدد را نادیده می‌گیرد، ولی قالب‌هایِ دیگر به آن تکیه می‌کنند، پس درستش بنویس)، فیلدها را یکی‌یکی بفرست، و با `.end()` تمامش کن. `?`ها خطایِ خودِ قالب را بالا می‌دهند. `S` هر قالبی است که دارد می‌نویسد (اینجا `serde_json`)؛ کدِ تو هرگز اسمِ JSON را نمی‌آورد. برایِ نوعی که یک مقدارِ واحد است، مثلِ `Rating` که پایین پیاده می‌کنی، نویسنده‌ی struct نیست: یک متد صدا می‌زنی، مثلاً `serialize_str`، و تمام.

### خواندنِ دستی: `Deserialize` و یک `Visitor`

خواندن از نوشتن سخت‌تر است، به دلیلی که دیدنش می‌ارزد. وقتی *می‌نویسی* مقدارت را می‌شناسی. وقتی *می‌خوانی* نمی‌دانی JSON چه دارد: رشته، عدد، شیء. پس `serde` کنترل را برمی‌گرداند. تو به deserializer می‌گویی «این شکل‌ها را می‌توانم بپذیرم»، با پیاده‌کردنِ یک `Visitor` (بازدیدکننده) که برایِ هر شکل یک متد دارد، و deserializer متدی را که با چیزِ پیداشده جور است صدا می‌زند.

`examples/05-visitor-deserialize.rs` یک `Minutes` را از `24` یا `"24m"` می‌خواند. visitor این است:

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

خودِ implِ `Deserialize` یک خط است، `d.deserialize_any(MinutesVisitor)`: «به ورودی نگاه کن و هر `visit_`ای که جور است صدا بزن». پنج ورودی:

```text
    24 -> Ok(Minutes(24))
 "24m" -> Ok(Minutes(24))
  "24" -> Err(Error("missing `m`", line: 1, column: 4))
  true -> Err(Error("invalid type: boolean `true`, expected a number of minutes like 24, or a string like \"24m\"", line: 1, column: 4))
    -3 -> Err(Error("invalid type: integer `-3`, expected a number of minutes like 24, or a string like \"24m\"", line: 1, column: 2))
```

دو عدد و یک رشته با پسوند کار می‌کنند. `"24"` به `visit_str` می‌رسد و با پیامِ *خودت* شکست می‌خورد. `true` و `-3` به هیچ متدی که نوشته‌ای نمی‌رسند، پس متدِ پیش‌فرضِ صفت برایت شکست می‌دهد، و پیام از `expecting` ساخته می‌شود: ``invalid type: boolean `true`, expected a number of minutes like 24, or a string like "24m"``. به همین دلیل `expecting` مهم است: آن را مثلِ انتهایِ جمله‌ی «expected ...» بنویس. `E::custom` خطایی از هر نوعی که قالب استفاده می‌کند می‌سازد، پس visitorِ تو به `serde_json` وابسته نیست.

جایِ DRF: این `to_internal_value` و `to_representation` برایِ یک فیلد است، به‌علاوه‌ی کمی `validate_<field>`. فرق این است که visitor تایپ‌دار است. دیکشنریِ `data`ی از جنسِ `Any` نیست که بگردی؛ هر شکل به‌صورتِ یک مقدارِ Rust در متدی که برایش ساخته شده می‌رسد.

---

## دست‌به‌کد

در این درس سرور نیست: هر دستور برنامه‌ای است که چند خط چاپ می‌کند. شش‌تایی را که کامپایل می‌شوند اجرا کن و با رونوشت‌هایِ بالا مقایسه کن:

```sh
cargo run -p p3-03-01-serde-depth --example 01-rename-skip-default
cargo run -p p3-03-01-serde-depth --example 02-flatten
cargo run -p p3-03-01-serde-depth --example 03-tagged-enums
cargo run -p p3-03-01-serde-depth --example 04-hand-serialize-struct
cargo run -p p3-03-01-serde-depth --example 05-visitor-deserialize
cargo run -p p3-03-01-serde-depth --example 06-deny-unknown-fields
```

(تا وقتی `src/lib.rs` ناتمام است، `cargo` قبل از هر اجرا حدودِ دوازده هشدارِ `unused variable` از اسکلتِ `todo!()`ِ تو چاپ می‌کند. با پیاده‌کردنِ تابع‌ها از بین می‌روند. رونوشت‌هایِ این درس آن‌ها را حذف کرده‌اند و فقط خروجیِ برنامه یا خطایِ مهم را نشان می‌دهند.) مثال‌هایِ `07` و `08` و `10` عمداً خراب‌اند و پشتِ feature به نامِ `broken` هستند؛ `09` اجرا می‌شود و فقط غلط است. «خطاهایی که خواهی دید» هر چهار را پوشش می‌دهد، و `11` و `12` رفعِ آن‌هایند.

تست‌ها قرمز شروع می‌شوند. `src/lib.rs` کلِ اسکلت را دارد و هر تابع یک `todo!()` است با یک doc comment که دقیقاً می‌گوید باید چه کند و با چه JSONای کار می‌کند:

```sh
cargo test -p p3-03-01-serde-depth --lib 2>&1 | grep 'test result'
```

```text
test result: FAILED. 1 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(تستِ تنها که می‌گذرد همان `Anime`ِ کاملِ ۳.۲.۳ است.) نردبان سه فایلِ تست دیگر دارد: `tests/build_test.rs` برایِ پله‌ی «بساز» و `tests/challenge_test.rs` برایِ چالش. این‌ها مجموع‌هایِ اجرایِ آن‌ها رویِ کدِ تمام‌شده‌ی `solution/` است:

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

بعد این‌ها را امتحان کن:

۱. در `examples/01-rename-skip-default.rs` به `episode_count` این را اضافه کن: `#[serde(rename = "eps")]`. JSONِ نوشته‌شده چه شکلی می‌شود، و آیا ورودیِ `{"episodeCount":3,...}` هنوز خوانده می‌شود؟
۲. در `examples/02-flatten.rs` به `Card` و `Audit` یک فیلد با نامِ یکسان بده (مثلاً `id`). JSONِ نوشته‌شده چه دارد؟ آیا هنوز برگردانده می‌شود؟
۳. در `examples/03-tagged-enums.rs` ورودیِ `{"type":"added","id":1,"title":"x"}` (با حروفِ کوچک) را به enumِ `Internal` بده. کدام پیام می‌گیری، و کدام attribute آن را موفق می‌کند؟

---

## خطاهایی که خواهی دید

(هشدارهایِ `todo!()`ِ `src/lib.rs` از رونوشت‌هایِ زیر حذف شده‌اند، تا هرکدام از خطایِ واقعی شروع شود.)

### `E0277` — کرانِ `Rating: Deserialize<'de>` برقرار نیست

`Review` را `Deserialize` derive می‌کند، ولی نوعِ فیلدش `Rating` فقط `Serialize` derive می‌کند. `examples/07-missing-deserialize-bound-broken.rs`:

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

**کامپایلر به چه اعتراض می‌کند:** derive برایِ `Review` یک `deserialize` می‌نویسد که باید برایِ کلیدِ `rating` یک `Rating` بخواند، و این به `Rating: Deserialize` نیاز دارد. `Rating` چنین implی ندارد. پیام بلند است چون derive به سه جا بسط پیدا می‌کند که به آن نیاز دارند (خواندنِ فیلد در شکلِ دنباله، در شکلِ نقشه، و بازگشتِ «missing field»)، و کامپایلر هرکدام را گزارش می‌کند. خطِ اولی، نبودنِ impl برایِ `Rating`، کلِ ماجراست؛ دوتایِ دیگر همان واقعیت از نقطه‌هایِ دیگرِ فراخوانی‌اند. `note: for local types consider adding #[derive(serde::Deserialize)]` همان رفع است که برایت نوشته شده.

**رفع:** derive کنش، یا وقتی JSON کپیِ ساده‌ی struct نیست دستی بنویسش، همان‌طور که در پله‌ی «بساز» برایِ `Rating` می‌کنی:

```rust
#[derive(Debug, Serialize, Deserialize)]
struct Rating(u8);
```

**چرا این رفع است:** `Deserialize`ِ derive‌شده فقط به‌اندازه‌ی فیلدهایش توانا است. هر نوعِ تودرتو به هر دو جهتی که به کار می‌بری نیاز دارد. همان قاعده‌ای است که `#[derive(Default)]` برایِ `Default` رویِ هر فیلد دارد (۲.۳.۴).

### `E0277` — `#[serde(default)]` رویِ نوعی که `Default` ندارد

`examples/08-default-without-default-broken.rs` یک `#[serde(default)]` رویِ فیلدی می‌گذارد که نوعش یک enum بدونِ `Default` است:

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

**کامپایلر به چه اعتراض می‌کند:** `#[serde(default)]` یعنی «وقتی کلید غایب است از `Default::default()` استفاده کن». کدِ تولیدشده آن را برایِ `Status` صدا می‌زند و `Status` آن صفت را پیاده نمی‌کند. خطایِ اول به خودِ attribute اشاره می‌کند، دومی به derive‌ای که آن را بسط داده.

**رفع:** بگو کدام گونه پیش‌فرض است. کتابخانه‌ی استاندارد برایش attribute دارد:

```rust
#[derive(Debug, Deserialize, Default)]
enum Status {
    Watching,
    #[default]
    Dropped,
}
```

**چرا این رفع است:** `#[derive(Default)]` رویِ یک enum یک گونه با `#[default]` لازم دارد (`WatchStatus` در `src/lib.rs` دقیقاً همین کار را برایِ `PlanToWatch` می‌کند). وقتی پیش‌فرض نباید `Default`ِ خودِ نوع باشد، `#[serde(default = "path::to::function")]` به‌جایش تابعی از خودت را صدا می‌زند، پس کلیدِ غایب می‌تواند چیزی بگوید که `Default`ِ خودِ نوع نمی‌گوید.

### یک `Err`ِ زمانِ اجرا: برچسبِ داخلی کنارِ یک عددِ لخت

اینجا هیچ‌چیز کامپایلر را متوقف نمی‌کند. `examples/10-tagged-newtype-panic-broken.rs` یک enumِ برچسب‌داخلی دارد با گونه‌ی `ById(u64)`:

```text
{"type":"ByTitle","title":"Frieren"}

thread 'main' (42116) panicked at phase3-backend-foundations\03-serialization-and-validation\01-serde-depth\examples\10-tagged-newtype-panic-broken.rs:20:55:
called `Result::unwrap()` on an `Err` value: Error("cannot serialize tagged newtype variant Lookup::ById containing an integer", line: 0, column: 0)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(شناسه‌ی ریسمان در پرانتز در هر اجرا عوض می‌شود.) خطِ اول درست چاپ شد: گونه‌ی structِ `ByTitle` کلید دارد، پس کلیدِ `type` می‌تواند کنارشان بنشیند. بعد `Lookup::ById(1)` شکست خورد. پنیکِ اینجا فقط `.unwrap()`ِ خودمان است؛ شکستِ واقعی همان `Err`ای است که unwrap شد.

**واقعاً چه خراب است:** برچسبِ داخلی یک کلیدِ اضافه *داخلِ شیءِ گونه* است. گونه‌ی struct یک شیء است، پس کار می‌کند. `ById(1)` باید می‌شد `{"type":"ById", ???}`: یک عددِ لخت کلیدی ندارد که برچسب کنارش بنشیند، و `serde` فقط هنگامِ نوشتن می‌تواند بفهمد. derive کامپایل شد؛ `Err` با اولین مقداری که به گونه‌ی بد می‌خورد می‌رسد. بدتر، `ByTitle` کار کرد، پس یک تستِ سریع از مسیرِ موفق می‌گذشت.

**رفع:** هر گونه‌ی یک enumِ برچسب‌داخلی را یک گونه‌ی struct کن (یا یک گونه‌ی چندتایی که یک struct نگه می‌دارد). `examples/12-tagged-struct-variant-fix.rs`:

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

**چرا این رفع است:** حالا هر دو گونه شیءهایی با کلیدِ نام‌دارند و برچسب جایی برایِ نشستن دارد. اگر واقعاً بارِ لخت لازم داری، برچسبِ مجاور (`tag` به‌علاوه‌ی `content`، از جدولِ بالا) را به کار ببر تا بار کلیدِ خودش را داشته باشد.

### هیچ خطایی نیست: `untagged` گونه‌ی اشتباه را برمی‌دارد

`examples/09-untagged-wrong-variant-trap.rs` زیرِ `#[serde(untagged)]` یک شکلِ پیش‌نویس (رتبه اختیاری) و یک شکلِ کامل (رتبه و وضعیت لازم) دارد، و بدنه‌ای می‌خواند که واضح است همان کامل است:

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

**واقعاً چه خراب است:** کامپایل می‌شود، اجرا می‌شود، و `Ok` برمی‌گرداند. ولی بدنه یک `status` داشت و نتیجه یک `Draft` است، پس `status` دور ریخته شد. `Draft` اول امتحان می‌شود؛ فقط `title` و یک `rating`ِ اختیاری می‌خواهد، بدنه هر دو را دارد، و کلیدِ ناشناخته (`status`) به‌طورِ پیش‌فرض نادیده گرفته می‌شود. اولین جور برنده است، به `Full` هرگز نمی‌رسد، و هیچ تستی که فقط «پارس شد؟» را بررسی کند چیزی نمی‌فهمد.

**رفع:** مشخص‌ترین گونه را اول بگذار، `examples/11-untagged-specific-first-fix.rs`:

```text
Full { title: "Frieren", rating: 9, status: "watching" }
Draft { title: "Frieren", rating: None }
```

**چرا این رفع است:** `untagged` یعنی «هرکدام را به ترتیب امتحان کن»، پس ترتیب همان قاعده است. رفعِ بهتر معمولاً این است که دیگر `untagged` نباشی: `tag` انتخاب را صریح می‌کند، و منتقل‌کردنِ فیلدهایِ `Draft` به یک structِ جدا که `deny_unknown_fields` دارد هم باعث می‌شد تلاشِ اول برایِ بدنه‌ای با کلیدِ `status` شکست بخورد. درسی که باید نگه داری این است که `untagged` تصمیم‌ها را پنهان می‌کند؛ فقط وقتی به کارش ببر که شکل‌ها واقعاً نمی‌توانند همپوشانی داشته باشند.

---

## تمرین

### گرم‌کردن

<details>
<summary>یک struct <code>#[serde(rename_all = "camelCase")]</code> و فیلدِ <code>watch_status</code> دارد. کلاینت کلیدِ <code>watch_status</code> می‌فرستد. چه می‌شود؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

پذیرفته نمی‌شود. `rename_all` در هر دو جهت تغییرنام می‌دهد، پس تنها کلیدی که در فیلد خوانده می‌شود `watchStatus` است. کلیدِ ناشناخته‌ی `watch_status` نادیده گرفته می‌شود (`deny_unknown_fields` نیست)، و نتیجه ``missing field `watchStatus` `` است. مثالِ ۰۱ دقیقاً همین خطا را نشان می‌دهد.

</details>

<details>
<summary>آیا <code>#[serde(skip_serializing_if = "Option::is_none")]</code> چگونگیِ <em>خواندنِ</em> یک کلیدِ <code>rating</code>ِ غایب را عوض می‌کند؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

نه، فقط نوشتن را تغییر می‌دهد. یک فیلدِ `Option`ِ غایب در `Deserialize`ِ derive‌شده، با هر attribute یا بی‌attribute، از قبل `None` خوانده می‌شود. به همین دلیل یک فیلد اغلب هم `default` دارد هم `skip_serializing_if`: یکی برایِ هر جهت.

</details>

<details>
<summary>در <code>#[serde(untagged)] enum E { A { x: Option&lt;u8&gt; }, B { x: u8, y: u8 } }</code> کدام گونه <code>{"x":1,"y":2}</code> را می‌خواند؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`A`. اول امتحان می‌شود، `x` جور است، و `y`ِ ناشناخته نادیده گرفته می‌شود. به `B` برایِ هر ورودی‌ای که `A` بپذیرد هرگز نمی‌رسد. عوض‌کردنِ ترتیب، یا برچسب‌دارکردنِ enum، درستش می‌کند.

</details>

<details>
<summary>آیا یک enumِ برچسب‌داخلی با گونه‌ی <code>ById(u64)</code> کامپایل می‌شود؟ مشکل کی پیدا می‌شود؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

کامپایل می‌شود. مشکل در زمانِ اجرا پیدا می‌شود، به‌صورتِ یک `Err` وقتی یک مقدارِ `ById` *نوشته* می‌شود: در یک عددِ لخت کلیدی نیست که `type` کنارش بنشیند. گونه‌هایِ struct هرگز این مشکل را ندارند، و به همین دلیل تستی که فقط گونه‌هایِ دیگر را بنویسد از آن می‌گذرد.

</details>

<details>
<summary>چرا <code>CreateAnime</code> <code>deny_unknown_fields</code> می‌گیرد ولی structی که از یک API شخصِ ثالث می‌خوانی معمولاً نباید؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

برایِ بدنه‌ی درخواستِ خودت، کلیدِ ناشناخته تقریباً همیشه باگِ کلاینت است (یک غلطِ تایپی)، و ردکردنش به او می‌گوید. برایِ پاسخِ دیگری، کلیدِ ناشناخته به احتمالِ زیاد فیلدی است که بعد از نوشتنِ کدِ تو اضافه کرده، و ردکردنش تغییرِ بی‌ضررِ او را به قطعیِ سرویسِ تو تبدیل می‌کند.

</details>

### تعمیر

هر چهار را درست کن، و هرکدام را با دستورِ سرفایلش بررسی کن:

۱. `examples/07-missing-deserialize-bound-broken.rs` کامپایل شود (با `--features broken` بیلدش کن).
۲. `examples/08-default-without-default-broken.rs` کامپایل شود، و خواندنِ `{"title":"Frieren"}` بدهد `Dropped`.
۳. `examples/10-tagged-newtype-panic-broken.rs` بدونِ پنیک اجرا شود و دو خطِ JSON چاپ کند. گونه‌ی `ById` را فقط حذف نکن.
۴. `examples/09-untagged-wrong-variant-trap.rs` یک `Full` چاپ کند. دو رفعِ متفاوت پیدا کن.

### پیاده‌سازی

هر چیزی در `src/lib.rs` که با «Implement 1» تا «Implement 5» علامت خورده (`todo!()`هایِ «بساز» و چالش در پله‌هایِ خودشان می‌آیند)، با attributeهایِ `#[serde(...)]` که رویِ نوع‌ها می‌روند. هر doc comment JSONِ دقیق را مشخص می‌کند (ترتیبِ کلیدها، کدام کلیدها حذف می‌شوند، کدام اختیاری‌اند)، پس هرگز لازم نیست برایِ دانستنِ آنچه باید بسازی به تست‌ها نگاه کنی. پنج قطعه، هرکدام یک ایده از «مفهوم»:

۱. `AnimeCard`: کارتِ camelCase (`rename_all`، `default`، `skip_serializing_if`)، به‌علاوه‌ی `card_to_json` و `card_from_json`.
۲. `AnimeDetail`: کارت و `Audit` در یک شیءِ تخت (`flatten`)، به‌علاوه‌ی `detail_to_json` و `detail_from_json`.
۳. `CreateAnime`: بدنه‌ی درخواستی که غلطِ تایپی را رد می‌کند و وضعیتِ پیش‌فرض دارد (`deny_unknown_fields`، `default`)، به‌علاوه‌ی `decode_create_anime`.
۴. `AnimeEvent`: فیدِ رویدادِ برچسب‌داخلی، به‌علاوه‌ی `events_to_json` و `events_from_json`.
۵. `Rating`: `Serialize`ِ دستی که `"9/10"` می‌نویسد.

```sh
cargo test -p p3-03-01-serde-depth --lib
```

۲۵ تست در `src/lib.rs` است، به تفکیکِ قطعه. دقت کن توابع در شکست چه برمی‌گردانند: *متنِ خطایِ `serde_json`* بی‌تغییر. تست‌ها از همین راه بررسی می‌کنند که یک غلطِ تایپی واقعاً در پیام نام برده شده.

### بساز

`Deserialize` را برایِ `Rating` با یک `Visitor` بنویس، در همان implی که در `src/lib.rs` منتظر است. یا رشته‌ی `"9/10"` را می‌خواند یا عددِ لخت `9` را، فقط ۱ تا ۱۰ را می‌پذیرد، و در غیر این صورت با یک پیامِ دقیق شکست می‌خورد. این مرز با [۳.۳.۲](../02-validation/README.fa.md) است: خواندن جایی است که «شکل» و «قاعده» به هم می‌رسند، و doc comment دقیقاً می‌گوید هر خطا از کجا می‌آید. طوری بنویسش که مقداری که رشته یا عددِ نامنفی نیست متنِ `expecting`ِ تو را گزارش کند.

```sh
cargo test -p p3-03-01-serde-depth --test build_test
```

`tests/build_test.rs` هشت تست دارد، از جمله یک تستِ نوشتن‌بعد‌خواندن برایِ هر رتبه از ۱ تا ۱۰.

### چالش (اختیاری)

`AnimeGenres` ژانرها را در یک `Vec<String>` نگه می‌دارد ولی قالبِ سیم یک رشته‌ی جداشده با ویرگول است، از راهِ `#[serde(with = "genre_list")]`. دو تابعِ ماژولِ `genre_list` را پیاده کن. `with` راهِ سومِ شخصی‌سازیِ serde است، بینِ یک attribute و یک implِ کاملاً دستی: فقط دو تابعِ فیلد را می‌نویسی و derive بقیه را انجام می‌دهد. فایل `tests/challenge_test.rs` است (پنج تست). عمداً کوچک مانده؛ درسِ بعد، [۳.۳.۲](../02-validation/README.fa.md)، جایی است که «این فهرست نباید خالی باشد» می‌رود.

```sh
cargo test -p p3-03-01-serde-depth --test challenge_test
```

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| `rename_all` / `rename` | تغییرِ نامِ کلیدهایِ JSON، در هر دو جهت | هر APIای که کلاینت‌هایش Rust نیستند |
| `default` | مقداری که وقتی کلید هنگامِ خواندن غایب است به کار می‌رود | فیلدهایِ اختیاریِ درخواست، تغییراتِ افزایشیِ API |
| `skip_serializing_if` | وقتی یک شرط درست است کلید را از JSONِ نوشته‌شده حذف می‌کند | فیلدهایِ `Option`، فهرست‌هایِ خالی |
| `flatten` | کلیدهایِ یک struct تودرتو را در والد می‌پاشد، یا کلیدهایِ باقی‌مانده را در یک نقشه جمع می‌کند | بلوک‌هایِ مشترکِ «audit» یا «صفحه‌بندی» |
| `deny_unknown_fields` | خواندن رویِ کلیدی که struct نمی‌شناسد شکست می‌خورد | بدنه‌ی درخواست‌ها، فایل‌هایِ پیکربندی |
| enumِ برچسب‌دار / بی‌برچسب | شکلِ نوشتنِ گونه‌ی یک enum: بیرونی، داخلی (`tag`)، مجاور (`tag` و `content`)، یا هیچ | فیدهایِ رویداد، بارهایِ چندشکلی |
| `Visitor` | شیئی که می‌گوید یک نوع از کدام شکل‌هایِ JSON خوانده می‌شود، با یک متد برایِ هر شکل | هر نوعی که JSONاش کپیِ فیلدهایش نیست |
| `serde(with = "...")` | یک فیلد را از یک جفت تابعِ خودت رد می‌کند | فیلدی که قالبِ سیمِ خودش را دارد |

### الان می‌دانی

- attributeها شکلِ JSON را بدونِ تغییرِ نوع عوض می‌کنند، و هرکدام در یک جهت یا هر دو کار می‌کند: `skip_serializing_if` می‌نویسد، `default` می‌خواند، `rename_all` هر دو.
- کلیدِ ناشناخته به‌طورِ پیش‌فرض در سکوت دور ریخته می‌شود؛ `deny_unknown_fields` آن را به خطایی تبدیل می‌کند که کلید را نام می‌برد، و با یک `flatten`ِ جمع‌کننده قاطی نمی‌شود.
- enumها چهار نمایشِ JSON دارند؛ برچسبِ داخلی انتخابِ معمولِ API است، هر گونه‌اش باید struct باشد، و `untagged` همیشه یعنی «اولین گونه‌ای که جور شود».
- `Serialize` با یک نویسنده‌ی struct و `Deserialize` با یک `Visitor` نوشته می‌شود؛ implهایِ تو هرگز `serde_json` را نمی‌آورند، فقط صفت‌هایِ مستقل از قالب را.
- derive به صفت رویِ هر نوعِ فیلد نیاز دارد، و `#[serde(default)]` به `Default` رویِ نوعِ فیلد.

### بعداً کامل‌تر می‌بینی

- **قاعده‌ها رویِ شکل (`1..=10`، ناتهی، ایمیل)**: [۳.۳.۲ — اعتبارسنجی](../02-validation/README.fa.md)
- **قراردادهایِ API و OpenAPI**: [۳.۳.۳ — قراردادهایِ API و OpenAPI (`utoipa`)](../03-api-contracts-and-openapi/README.fa.md)
- **نسخه‌بندیِ API و تکاملش**: [۳.۳.۴ — نسخه‌بندیِ API و تکاملش](../04-api-versioning-and-evolution/README.fa.md)
- **پاکت‌هایِ خطایِ یکدست**: [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)
- **اکسترکتورِ `Json<T>` چطور یک خواندنِ ناموفق را `422` گزارش می‌کند**: [۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)

### می‌توانی توضیح بدهی؟

- کدام‌یک از `rename_all` و `default` و `skip_serializing_if` نوشتن را تغییر می‌دهد، کدام خواندن را، و کدام هر دو را؟
- چرا `{"title":"Frieren","ratng":9}` بدونِ `deny_unknown_fields` یک درخواستِ موفق است، و چرا این خطرناک است؟
- چهار نمایشِ enum در سیم چه شکلی‌اند، و کدام را برایِ یک فیدِ رویداد انتخاب می‌کنی؟
- چرا `untagged` گاهی بدونِ هیچ خطایی گونه‌ی اشتباه را برمی‌گرداند، و چطور جلویش را می‌گیری؟
- چرا خواندن به `Visitor` نیاز دارد ولی نوشتن چیزِ شبیهش را لازم نداشت؟
- کارِ یک سریالایزرِ DRF کجا تمام می‌شود و کارِ `serde` کجا شروع؟

---

## بیشتر

- [Serde attributes](https://serde.rs/attributes.html): فهرستِ کامل: attributeهایِ container، گونه و فیلد، با همه‌ی سبک‌هایِ `rename_all`.
- [Enum representations](https://serde.rs/enum-representations.html): چهار شکل کنارِ هم، با همان داده‌ی نمونه‌ی بالا.
- [Implementing `Deserialize`](https://serde.rs/impl-deserialize.html) و [Implementing `Serialize`](https://serde.rs/impl-serialize.html): راهنمایِ رسمیِ دو implِ دستی.
- [Custom serialization with `with`](https://serde.rs/field-attrs.html#with): attributeِ فیلدی که پشتِ چالش است.
- [مستنداتِ `serde_json`](https://docs.rs/serde_json/1.0.150/serde_json/): `from_str`، `to_string`، `Value`، و نوعِ خطایی که چاپ کردی.
- [۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md): نوع‌هایِ `CreateAnime` و `Anime` که این درس تکه‌تکه‌شان کرد.
