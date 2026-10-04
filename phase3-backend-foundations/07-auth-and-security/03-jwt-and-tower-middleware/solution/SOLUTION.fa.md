# راه‌حل — ۳.۷.۳ JWT و میان‌افزار در `tower`

کلِ کریت در `src/lib.rs` است، و `cargo test` در همین پوشه هر ۲۳ تست را رویِ آن اجرا می‌کند. در ادامه، هر تابعِ نردبان و تصمیمی که در آن راحت غلط درمی‌آید.

## `bearer_token`

```rust
pub fn bearer_token(header_value: &str) -> Option<&str> {
    let (scheme, token) = header_value.split_once(' ')?;
    let plain = !token.is_empty() && !token.contains(char::is_whitespace);
    (scheme.eq_ignore_ascii_case("Bearer") && plain).then_some(token)
}
```

`split_once(' ')` فقط در اولین فاصله می‌شکند، پس `"Bearer  abc"` (دو فاصله) `" abc"` را به‌عنوانِ توکن می‌گذارد و بررسیِ فاصله ردش می‌کند. `"Bearer"` اصلاً فاصله ندارد، پس `?` همان‌جا `None` برمی‌گرداند. طرح بدونِ توجه به حروفِ بزرگ و کوچک مقایسه می‌شود، همان‌طور که RFC 7235 می‌گوید، ولی توکن دست‌نخورده برمی‌گردد: به حروف حساس است.

## `issue_token`

```rust
pub fn issue_token(config: &JwtConfig, user_id: &str) -> String {
    let now = (config.clock)();
    let claims = Claims { sub: user_id.to_string(), iat: now, exp: now + config.ttl_secs };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(config.secret.as_bytes()),
    )
    .expect("HS256 signing with a byte secret cannot fail")
}
```

`now` فقط یک بار خوانده می‌شود، پس `iat` و `exp` با هم می‌خوانند. `(config.clock)()` پرانتز لازم دارد: `config.clock()` دنبالِ متدی به نامِ `clock` می‌گردد. الگوریتم صریح نوشته شده، نه با تکیه بر `Header::default()`، تا الگوریتمِ توکن‌هایی که صادر می‌کنی در همان کدی دیده شود که برایِ راستی‌آزمایی قفلش می‌کند.

## `verify_token`

```rust
let mut validation = Validation::new(Algorithm::HS256);
validation.validate_exp = false;
validation.set_required_spec_claims(&["exp", "sub"]);

let key = DecodingKey::from_secret(config.secret.as_bytes());
let claims = decode::<Claims>(token, &key, &validation)
    .map_err(|error| match error.kind() {
        ErrorKind::InvalidAlgorithm => AuthError::WrongAlgorithm,
        ErrorKind::InvalidSignature => AuthError::BadSignature,
        _ => AuthError::Malformed,
    })?
    .claims;
```

سه چیز اینجا مهم است. بررسیِ `exp`ِ خودِ کتابخانه خاموش است (`validate_exp = false`)، چون ساعتِ واقعی را می‌خواند. مقایسه‌یِ انقضا بعد از `decode` می‌آید، و برایِ همین امضایِ بد حتی برایِ توکنِ منقضی گزارش می‌شود: امضا اول بررسی می‌شود. و `Validation::new(Algorithm::HS256)` همان قفلِ الگوریتم است: `algorithms` فهرستِ تک‌عضوی است، پس یک توکنِ HS512 پیش از دیده‌شدنِ امضایش `InvalidAlgorithm` می‌گیرد.

```rust
if (config.clock)() > claims.exp.saturating_add(config.leeway_secs) {
    return Err(AuthError::Expired);
}
Ok(claims)
```

`>` و نه `>=`: دقیقاً در `exp + leeway_secs` توکن هنوز معتبر است. `saturating_add` جلویِ سرریز و پنیکِ یک `exp`ِ عظیم نزدیک به `u64::MAX` را در بیلدِ دیباگ می‌گیرد. مهلتِ تحمل از `config` می‌آید، نه از پیش‌فرضِ ۶۰ثانیه‌ایِ کتابخانه.

## `require_auth`

```rust
pub async fn require_auth(
    State(config): State<JwtConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(bearer_token)
        .ok_or(AuthError::Missing)?;
    let claims = verify_token(&config, token)?;
    request.extensions_mut().insert(AuthUser(claims.sub));
    Ok(next.run(request).await)
}
```

هر `and_then` یک راهِ ناقابل‌استفاده‌بودنِ هدر است: نبودن، متنِ نامعتبر، مقدارِ غیرِ Bearer. همه `Missing` هستند، چون کلاینت اعتباری ارائه نکرده. `verify_token(...)?` `AuthError`ِ خودش را مستقیم بیرون می‌دهد، و پیاده‌سازیِ `IntoResponse` آن را به `401` تبدیل می‌کند. `token` از `request` قرض می‌گیرد، و قرض در `verify_token` تمام می‌شود چون `claims` داده‌اش را مالک است، پس بعدش می‌شود `request` را تغییر داد.

## `admin_app`

```rust
async fn require_subject(
    State(admin): State<Arc<str>>,
    Extension(user): Extension<AuthUser>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if user.0 == *admin { Ok(next.run(request).await) } else { Err(StatusCode::FORBIDDEN) }
}
```

```rust
Router::new()
    .route("/admin", get(admin_ping))
    .route_layer(from_fn_with_state(Arc::<str>::from(admin), require_subject))
    .route("/whoami", get(whoami))
    .route_layer(from_fn_with_state(config, require_auth))
```

ترتیب کلِ تمرین است. اولین `route_layer` فقط `/admin` را می‌پیچد. دومی هر دو مسیر را می‌پیچد، و چون آخر اضافه شده بیرونی‌ترین است: `require_auth` اول اجرا می‌شود، پس وقتی `require_subject` `Extension<AuthUser>` را می‌خواهد وجود دارد، و درخواستِ بدونِ توکن هیچ‌وقت به بررسیِ `403` نمی‌رسد. جای دو `route_layer` را عوض کنی `/admin` با «Missing request extension» `500` می‌دهد، همان شکستِ `examples/04-forgot-the-layer.rs`.

## اجرا

```sh
cargo test
cargo fmt --check
```

هر ۲۳ تست پاس می‌شوند، همه با ساعتِ ثابت. هیچ‌کدام نمی‌خوابد یا زمانِ سیستم را نمی‌خواند.
