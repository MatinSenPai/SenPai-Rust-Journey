# راه‌حل — ۳.۸.۲ ردیابیِ درخواست و correlation ID

## `is_valid_request_id`

```rust
pub fn is_valid_request_id(candidate: &str) -> bool {
    (1..=MAX_REQUEST_ID_LEN).contains(&candidate.len())
        && candidate
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}
```

دو شرط با `&&` به‌هم وصل شده‌اند. بررسیِ طول از `str::len` استفاده می‌کند که بایت می‌شمارد، و مشخصات هم بایت گفته، پس `"café"` طولش ۵ است و در بررسیِ نویسه هم رد می‌شود، چون `é` ASCII نیست. `is_ascii_alphanumeric` آزمونِ فقط‌ASCII است: `char::is_alphanumeric` حرف‌هایِ `é` و فارسی را هم می‌پذیرفت، و آن‌وقت فراخواننده می‌توانست یونیکد در لاگت بگذارد. رشته‌یِ خالی در بررسیِ طول رد می‌شود، پس حالتِ جداگانه لازم نیست. تست‌هایِ `length_limit_is_64_bytes_inclusive` و `rejects_characters_outside_the_allowed_set` هر دو لبه را قفل می‌کنند.

## `resolve_request_id`

```rust
pub fn resolve_request_id(incoming: Option<&str>, generate: impl FnOnce() -> String) -> String {
    match incoming {
        Some(id) if is_valid_request_id(id) => id.to_string(),
        _ => generate(),
    }
}
```

شرطِ match کلِ کار را می‌کند. شاخه‌یِ `_` هم هدرِ نبود و هم هدرِ نامعتبر را می‌گیرد، که از دیدِ صداکننده‌یِ این تابع یک حالت‌اند: یکی تازه بساز. `generate` از نوعِ `FnOnce` است و فقط در شاخه‌یِ دوم آمده، پس یک شناسه‌یِ معتبر هیچ‌وقت آن را صدا نمی‌زند (تست یک closure می‌دهد که پنیک می‌کند تا ثابت کند). هیچ‌چیز در اینجا ورودی را نمی‌برد و ویرایش نمی‌کند: یک شناسه‌یِ نامعتبر کامل عوض می‌شود.

## `error_body`

```rust
pub fn error_body(request_id: &str, code: &str, message: &str) -> Value {
    json!({"error": {"code": code, "message": message, "request_id": request_id}})
}
```

`serde_json::json!` یک `Value` می‌سازد و رشته‌ها را برایت escape می‌کند، پس پیامی که گیومه یا خطِ جدید دارد JSON معتبر می‌ماند. ساختنِ متن با دست و `format!` این را نمی‌کرد.

## `request_id_middleware`

```rust
let incoming = request
    .headers()
    .get(REQUEST_ID_HEADER)
    .and_then(|value| value.to_str().ok());
let id = resolve_request_id(incoming, new_request_id);
request.extensions_mut().insert(RequestId(id.clone()));
```

`HeaderValue::to_str` برایِ بایت‌هایِ بیرونِ ASCIIِ قابل‌دیدن شکست می‌خورد، و `.ok()` آن شکست را به `None` تبدیل می‌کند، پس «متن نیست» و «نیست» هر دو به ساختنِ شناسه می‌رسند، همان‌طور که مشخصات می‌گوید. `new_request_id` به‌شکلِ تابع داده شده، نه صدازده‌شده، پس فقط وقتی اجرا می‌شود که `resolve_request_id` به آن نیاز داشته باشد.

```rust
let span = tracing::info_span!(
    "request",
    request_id = %id,
    method = %request.method(),
    path = %request.uri().path(),
);
async move { /* ... */ }.instrument(span).await
```

`%` هر فیلد را با `Display` ثبت می‌کند، پس شناسه بدونِ گیومه چاپ می‌شود (`request_id=abc-123`)، که همان چیزی است که تست‌ها دنبالش‌اند. `uri().path()` مسیر بدونِ query string است: تستِ `the_path_in_the_span_has_no_query_string` عبارتِ `?secret=hunter2` را می‌فرستد و بررسی می‌کند هیچ‌وقت به لاگ نمی‌رسد. بازه از `request` ساخته می‌شود پیش از آنکه `next.run(request)` آن را منتقل کند. هرچه بعد از آن است داخلِ یک بلوکِ `async move` است که با `.instrument(span)` پیچیده شده. کلِ داستانِ بازه همین است: بازه به future چسبیده است، پس در هر `.await` می‌ماند، و یک نگهبانِ `enter()` نیست، که اینجا کامپایل هم نمی‌شد (`E0277`ِ «خطاهایی که خواهی دید» در خودِ درس).

```rust
if let Some(info) = response.extensions_mut().remove::<ErrorInfo>() {
    if response.status().is_server_error() {
        tracing::error!(code = %info.code, "request failed");
    } else {
        tracing::warn!(code = %info.code, "request failed");
    }
    let (mut parts, _old_body) = response.into_parts();
    parts.headers.remove(CONTENT_LENGTH);
    parts.headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let body = error_body(&id, &info.code, &info.message).to_string();
    response = Response::from_parts(parts, Body::from(body));
}
```

`ApiError::into_response` یک `ErrorInfo` در extensionهایِ پاسخ جا می‌گذارد، و `remove` آن را بیرون می‌کشد تا جلوتر نرود. فقط پاسخی که یکی دارد بازنویسی می‌شود: یک `200` و ۴۰۴ِ خودِ روتر برایِ مسیرِ ناشناخته همان‌طور که اپ ساخته برمی‌گردند، به‌علاوه‌یِ هدر (`a_route_that_does_not_exist_still_gets_an_id`). کدِ وضعیت می‌ماند چون `into_parts` آن را نگه می‌دارد. `content-length` باید برود: بدنه‌یِ قدیمی ۶۱ بایت بود، بدنه‌یِ تازه بلندتر است، و یک `content-length`ِ کهنه دیگر با بدنه جور نیست. با حذفش، طول از `Body`ِ تازه می‌آید. سطحِ لاگ از دسته‌یِ کدِ وضعیت پیروی می‌کند: ۴xx اشتباهِ فراخواننده است و `WARN`، ۵xx مالِ ماست و `ERROR` (`failures_are_logged_with_their_code_at_the_right_level`).

```rust
tracing::info!(status = response.status().as_u16(), "finished");
response.headers_mut().insert(REQUEST_ID_HEADER, HeaderValue::from_str(&id).unwrap());
```

`status` به‌شکلِ عدد ثبت می‌شود، پس خط `status=200` می‌خواند. هدر آخر گذاشته می‌شود، با `insert`، که هر `x-request-id`ای را که هندلر گذاشته باشد جایگزین می‌کند. `unwrap` نمی‌تواند شکست بخورد: شناسه یا رشته‌یِ ASCIIِ معتبری است که قبلاً مقدارِ هدر بوده، یا یک UUID.

## خروجیِ تست‌ها

```text
running 12 tests
test an_oversized_id_is_replaced ... ok
test a_hostile_id_never_reaches_the_log ... ok
test a_malformed_id_is_replaced ... ok
test a_missing_id_is_generated_and_echoed ... ok
test a_valid_incoming_id_is_kept_and_echoed ... ok
test a_generated_id_is_the_one_in_the_log ... ok
test an_error_body_carries_the_id_and_keeps_the_status ... ok
test handler_log_lines_carry_the_id_through_the_span ... ok
test the_path_in_the_span_has_no_query_string ... ok
test failures_are_logged_with_their_code_at_the_right_level ... ok
test a_route_that_does_not_exist_still_gets_an_id ... ok
test a_successful_response_body_is_untouched ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

```text
running 8 tests
test accepts_ordinary_ids ... ok
test error_body_has_the_documented_shape ... ok
test generated_ids_are_uuids_and_unique ... ok
test valid_incoming_id_is_kept_and_generate_is_not_called ... ok
test length_limit_is_64_bytes_inclusive ... ok
test missing_incoming_id_is_generated ... ok
test rejects_characters_outside_the_allowed_set ... ok
test invalid_incoming_id_is_replaced_not_repaired ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(ترتیبِ خطوط و زمان‌ها در اجراهایِ مختلف فرق می‌کند: `cargo test` تست‌ها را موازی اجرا می‌کند.)

## چالش (اختیاری)

هیچ تستی ندارد، و یک چینشی که کار می‌کند:

```rust
ServiceBuilder::new()
    .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
    .layer(from_fn(replace_invalid_id))
    .layer(trace)
    .layer(PropagateRequestIdLayer::x_request_id())
```

`replace_invalid_id` هدرِ `x-request-id` را از درخواست می‌خواند، و اگر `is_valid_request_id` نه بگوید، هدر را با `new_request_id()` بازنویسی می‌کند. بعد از `SetRequestIdLayer` می‌نشیند (که اگر هدر نبود آن را پر کرده) و پیش از `TraceLayer` (تا بازه مقدارِ اصلاح‌شده را بخواند). لایه‌هایِ آماده ساختن، بازه با تأخیر، و پژواک را بر عهده گرفتند. چیزی که نتوانستی واگذار کنی: قضاوت درباره‌یِ اینکه شناسه‌یِ ورودی قابلِ‌قبول است یا نه، `RequestId`ِ تایپ‌دار در extensionها، و شناسه در بدنه‌یِ خطا. ترتیب برایِ پژواک مهم است. اگر `PropagateRequestIdLayer` *پیش از* `replace_invalid_id` فهرست شود (یعنی بیرونِ آن)، پاسخ `"not valid!"` را پژواک داد در حالی که لاگ UUIDِ تازه را نشان می‌داد: لایه هدرِ درخواست را وقتی می‌خواند که درخواست به آن می‌رسد، پیش از جایگزینی. اگر بعد از آن فهرست شود، مثلِ بالا، هدرِ پژواک‌شده و لاگ یک شناسه دارند.
