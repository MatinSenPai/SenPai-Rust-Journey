//! Exercises for 3.3.4 — API versioning and evolution.
//!
//! One domain type (`Anime`), two wire shapes (`AnimeV1`, `AnimeV2`), and the
//! rules for moving between them: a classifier for schema changes, the
//! lifecycle headers a retiring version carries, `Accept`-header
//! negotiation, and a router that nests `/v1` and `/v2`. Everything except
//! the `todo!()`s is given. `tests/model_test.rs` checks the pure logic;
//! `tests/api_test.rs` checks the router through `tower::ServiceExt::oneshot`.

use std::sync::Arc;

use axum::extract::{Path, Request, State};
use axum::http::header::{ACCEPT, VARY};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Where you are with a show. On the wire it is snake_case:
/// `"watching"`, `"completed"`, `"plan_to_watch"`, `"dropped"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
}

/// The domain type: what the service knows, with no opinion about JSON.
/// Both wire versions are built from it.
#[derive(Debug, Clone, PartialEq)]
pub struct Anime {
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    /// `1..=10`, or `None` if not rated yet.
    pub rating: Option<u8>,
    /// Episode count, if known. Added after v1 shipped.
    pub episodes: Option<u32>,
}

/// The v1 contract, frozen: `{"id":1,"title":"Frieren","status":"watching","rating":9}`.
/// `rating` is `null` when there is none. v1 never shows `episodes`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeV1 {
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    pub rating: Option<u8>,
}

/// The v2 contract. `status` is renamed to `watch_status` (a breaking change,
/// which is why v2 exists) and `episodes` is added (not breaking). As JSON:
/// `{"id":1,"title":"Frieren","watch_status":"watching","rating":9,"episodes":28}`.
/// `rating` stays `null` when there is none, but `episodes` is left out
/// entirely when unknown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimeV2 {
    pub id: u64,
    pub title: String,
    pub watch_status: WatchStatus,
    pub rating: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episodes: Option<u32>,
}

impl From<&Anime> for AnimeV1 {
    /// Copies `id`, `title`, `status` and `rating`; `episodes` is dropped.
    fn from(a: &Anime) -> Self {
        AnimeV1 {
            id: a.id,
            title: a.title.clone(),
            status: a.status,
            rating: a.rating,
        }
    }
}

impl From<&Anime> for AnimeV2 {
    /// Copies `id`, `title`, `rating` and `episodes`; `status` becomes
    /// `watch_status`.
    fn from(a: &Anime) -> Self {
        AnimeV2 {
            id: a.id,
            title: a.title.clone(),
            watch_status: a.status,
            rating: a.rating,
            episodes: a.episodes,
        }
    }
}

/// Whether a change can break a client that worked before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compat {
    Breaking,
    Compatible,
}

/// One kind of schema change. "Response" is what the server sends and
/// clients read; "request" is what clients send and the server reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    AddResponseField,
    RemoveResponseField,
    RenameResponseField,
    ChangeResponseFieldType,
    AddResponseEnumValue,
    RemoveResponseEnumValue,
    AddOptionalRequestField,
    AddRequiredRequestField,
    RemoveRequestField,
    TightenRequestValidation,
    LoosenRequestValidation,
    AddEndpoint,
    RemoveEndpoint,
}

/// Classifies one change. Assume clients are unknown and the server ignores
/// unknown request fields (serde's default). The table is complete:
///
/// | Change | Result |
/// |---|---|
/// | `AddResponseField` | `Compatible` |
/// | `RemoveResponseField` | `Breaking` |
/// | `RenameResponseField` | `Breaking` |
/// | `ChangeResponseFieldType` | `Breaking` |
/// | `AddResponseEnumValue` | `Breaking` (an exhaustive client has no arm for it) |
/// | `RemoveResponseEnumValue` | `Compatible` |
/// | `AddOptionalRequestField` | `Compatible` |
/// | `AddRequiredRequestField` | `Breaking` |
/// | `RemoveRequestField` | `Compatible` |
/// | `TightenRequestValidation` | `Breaking` |
/// | `LoosenRequestValidation` | `Compatible` |
/// | `AddEndpoint` | `Compatible` |
/// | `RemoveEndpoint` | `Breaking` |
pub fn classify(change: Change) -> Compat {
    use Change::*;
    match change {
        RemoveResponseField
        | RenameResponseField
        | ChangeResponseFieldType
        | AddResponseEnumValue
        | AddRequiredRequestField
        | TightenRequestValidation
        | RemoveEndpoint => Compat::Breaking,
        AddResponseField
        | RemoveResponseEnumValue
        | AddOptionalRequestField
        | RemoveRequestField
        | LoosenRequestValidation
        | AddEndpoint => Compat::Compatible,
    }
}

/// `true` when at least one change in the slice is `Breaking`, so the
/// release needs a new version. An empty slice is `false`.
pub fn requires_new_version(changes: &[Change]) -> bool {
    changes.iter().any(|c| classify(*c) == Compat::Breaking)
}

/// The headers a deprecated version carries, exactly three pairs in this
/// order, names lowercase:
///
/// 1. `("deprecation", "@<deprecated_at_unix>")`, e.g. `@1767225600`
///    (RFC 9745: an `@` followed by Unix seconds).
/// 2. `("sunset", sunset)`: the given HTTP-date, unchanged.
/// 3. `("link", "<successor>; rel=\"successor-version\"")`.
pub fn deprecation_headers(
    deprecated_at_unix: u64,
    sunset: &str,
    successor: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("deprecation", format!("@{deprecated_at_unix}")),
        ("sunset", sunset.to_string()),
        ("link", format!("<{successor}>; rel=\"successor-version\"")),
    ]
}

/// Which wire shape a client asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiVersion {
    V1,
    V2,
}

/// Picks a version from a raw `Accept` header value (`None` if absent).
///
/// Split on `,`, trim each entry, drop everything from the first `;`, and
/// compare ASCII-case-insensitively. The first entry equal to
/// `application/vnd.anime.v2+json` gives `V2`; the first equal to
/// `application/vnd.anime.v1+json` gives `V1`; whichever comes first wins.
/// If no entry matches (header absent, `*/*`, `application/json`, an unknown
/// version like `...v3+json`) the answer is `V1`: clients that say nothing
/// keep the oldest contract. `q` values are ignored.
pub fn version_from_accept(accept: Option<&str>) -> ApiVersion {
    for part in accept.unwrap_or("").split(',') {
        let media = part.split(';').next().unwrap_or("").trim();
        if media.eq_ignore_ascii_case("application/vnd.anime.v2+json") {
            return ApiVersion::V2;
        }
        if media.eq_ignore_ascii_case("application/vnd.anime.v1+json") {
            return ApiVersion::V1;
        }
    }
    ApiVersion::V1
}

/// A read-only in-memory catalog.
pub struct AnimeStore {
    items: Vec<Anime>,
}

impl AnimeStore {
    pub fn new(items: Vec<Anime>) -> Self {
        AnimeStore { items }
    }

    /// Two shows: id 1 `Frieren`, `Watching`, rating 9, 28 episodes; id 2
    /// `Dandadan`, `PlanToWatch`, no rating, no episode count.
    pub fn seeded() -> Self {
        AnimeStore::new(vec![
            Anime {
                id: 1,
                title: "Frieren".into(),
                status: WatchStatus::Watching,
                rating: Some(9),
                episodes: Some(28),
            },
            Anime {
                id: 2,
                title: "Dandadan".into(),
                status: WatchStatus::PlanToWatch,
                rating: None,
                episodes: None,
            },
        ])
    }

    pub fn list(&self) -> &[Anime] {
        &self.items
    }

    pub fn get(&self, id: u64) -> Option<&Anime> {
        self.items.iter().find(|a| a.id == id)
    }
}

/// When v1 was deprecated: 2026-01-01T00:00:00Z.
pub const V1_DEPRECATED_AT: u64 = 1_767_225_600;
/// When v1 stops answering.
pub const V1_SUNSET: &str = "Thu, 31 Dec 2026 23:59:59 GMT";
/// Where v1 clients should go.
pub const V1_SUCCESSOR: &str = "/v2/anime";

type ApiError = (StatusCode, Json<Value>);

fn not_found() -> ApiError {
    (
        StatusCode::NOT_FOUND,
        Json(json!({"error": "anime not found"})),
    )
}

async fn list_v1(State(s): State<Arc<AnimeStore>>) -> Json<Vec<AnimeV1>> {
    Json(s.list().iter().map(AnimeV1::from).collect())
}

async fn get_v1(
    State(s): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<Json<AnimeV1>, ApiError> {
    s.get(id).map(|a| Json(a.into())).ok_or_else(not_found)
}

async fn list_v2(State(s): State<Arc<AnimeStore>>) -> Json<Vec<AnimeV2>> {
    Json(s.list().iter().map(AnimeV2::from).collect())
}

async fn get_v2(
    State(s): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
) -> Result<Json<AnimeV2>, ApiError> {
    s.get(id).map(|a| Json(a.into())).ok_or_else(not_found)
}

fn wanted(headers: &HeaderMap) -> ApiVersion {
    version_from_accept(headers.get(ACCEPT).and_then(|v| v.to_str().ok()))
}

async fn list_negotiated(State(s): State<Arc<AnimeStore>>, headers: HeaderMap) -> Response {
    let body = match wanted(&headers) {
        ApiVersion::V1 => list_v1(State(s)).await.into_response(),
        ApiVersion::V2 => list_v2(State(s)).await.into_response(),
    };
    ([(VARY, "accept")], body).into_response()
}

async fn get_negotiated(
    State(s): State<Arc<AnimeStore>>,
    Path(id): Path<u64>,
    headers: HeaderMap,
) -> Response {
    let body = match wanted(&headers) {
        ApiVersion::V1 => get_v1(State(s), Path(id)).await.into_response(),
        ApiVersion::V2 => get_v2(State(s), Path(id)).await.into_response(),
    };
    ([(VARY, "accept")], body).into_response()
}

/// Middleware: runs the inner handler, then adds the three
/// [`deprecation_headers`] for v1 to whatever it answered.
async fn mark_v1_deprecated(req: Request, next: Next) -> Response {
    let mut response = next.run(req).await;
    for (name, value) in deprecation_headers(V1_DEPRECATED_AT, V1_SUNSET, V1_SUCCESSOR) {
        response.headers_mut().insert(
            HeaderName::from_static(name),
            HeaderValue::from_str(&value).unwrap(),
        );
    }
    response
}

/// The whole API, over a shared store. The routes:
///
/// | Path | Answers | Extra |
/// |---|---|---|
/// | `GET /v1/anime`, `GET /v1/anime/{id}` | `AnimeV1` shapes | every response (404 too) carries the three headers from `deprecation_headers(V1_DEPRECATED_AT, V1_SUNSET, V1_SUCCESSOR)` |
/// | `GET /v2/anime`, `GET /v2/anime/{id}` | `AnimeV2` shapes | no deprecation headers |
/// | `GET /anime`, `GET /anime/{id}` | the shape `version_from_accept` picks from the `Accept` header | `vary: accept` on every response |
///
/// A missing id is `404` with `{"error":"anime not found"}` on every route.
/// The handlers `list_v1`, `get_v1`, `list_v2`, `get_v2`, `list_negotiated`,
/// `get_negotiated` and the middleware `mark_v1_deprecated` are given.
pub fn app(store: Arc<AnimeStore>) -> Router {
    let v1 = Router::new()
        .route("/anime", get(list_v1))
        .route("/anime/{id}", get(get_v1))
        .layer(middleware::from_fn(mark_v1_deprecated));
    let v2 = Router::new()
        .route("/anime", get(list_v2))
        .route("/anime/{id}", get(get_v2));
    Router::new()
        .nest("/v1", v1)
        .nest("/v2", v2)
        .route("/anime", get(list_negotiated))
        .route("/anime/{id}", get(get_negotiated))
        .with_state(store)
}

/// Build rung: after the sunset date every `/v1/...` request answers
/// `410 Gone`, with the same `link` header value as the third pair of
/// `deprecation_headers` (successor `V1_SUCCESSOR`) and the JSON
/// body `{"error":"v1 was retired; use /v2"}`. `/v2` and the unversioned
/// routes behave as in [`app`].
pub fn app_after_sunset(store: Arc<AnimeStore>) -> Router {
    async fn gone() -> impl IntoResponse {
        (
            StatusCode::GONE,
            [(
                "link",
                format!("<{V1_SUCCESSOR}>; rel=\"successor-version\""),
            )],
            Json(json!({"error": "v1 was retired; use /v2"})),
        )
    }
    let v1 = Router::new().fallback(gone);
    let v2 = Router::new()
        .route("/anime", get(list_v2))
        .route("/anime/{id}", get(get_v2));
    Router::new()
        .nest("/v1", v1)
        .nest("/v2", v2)
        .route("/anime", get(list_negotiated))
        .route("/anime/{id}", get(get_negotiated))
        .with_state(store)
}

/// Challenge: compares two JSON response objects and lists the changes.
/// First go through `old`'s keys in sorted order: a key missing from `new`
/// gives `RemoveResponseField`; a key in both whose JSON kinds differ (null,
/// bool, number, string, array, object) gives `ChangeResponseFieldType`.
/// Then go through the keys only `new` has, in sorted order, each giving
/// `AddResponseField`. A rename therefore shows up as a remove plus an add.
/// If either input is not an object the answer is an empty list.
pub fn diff_shapes(old: &Value, new: &Value) -> Vec<Change> {
    let (Some(o), Some(n)) = (old.as_object(), new.as_object()) else {
        return vec![];
    };
    let mut out = vec![];
    let mut old_keys: Vec<_> = o.keys().collect();
    old_keys.sort();
    for k in old_keys {
        match n.get(k) {
            None => out.push(Change::RemoveResponseField),
            Some(nv) if std::mem::discriminant(nv) != std::mem::discriminant(&o[k]) => {
                out.push(Change::ChangeResponseFieldType)
            }
            _ => {}
        }
    }
    out.extend(
        n.keys()
            .filter(|k| !o.contains_key(*k))
            .map(|_| Change::AddResponseField),
    );
    out
}
