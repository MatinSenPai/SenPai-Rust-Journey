# ۰۶.۲ — JWT و middleware در `tower`

درس قبل password را هش کرد تا login بپرسد «واقعاً خودت هستی؟» حالا سؤال بعدی این است: پس از یک‌بار اثبات هویت، هر request بعدی چگونه بدون فرستادن password و بدون نگه‌داشتن همهٔ loginها در حافظهٔ سرور، هویت خود را ثابت کند؟ پاسخ این درس **JWT (JSON Web Token)** است: در login صادر و در middlewareِ `axum` برای هر route محافظت‌شده تأیید می‌شود.

```senpai-visual
{"kind":"network","labels":["login","JWT امضاشده","middleware","handler محافظت‌شده"]}
```

## JWT واقعاً چیست؟

JWT سه قطعهٔ base64url است که با نقطه وصل شده‌اند:

```
eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJ1c2VyLTQyIiwiZXhwIjoxNzUyMDgwMDAwfQ.dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
└──────── header ────────┘└─────────────── payload ───────────────┘└──────────── signature ────────────┘
```

- `header` الگوریتم و نوع token است؛ مثلاً `{"alg":"HS256","typ":"JWT"}`.
- `payload` شامل claimهاست: در این درس `sub` برای هویت و `exp` برای زمان انقضا.
- `signature` برای `HS256` برابر `HMAC-SHA256(header + "." + payload, secret)` است.

نکتهٔ حیاتی: JWT **امضاشده است، encrypt نشده**. base64url فقط encoding است و هرکس token را دارد header و payload را می‌خواند؛ secret فقط برای تأیید signature لازم است. password، secret یا هر داده‌ای که حاضر نیستی در query string ببینی هرگز در claim نگذار. user id، role و expiration اشکالی ندارند.

## JWT در برابر session سمت سرور

session سنتی (مثل cookieِ `sessionid` در Django) state را در database یا cache نگه می‌دارد و در هر request lookup می‌کند. JWT state را در خود token حمل می‌کند و با امضا قابل‌اعتماد می‌شود، بی‌آنکه lookup لازم باشد. بهایش: session را با حذف رکورد فوراً revoke می‌کنی؛ JWT تا `exp` معتبر است مگر denylist یا سازوکار revoke جداگانه بسازی. بنابراین عمر JWT کوتاه است و در سیستم واقعی معمولاً refresh token جدا دارد.

## صدور و بررسی token در middleware

دو `todo!()` همین دو نیمه‌اند:

```rust
pub fn issue_token(user_id: &str, secret: &str) -> String {
    // build Claims { sub: user_id, exp: one hour from now }
    // encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
}

pub async fn require_auth(
    State(secret): State<String>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // pull "Authorization: Bearer <token>" off the request
    // decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
    // Ok => stash the user id in request.extensions_mut(), then next.run(request).await
    // Err => StatusCode::UNAUTHORIZED
}
```

`require_auth` با `axum::middleware::from_fn_with_state` سازگار است و آن را کنار `capstone-taskforge/taskforge-api/src/auth.rs` ببین. هر دو header را می‌گیرند، روی خطا `401` می‌دهند و روی موفقیت `next.run(request).await` را صدا می‌زنند. تفاوت این است که `require_bearer_token` یک shared secret ثابت را مقایسه می‌کند، اما این درس JWT امضاشده، کاربرمحور و منقضی‌شونده را بررسی می‌کند.

## middleware و request extensions

```rust
pub fn app(secret: String) -> Router {
    Router::new()
        .route("/whoami", get(whoami))
        .route_layer(from_fn_with_state(secret, require_auth))
}
```

`route_layer` routeهای پیش از خودش را با middleware می‌پوشاند. request نامعتبر اصلاً به `whoami` نمی‌رسد. request معتبر یک `AuthUser` در `request.extensions_mut()` می‌گیرد؛ handler پایین‌دست آن را با `Extension<AuthUser>` بیرون می‌کشد. پس handler نه header را دوباره parse می‌کند و نه signature را دوباره می‌سنجد.

در `src/lib.rs` `issue_token` را با claim یک‌ساعته و `require_auth` را با استخراج `Bearer`, `decode`, درج `AuthUser` و `401` برای همهٔ خطاها کامل کن. تست‌ها در `tests/jwt_test.rs` از سطح publicِ `app` استفاده می‌کنند:

```sh
cargo test -p p3-06-02-jwt-and-tower-middleware
```

## قدم بعدی

بعد از تست، `solution/SOLUTION.fa.md` را بخوان.
