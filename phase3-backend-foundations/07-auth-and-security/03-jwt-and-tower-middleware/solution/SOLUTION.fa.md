# راه‌حل

```rust
pub fn issue_token(user_id: &str, secret: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after the Unix epoch")
        .as_secs();
    let claims = Claims {
        sub: user_id.to_string(),
        exp: (now + ONE_HOUR_IN_SECONDS) as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .expect("encoding a valid Claims struct should never fail")
}
```

`Header::default()`، `HS256` را برمی‌گزیند؛ همان secret هم امضا و هم verify می‌کند. `encode`، `Claims` را JSON می‌کند، header و payload را base64url می‌کند، روی `header.payload` امضای HMAC می‌زند و سه بخش را با نقطه وصل می‌کند.

```rust
pub async fn require_auth(
    State(secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthUser(data.claims.sub));

    Ok(next.run(request).await)
}
```

زنجیرهٔ `and_then` یعنی header را بگیر، UTF-8 معتبر بخواه، prefixِ `Bearer ` را جدا کن، و در هر شکست `401` برگردان. `decode::<Claims>` امضای HMAC را دوباره محاسبه و با token مقایسه می‌کند؛ `Validation::default()` هم `exp` را بررسی می‌کند. دلیلِ دورریختن علت دقیق شکست با `.map_err(|_| StatusCode::UNAUTHORIZED)` این است که برای caller همه‌شان یک راه‌حل دارند: دوباره authenticate شو؛ تفکیک بیش‌ازحد به مهاجم اطلاعات می‌دهد.

بعد از موفقیت، `AuthUser(data.claims.sub)` در extension قرار می‌گیرد. تفاوتش با token ثابتِ `require_bearer_token` همین‌جاست: آن فقط می‌گوید caller shared secret را داشته؛ JWT هویتِ تک‌کاربر (`sub`) و امکان claimهای role/scope و expiry را به handler می‌رساند.

JWT بدون `exp` تا ابد معتبر می‌ماند و token لو‌رفته دسترسی دائمی می‌دهد؛ تغییر signing secret هم همهٔ کاربران را یکجا خارج می‌کند. برای revoke زودتر می‌توان `jti` یا user id را با TTL در Redis/database denylist کرد و در هر request lookup داشت؛ دقیقاً همان هزینهٔ state که JWT خالص قرار بود حذف کند. تست `app().oneshot(...)` هم تضمین می‌کند middleware واقعاً route را کوتاه‌مدار می‌کند و request ردشده به handler نمی‌رسد.
