# راه‌حل — ۳.۲.۲ نوشتنِ اکسترکتورِ خودت

## تعمیر

۱. `05`: `type Rejection = (StatusCode, &'static str);`. تاپلِ کدِ وضعیت و متن از قبل `IntoResponse` دارد؛ enum نداشت. (`Missing` و `Unreadable` را به یک کد و پیام نگاشت کن.)
۲. `06`: `Result<UserAgent, (StatusCode, &'static str)>` بگیر و رویش `match` کن، مثلِ `examples/04-optional-with-result.rs`. `Option<UserAgent>` به `OptionalFromRequestParts` نیاز داشت.
۳. `07`: `impl<S: Send + Sync> FromRequestParts<S> for UserAgent`.
۴. `08`: `async fn upload(method: Method, body: String)`. اکسترکتورِ بدنه آخر می‌رود.

## `ApiKey`

```rust
let key = parts
    .headers
    .get("x-api-key")
    .and_then(|value| value.to_str().ok())
    .map(str::trim)
    .filter(|key| !key.is_empty())
    .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
Ok(ApiKey(key.to_string()))
```

سه شکست (نبودن، متن نبودن، خالی‌بودن) به یک `None` می‌رسند، پس یک `ok_or` به هر سه همان `401` را می‌دهد. `?` ردِ درخواست را برمی‌گرداند و `axum` آن را به پاسخ تبدیل می‌کند.

## `Pagination`

```rust
let Query(raw) = Query::<RawPagination>::from_request_parts(parts, state)
    .await
    .map_err(|_| (StatusCode::BAD_REQUEST, "page and per_page must be whole numbers"))?;
let page = raw.page.unwrap_or(1);
let per_page = raw.per_page.unwrap_or(DEFAULT_PER_PAGE);
if page == 0 || per_page == 0 { /* 400, at least 1 */ }
Ok(Pagination { page, per_page: per_page.min(MAX_PER_PAGE) })
```

فیلدهایِ `RawPagination` از نوعِ `Option<u32>` هستند، پس کلیدِ غایب `None` است و خودِ `Query` مقدارهایِ `abc`، `-1` (که `u32` نیست) و کلیدِ تکراری را رد می‌کند. صفر یک `u32`ِ معتبر است، پس اکسترکتور خودش بررسی‌اش می‌کند. سقف با `min` اعمال می‌شود، نه با خطا.

## `ClientVersion`

هدرِ غایب اولین `400` را می‌دهد. بقیه‌چیز از یک ردِ `bad` رد می‌شود: `to_str`، بعد `split_once('.')`، بعد `parse::<u32>()` رویِ هر نیمه. `1` نقطه ندارد؛ `1.2.3` نیمه‌ی `2.3` می‌ماند که `u32` نیست؛ `1.`، `.4`، `1.-2` و رشته‌ی خالی پارس نمی‌شوند. هر دو نیمه باید پارس شوند، پس «دقیقاً دو عددِ صحیح» از سه قدمِ کوچک بیرون می‌آید.

## چالش

```rust
impl<S> FromRequestParts<S> for Authorized
where
    S: Send + Sync,
    KeyStore: FromRef<S>,
{
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ApiKey(key) = ApiKey::from_request_parts(parts, state).await?;
        let store = KeyStore::from_ref(state);
        /* 403 when the key is not in the set */
    }
}
```

`ApiKey` خودش کلیدِ غایب را با `401` رد می‌کند و `?` آن را دست‌نخورده رد می‌کند. `KeyStore: FromRef<S>` یعنی state فقط باید بتواند یک `KeyStore` تولید کند؛ روتری که stateاش دقیقاً یک `KeyStore` است هم کار می‌کند، چون `FromRef<T> for T` برایِ نوع‌هایِ `Clone` وجود دارد. نسخه‌ی کامل در `src/lib.rs` زیرِ `challenge` است.
