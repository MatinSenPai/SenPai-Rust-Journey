# راه‌حل — ۳.۳.۳ قراردادهایِ API و OpenAPI (`utoipa`)

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/tests/` را می‌گذراند، از جمله `build_test.rs` برایِ پله‌ی «بساز».

## چهار خواننده

هر چهار رویِ سند به‌شکلِ یک `serde_json::Value` ساده کار می‌کنند، و هر چهار به یک واقعیت تکیه دارند: ایندکس‌کردنِ یک `Value` با کلیدِ غایب به‌جایِ panic مقدارِ `Value::Null` می‌دهد، پس مسیر، متد یا کدِ وضعیتِ غایب به «هیچ» می‌رسد.

```rust
pub fn schema_names(doc: &Value) -> Vec<String> {
    let mut names: Vec<String> = match doc["components"]["schemas"].as_object() {
        Some(schemas) => schemas.keys().cloned().collect(),
        None => Vec::new(),
    };
    names.sort();
    names
}
```

`as_object()` برایِ `Null` مقدارِ `None` است، که «`components` نیست» و «`schemas` نیست» را در یک شاخه می‌پوشاند. مرتب‌سازی صریح است چون `serde_json::Value` کلیدهایِ شیء را فقط به‌طورِ پیش‌فرض در یک `BTreeMap` نگه می‌دارد؛ این‌جا مرتب‌سازی به آن تکیه نمی‌کند.

```rust
pub fn operation_statuses(doc: &Value, method: &str, path: &str) -> Vec<String> {
    let responses = &doc["paths"][path][method.to_lowercase()]["responses"];
    /* same keys-then-sort as above */
}
```

OpenAPI متدها را با حروفِ کوچک می‌نویسد (`"get"`)، پس `method.to_lowercase()` باعث می‌شود `"GET"`، `"Get"` و `"get"` یک جستجو باشند. `response_content_types` همان پیمایش است، یک سطح عمیق‌تر (`["responses"][status]["content"]`)؛ یک `204` `content` ندارد، پس به `Null` می‌رسد و یک `Vec`ِ خالی می‌دهد.

```rust
pub fn undocumented(doc: &Value, routes: &[(&str, &str)]) -> Vec<String> {
    routes.iter()
        .filter(|(method, path)| doc["paths"][*path][method.to_lowercase()].is_null())
        .map(|(method, path)| format!("{method} {path}"))
        .collect()
}
```

`filter` مسیرهایِ بدونِ عملیات را نگه می‌دارد و `map` آن‌ها را با املایِ خودِ فراخواننده از متد قالب می‌دهد (`"Post /a"`، نه `"post /a"`). پیمایشِ `routes` به ترتیب است که خروجی را به همان ترتیبِ داده‌شده نگه می‌دارد.

## املایِ `content(...)`

در `utoipa` 5 پاسخِ چندنوعی `content((ApiError = "application/json"), (String = "text/plain"))` است: اول اسکیما، بعد نوعِ رسانه‌ای پس از `=`. ترتیبِ `utoipa` 4 (`("application/json" = ApiError)`) یک خطایِ پارسِ ماکرو است، `expected ,`. شکلِ تک‌نوعی `body = String, content_type = "text/plain"` هم معتبر است و برایِ `400` و `415` به کار رفته.

## پله‌ی «بساز»

```rust
#[utoipa::path(put, path = "/anime/{id}", request_body = CreateAnime,
    params(("id" = u64, Path, description = "the anime's id")),
    responses((status = 200, body = Anime), (status = 404, body = ApiError),
              (status = 422, content((ApiError = "application/json"), (String = "text/plain")))))]
pub async fn replace_anime(/* State, Path(id), Json(input) */) -> ApiResult<Json<Anime>>
```

چهار ویرایش یک endpointِ جدید را کامل می‌کند: هندلر با attributeاش، `paths(..., replace_anime)`، `.put(replace_anime)` رویِ مسیرِ `/anime/{id}`، و `("PUT", "/anime/{id}")` در `ROUTES`. اگر سومی را فراموش کنی مسیر `405` جواب می‌دهد؛ اگر اولی یا دومی را فراموش کنی، `undocumented` مسیر را فهرست می‌کند؛ اگر چهارمی را فراموش کنی، هیچ‌چیز در خواننده‌ها متوجه نمی‌شود، چون `ROUTES` همان فهرستی است که به آن اعتماد می‌کنند (`build_test.rs` آن ورودی را مستقیم بررسی می‌کند). رتبه پیش از شناسه بررسی می‌شود، مثلِ `POST`، و `replace_anime` سه فیلد را در جا بازنویسی می‌کند، پس شناسه هرگز عوض نمی‌شود و یک `PUT`ِ مشابهِ تکراری همان پاسخ را می‌دهد (`put_replaces_the_whole_anime_and_keeps_the_id` آن را دو بار می‌فرستد).
