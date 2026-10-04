//! Sign one token with `jsonwebtoken`, then take it apart with nothing but a
//! base64url decoder and `split('.')`. No key is used to read it.

use jsonwebtoken::{encode, EncodingKey, Header};
use p3_07_03_jwt_and_tower_middleware::Claims;

/// Decodes unpadded base64url. Enough for this demo, and nothing else.
fn b64url_decode(text: &str) -> Vec<u8> {
    let value = |c: u8| match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'-' => 62,
        _ => 63,
    };
    let mut out = Vec::new();
    for chunk in text.as_bytes().chunks(4) {
        let n = chunk
            .iter()
            .fold(0u32, |acc, &c| (acc << 6) | value(c) as u32);
        let n = n << (6 * (4 - chunk.len()));
        for i in 0..chunk.len() - 1 {
            out.push((n >> (16 - 8 * i)) as u8);
        }
    }
    out
}

fn main() {
    // Fixed numbers, so the output is the same on every run.
    let claims = Claims {
        sub: "user-42".to_string(),
        iat: 1_700_000_000,
        exp: 1_700_003_600,
    };
    let key = EncodingKey::from_secret(b"demo-secret-for-3-7-3");
    let token = encode(&Header::default(), &claims, &key).unwrap();
    println!("token:     {token}");

    let parts: Vec<&str> = token.split('.').collect();
    println!("parts:     {}", parts.len());
    println!(
        "header:    {}",
        String::from_utf8(b64url_decode(parts[0])).unwrap()
    );
    println!(
        "payload:   {}",
        String::from_utf8(b64url_decode(parts[1])).unwrap()
    );
    println!(
        "signature: {} base64url characters (32 bytes of HMAC-SHA256)",
        parts[2].len()
    );
}
