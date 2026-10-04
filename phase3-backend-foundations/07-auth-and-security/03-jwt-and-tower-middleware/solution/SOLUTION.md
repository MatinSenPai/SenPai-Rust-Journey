# Solution — 3.7.3 JWTs and `tower` middleware

The whole crate is in `src/lib.rs`, and `cargo test` in this directory runs all 23 tests against it. Below, each function of the ladder and the one decision in it that is easy to get wrong.

## `bearer_token`

```rust
pub fn bearer_token(header_value: &str) -> Option<&str> {
    let (scheme, token) = header_value.split_once(' ')?;
    let plain = !token.is_empty() && !token.contains(char::is_whitespace);
    (scheme.eq_ignore_ascii_case("Bearer") && plain).then_some(token)
}
```

`split_once(' ')` splits at the first space only, so `"Bearer  abc"` (two spaces) leaves `" abc"` as the token, and the whitespace check rejects it. `"Bearer"` has no space at all, so the `?` returns `None`. The scheme is compared without regard to case, as RFC 7235 says, but the token is returned untouched: it is case-sensitive.

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

`now` is read once, so `iat` and `exp` agree. `(config.clock)()` needs the parentheses: `config.clock()` would look for a method named `clock`. The algorithm is named explicitly instead of relying on `Header::default()`, so the algorithm of the tokens you issue is visible in the code that pins it for verification.

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

Three things matter here. The library's own `exp` check is off (`validate_exp = false`), because it reads the real clock. The expiry comparison comes after `decode`, which is why a bad signature is reported even for an expired token: the signature is checked first. And `Validation::new(Algorithm::HS256)` is the algorithm pin: `algorithms` is the one-element list, so an HS512 token is `InvalidAlgorithm` before its signature is looked at.

```rust
if (config.clock)() > claims.exp.saturating_add(config.leeway_secs) {
    return Err(AuthError::Expired);
}
Ok(claims)
```

`>` and not `>=`: at exactly `exp + leeway_secs` the token is still valid. `saturating_add` keeps a huge `exp` near `u64::MAX` from overflowing and panicking in a debug build. The leeway comes from `config`, not from the library's default of 60 seconds.

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

Each `and_then` is one way for the header to be unusable: absent, not valid text, not a Bearer value. All of them are `Missing`, because the client has not presented credentials. `verify_token(...)?` passes its `AuthError` straight out, and the `IntoResponse` impl turns it into the `401`. `token` borrows from `request`, and the borrow ends at `verify_token` because `claims` owns its data, so `request` can be mutated afterwards.

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

The order is the whole exercise. The first `route_layer` wraps only `/admin`. The second wraps both routes, and because it was added last it is outermost: `require_auth` runs first, so `Extension<AuthUser>` exists by the time `require_subject` asks for it, and a request with no token never reaches the `403` check. Swap the two `route_layer` calls and `/admin` answers `500` with "Missing request extension", the failure from `examples/04-forgot-the-layer.rs`.

## Running it

```sh
cargo test
cargo fmt --check
```

All 23 tests pass, with a fixed clock throughout. None of them sleeps or reads the system time.
