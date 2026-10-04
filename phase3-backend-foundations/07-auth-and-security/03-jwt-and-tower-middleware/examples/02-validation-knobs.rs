//! What `jsonwebtoken::decode` does with a `Validation`: expiry, leeway, the
//! algorithm list, a missing claim. Expiry uses the real clock here, which is
//! exactly the problem the lesson's `Clock` removes.

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use p3_07_03_jwt_and_tower_middleware::{system_now, Claims};
use serde_json::json;

const SECRET: &[u8] = b"demo-secret-for-3-7-3";

fn sign(alg: Algorithm, exp: u64) -> String {
    let claims = Claims {
        sub: "user-42".to_string(),
        iat: 0,
        exp,
    };
    encode(
        &Header::new(alg),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .unwrap()
}

fn check(label: &str, token: &str, validation: &Validation) {
    let key = DecodingKey::from_secret(SECRET);
    match decode::<serde_json::Value>(token, &key, validation) {
        Ok(data) => println!("{label:<24} ok, sub = {}", data.claims["sub"]),
        Err(error) => println!("{label:<24} {:?}", error.kind()),
    }
}

fn main() {
    let now = system_now();
    let hs256 = Validation::new(Algorithm::HS256);
    println!("default leeway: {} s", hs256.leeway);

    check(
        "valid until 2100",
        &sign(Algorithm::HS256, 4_102_444_800),
        &hs256,
    );
    check("expired in 1970", &sign(Algorithm::HS256, 1), &hs256);
    check(
        "expired 30 s ago",
        &sign(Algorithm::HS256, now - 30),
        &hs256,
    );
    let mut strict = Validation::new(Algorithm::HS256);
    strict.leeway = 0;
    check(
        "30 s ago, leeway 0",
        &sign(Algorithm::HS256, now - 30),
        &strict,
    );

    check(
        "HS512, HS256 pinned",
        &sign(Algorithm::HS512, 4_102_444_800),
        &hs256,
    );
    let mut wide = Validation::new(Algorithm::HS256);
    wide.algorithms = vec![Algorithm::HS256, Algorithm::HS512];
    check(
        "HS512, both allowed",
        &sign(Algorithm::HS512, 4_102_444_800),
        &wide,
    );

    let no_exp = encode(
        &Header::default(),
        &json!({"sub": "user-42", "iat": 0}),
        &EncodingKey::from_secret(SECRET),
    )
    .unwrap();
    check("no exp claim", &no_exp, &hs256);
    check(
        "alg none",
        "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJzdWIiOiJhZG1pbiJ9.",
        &hs256,
    );
}
