//! Tests for 3.7.3. No test reads the real clock: every config gets a clock
//! that returns a fixed number. Middleware is driven with `oneshot`, as in
//! 3.2.1.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::Router;
use jsonwebtoken::{decode_header, encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use tower::ServiceExt;

use p3_07_03_jwt_and_tower_middleware::{
    admin_app, app, bearer_token, issue_token, verify_token, AuthError, Claims, JwtConfig,
};

const SECRET: &str = "test-secret-for-the-3-7-3-lesson";
const NOW: u64 = 1_700_000_000;

fn config_at(now: u64) -> JwtConfig {
    JwtConfig::new(SECRET)
        .with_ttl(3600)
        .with_leeway(30)
        .with_clock(Arc::new(move || now))
}

/// Base64url without padding, enough to build tokens by hand.
fn b64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |acc, (i, b)| acc | (*b as u32) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

// ------------------------------------------------------------- bearer_token

#[test]
fn bearer_token_accepts_the_scheme_in_any_case() {
    assert_eq!(bearer_token("Bearer abc.def.ghi"), Some("abc.def.ghi"));
    assert_eq!(bearer_token("bearer abc"), Some("abc"));
    assert_eq!(bearer_token("BEARER abc"), Some("abc"));
}

#[test]
fn bearer_token_rejects_everything_else() {
    for value in [
        "Bearer",
        "Bearer ",
        "Basic abc",
        "abc",
        "Bearer  abc",
        "Bearer a b",
        "Bearer abc ",
        "",
    ] {
        assert_eq!(bearer_token(value), None, "{value:?} should be rejected");
    }
}

// -------------------------------------------------------------- issue_token

#[test]
fn an_issued_token_has_the_documented_claims() {
    let config = config_at(NOW);
    let token = issue_token(&config, "user-42");

    assert_eq!(token.split('.').count(), 3);
    assert_eq!(decode_header(&token).unwrap().alg, Algorithm::HS256);
    assert_eq!(
        verify_token(&config, &token).unwrap(),
        Claims {
            sub: "user-42".to_string(),
            iat: NOW,
            exp: NOW + 3600,
        }
    );
}

#[test]
fn the_ttl_comes_from_the_config() {
    let config = config_at(NOW).with_ttl(60);
    let claims = verify_token(&config, &issue_token(&config, "u")).unwrap();
    assert_eq!(claims.exp, NOW + 60);
}

// ------------------------------------------------------------- verify_token

#[test]
fn a_token_is_valid_up_to_exp_plus_leeway_inclusive() {
    let token = issue_token(&config_at(NOW), "user-1");
    let last_good_second = NOW + 3600 + 30;

    assert!(verify_token(&config_at(last_good_second), &token).is_ok());
    assert_eq!(
        verify_token(&config_at(last_good_second + 1), &token),
        Err(AuthError::Expired)
    );
}

#[test]
fn leeway_of_zero_means_expired_one_second_after_exp() {
    let strict = config_at(NOW + 3601).with_leeway(0);
    let token = issue_token(&config_at(NOW), "user-1");
    assert_eq!(verify_token(&strict, &token), Err(AuthError::Expired));
    assert!(verify_token(&config_at(NOW + 3600).with_leeway(0), &token).is_ok());
}

#[test]
fn a_different_secret_is_a_bad_signature() {
    let mut other = config_at(NOW);
    other.secret = "another-secret-entirely".to_string();
    let token = issue_token(&other, "user-1");

    assert_eq!(
        verify_token(&config_at(NOW), &token),
        Err(AuthError::BadSignature)
    );
}

#[test]
fn editing_the_payload_breaks_the_signature() {
    let token = issue_token(&config_at(NOW), "user-1");
    let parts: Vec<&str> = token.split('.').collect();
    let forged = json!({"sub": "admin", "iat": NOW, "exp": NOW + 3600}).to_string();
    let tampered = format!("{}.{}.{}", parts[0], b64(forged.as_bytes()), parts[2]);

    assert_eq!(
        verify_token(&config_at(NOW), &tampered),
        Err(AuthError::BadSignature)
    );
}

#[test]
fn a_bad_signature_wins_over_expiry() {
    let mut other = config_at(NOW);
    other.secret = "another-secret-entirely".to_string();
    let token = issue_token(&other, "user-1");

    assert_eq!(
        verify_token(&config_at(NOW + 1_000_000), &token),
        Err(AuthError::BadSignature)
    );
}

#[test]
fn garbage_is_malformed() {
    for token in ["", "not-a-jwt", "a.b", "a.b.c", "....."] {
        assert_eq!(
            verify_token(&config_at(NOW), token),
            Err(AuthError::Malformed),
            "{token:?}"
        );
    }
}

#[test]
fn a_token_without_exp_is_malformed() {
    let key = EncodingKey::from_secret(SECRET.as_bytes());
    let token = encode(&Header::default(), &json!({"sub": "u", "iat": NOW}), &key).unwrap();

    assert_eq!(
        verify_token(&config_at(NOW), &token),
        Err(AuthError::Malformed)
    );
}

#[test]
fn a_token_without_sub_is_malformed() {
    let key = EncodingKey::from_secret(SECRET.as_bytes());
    let claims = json!({"iat": NOW, "exp": NOW + 60});
    let token = encode(&Header::default(), &claims, &key).unwrap();

    assert_eq!(
        verify_token(&config_at(NOW), &token),
        Err(AuthError::Malformed)
    );
}

#[test]
fn another_hmac_algorithm_is_the_wrong_algorithm() {
    let key = EncodingKey::from_secret(SECRET.as_bytes());
    let claims = json!({"sub": "u", "iat": NOW, "exp": NOW + 60});
    let token = encode(&Header::new(Algorithm::HS512), &claims, &key).unwrap();

    assert_eq!(
        verify_token(&config_at(NOW), &token),
        Err(AuthError::WrongAlgorithm)
    );
}

#[test]
fn alg_none_is_never_accepted() {
    let header = b64(br#"{"alg":"none","typ":"JWT"}"#);
    let payload = b64(json!({"sub": "admin", "iat": NOW, "exp": NOW + 3600})
        .to_string()
        .as_bytes());

    for token in [
        format!("{header}.{payload}."),
        format!("{header}.{payload}"),
    ] {
        assert_eq!(
            verify_token(&config_at(NOW), &token),
            Err(AuthError::Malformed),
            "{token}"
        );
    }
}

// --------------------------------------------------------------- middleware

async fn send(router: Router, uri: &str, authorization: Option<&str>) -> (StatusCode, Value, bool) {
    let mut request = Request::builder().uri(uri);
    if let Some(value) = authorization {
        request = request.header(header::AUTHORIZATION, value);
    }
    let response = router
        .oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let challenged = response
        .headers()
        .get(header::WWW_AUTHENTICATE)
        .is_some_and(|v| v == "Bearer");
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    (status, body, challenged)
}

fn bearer(config: &JwtConfig, user: &str) -> String {
    format!("Bearer {}", issue_token(config, user))
}

#[tokio::test]
async fn a_valid_token_reaches_the_handler_with_the_identity() {
    let config = config_at(NOW);
    let auth = bearer(&config, "user-42");

    let (status, body, _) = send(app(config), "/whoami", Some(&auth)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], "user-42");
}

#[tokio::test]
async fn no_credentials_is_401_missing_token_with_a_challenge() {
    for authorization in [None, Some("Basic dXNlcjpwYXNz"), Some("garbage")] {
        let (status, body, challenged) = send(app(config_at(NOW)), "/whoami", authorization).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED, "{authorization:?}");
        assert_eq!(body, json!({"error": "missing_token"}), "{authorization:?}");
        assert!(challenged, "WWW-Authenticate: Bearer expected");
    }
}

#[tokio::test]
async fn an_unreadable_or_forged_token_is_401_invalid_token() {
    let mut other = config_at(NOW);
    other.secret = "another-secret-entirely".to_string();
    let forged = bearer(&other, "user-42");

    for authorization in ["Bearer not-a-real-jwt", forged.as_str()] {
        let (status, body, challenged) =
            send(app(config_at(NOW)), "/whoami", Some(authorization)).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body, json!({"error": "invalid_token"}));
        assert!(challenged);
    }
}

#[tokio::test]
async fn an_expired_token_is_401_token_expired() {
    let auth = bearer(&config_at(NOW), "user-42");
    let later = config_at(NOW + 3600 + 31);

    let (status, body, challenged) = send(app(later), "/whoami", Some(&auth)).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body, json!({"error": "token_expired"}));
    assert!(challenged);
}

#[tokio::test]
async fn a_route_added_after_the_layer_is_public() {
    let (status, _, _) = send(app(config_at(NOW)), "/health", None).await;
    assert_eq!(status, StatusCode::OK);
}

// -------------------------------------------------------------------- admin

#[tokio::test]
async fn admin_route_lets_the_admin_in() {
    let config = config_at(NOW);
    let auth = bearer(&config, "matin");

    let (status, body, _) = send(admin_app(config, "matin"), "/admin", Some(&auth)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, Value::String("pong".to_string()));
}

#[tokio::test]
async fn admin_route_is_403_for_any_other_valid_subject() {
    let config = config_at(NOW);
    let auth = bearer(&config, "someone-else");

    let (status, _, _) = send(admin_app(config, "matin"), "/admin", Some(&auth)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_route_without_a_valid_token_is_401_not_403_or_500() {
    let router = || admin_app(config_at(NOW), "matin");

    let (missing, body, _) = send(router(), "/admin", None).await;
    assert_eq!(missing, StatusCode::UNAUTHORIZED);
    assert_eq!(body, json!({"error": "missing_token"}));

    let (garbage, _, _) = send(router(), "/admin", Some("Bearer nope")).await;
    assert_eq!(garbage, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_whoami_route_of_admin_app_is_open_to_any_valid_subject() {
    let config = config_at(NOW);
    let auth = bearer(&config, "someone-else");

    let (status, body, _) = send(admin_app(config, "matin"), "/whoami", Some(&auth)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], "someone-else");
}
