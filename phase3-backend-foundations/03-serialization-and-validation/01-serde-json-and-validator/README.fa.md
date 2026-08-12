# ۰۳.۱ — `serde_json` و `validator`

## دو کار مجزا که DRF یکجا انجام می‌دهد

DRF Serializer هم JSON را به object و برعکس تبدیل می‌کند و هم validation دارد. Rust عمداً این دو مسئولیت را در دو library نگه می‌دارد:

- `serde` و `serde_json` فقط **شکل** را می‌سنجند: fieldها و typeها درست‌اند؟ `rating: "nine"` به‌جای `9` همین‌جا رد می‌شود.
- `validator` **قانون** را روی `ReviewSubmission` درست‌ساخت می‌سنجد: آیا rating بین ۱ و ۱۰ است؟ title خالی نیست؟ type `u8` به‌تنهایی بازه‌ی ۱ تا ۱۰ را بیان نمی‌کند.

## derive کردن `Deserialize` و `Serialize`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewSubmission {
    pub title: String,
    pub rating: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
```

`#[derive(Deserialize)]` parsing استاندارد JSON را تولید می‌کند. `#[serde(default)]` نبودن کامل `comment` را به `None` تبدیل می‌کند؛ بدون آن نبودن key خطای deserialize بود. `skip_serializing_if` در جهت عکس، `None` را حذف می‌کند نه اینکه `"comment": null` بفرستد. انتخاب null یا key حذف‌شده تصمیم API واقعی است.

## derive کردن `Validate`

```rust
#[derive(Debug, Validate)]
pub struct ReviewSubmission {
    #[validate(length(min = 1, max = 200, message = "title must be 1-200 characters"))]
    pub title: String,
    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    pub rating: u8,
    #[validate(length(max = 1000, message = "comment must be at most 1000 characters"))]
    pub comment: Option<String>,
}
```

derive متد `.validate(&self) -> Result<(), ValidationErrors>` می‌سازد. `length` برای text/collection، `range` برای number و ruleهایی مانند `email`، `url`، `must_match` و `custom` نیز وجود دارند. برای `Option<String>`، validator rule را فقط در `Some` اجرا می‌کند و `None` را خودکار رد می‌کند؛ flag جدا لازم نیست.

`ValidationErrors` map ساخت‌یافته‌ی per-field است. `errors.field_errors()` یک `HashMap<&str, &Vec<ValidationError>>` می‌دهد، همتای strongly typed `serializer.errors` در DRF.

```senpai-visual
{"kind":"result","labels":["JSON خام","serde: شکل","ReviewSubmission","validator: قانون","ValidationErrors"]}
```

مثل تحویل فرم است: ابتدا نوع و خانه‌های لازم، سپس قواعد پذیرش. مرز تشبیه: input structurally خراب به validation مرحله دوم نمی‌رسد؛ هر دو دسته خطا یکجا دیده نمی‌شوند.

## تمرین تو

`validation_summary` را به `Vec<String>` مرتب از `"field: message"` تبدیل کن و `parse_review` را بنویس که InvalidJson و Invalid rule را جدا می‌کند.

## ایست بازرسی

`CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md` را بخوان.
