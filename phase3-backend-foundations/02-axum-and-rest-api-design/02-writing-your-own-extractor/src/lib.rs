//! Exercises for 3.2.2 — Writing your own extractor.
//!
//! Three extractors, each implementing `FromRequestParts` by hand, and a
//! small router that uses them. Every rejection is a
//! `(StatusCode, &'static str)`, a type axum already knows how to turn into a
//! response. The README explains each rule.

use axum::extract::{FromRequestParts, Query};
use axum::http::{request::Parts, StatusCode};
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

/// The `per_page` used when the query string doesn't say.
pub const DEFAULT_PER_PAGE: u32 = 20;
/// The largest `per_page` an extractor hands to a handler.
pub const MAX_PER_PAGE: u32 = 100;

/// The caller's API key, taken from the `x-api-key` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKey(pub String);

/// Reads the `x-api-key` header.
///
/// Succeeds with the header value with surrounding whitespace trimmed. Rejects
/// with `401 Unauthorized` and the body text `missing x-api-key header` when
/// the header is absent, is not valid text, or is empty after trimming. No
/// other status code is used.
impl<S: Send + Sync> FromRequestParts<S> for ApiKey {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        todo!("return the trimmed x-api-key header value as an ApiKey, or the 401 rejection described above")
    }
}

/// Which page of a list the caller wants, and how many items per page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

/// The raw shape of the query string; `Pagination` is what handlers see.
#[derive(Deserialize)]
struct RawPagination {
    page: Option<u32>,
    per_page: Option<u32>,
}

/// Reads `?page=N&per_page=M` from the query string.
///
/// - A missing `page` means `1`. A missing `per_page` means
///   [`DEFAULT_PER_PAGE`].
/// - A `per_page` above [`MAX_PER_PAGE`] is not an error: it is lowered to
///   [`MAX_PER_PAGE`].
/// - A value that is not a whole number (`page=abc`, `page=-1`, a repeated
///   key) rejects with `400 Bad Request` and the body text
///   `page and per_page must be whole numbers`.
/// - A `page` or `per_page` of `0` rejects with `400 Bad Request` and the
///   body text `page and per_page must be at least 1`.
impl<S: Send + Sync> FromRequestParts<S> for Pagination {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        todo!("build a Pagination from the query string, applying the defaults, the cap and the two 400 rejections described above")
    }
}

/// The client app's version, taken from the `x-client-version` header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientVersion {
    pub major: u32,
    pub minor: u32,
}

/// Reads the `x-client-version` header, written `MAJOR.MINOR` (for example
/// `1.4`), where both parts are whole numbers.
///
/// - Header absent: `400 Bad Request`, body text `missing x-client-version
///   header`.
/// - Header present but not valid text, or not exactly two whole numbers
///   separated by one dot (`1`, `1.2.3`, `a.b`, `1.`): `400 Bad Request`, body
///   text `x-client-version must look like 1.4`.
impl<S: Send + Sync> FromRequestParts<S> for ClientVersion {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        todo!("parse the x-client-version header into a ClientVersion, or reject with the matching 400 described above")
    }
}

/// `GET /anime` — needs a key and a page. The extractors run left to right,
/// so a request with no key is turned away before its query string is read.
pub async fn list_anime(ApiKey(_key): ApiKey, pagination: Pagination) -> String {
    format!("page={} per_page={}", pagination.page, pagination.per_page)
}

/// `GET /version` — echoes the client version.
pub async fn client_info(version: ClientVersion) -> String {
    format!("client {}.{}", version.major, version.minor)
}

/// The router the tests drive with `oneshot`.
pub fn app() -> Router {
    Router::new()
        .route("/anime", get(list_anime))
        .route("/version", get(client_info))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    async fn call(uri: &str, headers: &[(&str, &str)]) -> (StatusCode, String) {
        let mut builder = Request::builder().uri(uri);
        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }
        let response = app()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024).await.unwrap();
        (status, String::from_utf8(bytes.to_vec()).unwrap())
    }

    const KEY: (&str, &str) = ("x-api-key", "secret-123");

    mod api_key {
        use super::*;

        #[tokio::test]
        async fn a_present_key_lets_the_request_through() {
            let (status, _) = call("/anime", &[KEY]).await;
            assert_eq!(status, StatusCode::OK);
        }

        #[tokio::test]
        async fn a_missing_key_is_401() {
            let (status, body) = call("/anime", &[]).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(body, "missing x-api-key header");
        }

        #[tokio::test]
        async fn a_blank_key_is_401_too() {
            let (status, body) = call("/anime", &[("x-api-key", "   ")]).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(body, "missing x-api-key header");
        }

        #[tokio::test]
        async fn the_value_is_trimmed() {
            let (mut parts, _) = Request::builder()
                .header("x-api-key", "  abc  ")
                .body(())
                .unwrap()
                .into_parts();
            let key = ApiKey::from_request_parts(&mut parts, &()).await.unwrap();
            assert_eq!(key, ApiKey("abc".to_string()));
        }

        #[tokio::test]
        async fn extractors_run_left_to_right_so_the_key_is_checked_first() {
            let (status, _) = call("/anime?page=abc", &[]).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
        }
    }

    mod pagination {
        use super::*;

        #[tokio::test]
        async fn defaults_apply_when_the_query_is_empty() {
            let (status, body) = call("/anime", &[KEY]).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body, "page=1 per_page=20");
        }

        #[tokio::test]
        async fn explicit_values_are_used() {
            let (_, body) = call("/anime?page=3&per_page=50", &[KEY]).await;
            assert_eq!(body, "page=3 per_page=50");
        }

        #[tokio::test]
        async fn one_missing_value_gets_only_its_own_default() {
            let (_, body) = call("/anime?page=7", &[KEY]).await;
            assert_eq!(body, "page=7 per_page=20");
            let (_, body) = call("/anime?per_page=5", &[KEY]).await;
            assert_eq!(body, "page=1 per_page=5");
        }

        #[tokio::test]
        async fn a_too_large_per_page_is_capped_not_rejected() {
            let (status, body) = call("/anime?per_page=5000", &[KEY]).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body, "page=1 per_page=100");
        }

        #[tokio::test]
        async fn garbage_is_400() {
            for uri in ["/anime?page=abc", "/anime?page=-1", "/anime?page=1&page=2"] {
                let (status, body) = call(uri, &[KEY]).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
                assert_eq!(body, "page and per_page must be whole numbers", "{uri}");
            }
        }

        #[tokio::test]
        async fn zero_is_400() {
            for uri in ["/anime?page=0", "/anime?per_page=0"] {
                let (status, body) = call(uri, &[KEY]).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
                assert_eq!(body, "page and per_page must be at least 1", "{uri}");
            }
        }
    }

    mod client_version {
        use super::*;

        #[tokio::test]
        async fn a_good_version_is_parsed() {
            let (status, body) = call("/version", &[("x-client-version", "1.4")]).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body, "client 1.4");
        }

        #[tokio::test]
        async fn a_missing_header_is_400() {
            let (status, body) = call("/version", &[]).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(body, "missing x-client-version header");
        }

        #[tokio::test]
        async fn a_malformed_value_is_400() {
            for bad in ["1", "1.2.3", "a.b", "1.", ".4", "", "1.-2"] {
                let (status, body) = call("/version", &[("x-client-version", bad)]).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{bad:?}");
                assert_eq!(body, "x-client-version must look like 1.4", "{bad:?}");
            }
        }
    }
}
