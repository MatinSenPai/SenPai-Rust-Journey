//! Exercises for 3.3.3 — API contracts and OpenAPI (`utoipa`).
//!
//! The first half of this file is a small anime API whose contract is
//! *generated*: every type derives `ToSchema`, every handler carries a
//! `#[utoipa::path]`, and `ApiDoc` gathers them into one OpenAPI document.
//! All of it is given. The second half (the "Implement" rung) is four small
//! readers for that document, so a test can ask the contract questions like
//! "which status codes does `POST /anime` promise?".
//!
//! The readers work on the document as a plain `serde_json::Value`, the
//! exact JSON a client downloads from `GET /api-docs/openapi.json`.

use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{OpenApi, ToSchema};

/// Where you are with a show. On the wire it is snake_case:
/// `"watching"`, `"completed"`, `"plan_to_watch"`, `"dropped"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus {
    Watching,
    Completed,
    PlanToWatch,
    Dropped,
}

/// One catalog entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Anime {
    #[schema(example = 1)]
    pub id: u64,
    #[schema(example = "Frieren")]
    pub title: String,
    pub status: WatchStatus,
    /// `1..=10`, or `null` when the show is not rated yet.
    #[schema(minimum = 1, maximum = 10)]
    pub rating: Option<u8>,
}

/// The body of `POST /anime`. The server picks the id.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAnime {
    #[schema(example = "Frieren")]
    pub title: String,
    pub status: WatchStatus,
    #[schema(minimum = 1, maximum = 10)]
    pub rating: Option<u8>,
}

/// The JSON body of every error this API produces itself.
#[derive(Debug, Serialize, ToSchema)]
pub struct ApiError {
    #[schema(example = "anime not found")]
    pub error: String,
}

#[derive(Default)]
struct Inner {
    last_id: u64,
    items: Vec<Anime>,
}

/// The in-memory catalog, shared by every handler. Ids start at 1, go up by
/// one per create, and are never handed out twice.
#[derive(Default)]
pub struct Catalog {
    inner: Mutex<Inner>,
}

type ApiResult<T> = Result<T, (StatusCode, Json<ApiError>)>;

fn error(status: StatusCode, message: String) -> (StatusCode, Json<ApiError>) {
    (status, Json(ApiError { error: message }))
}

/// `GET /anime`: `200 OK`, a JSON array of every anime in creation order.
#[utoipa::path(
    get,
    path = "/anime",
    responses((status = 200, description = "every anime", body = Vec<Anime>)),
    tag = "anime"
)]
pub async fn list_anime(State(catalog): State<Arc<Catalog>>) -> Json<Vec<Anime>> {
    Json(catalog.inner.lock().unwrap().items.clone())
}

/// `POST /anime`, body [`CreateAnime`].
///
/// - `201 Created` and the new [`Anime`] (ids start at 1).
/// - `422` with an [`ApiError`] body when the rating is outside `1..=10`
///   (`{"error":"rating must be between 1 and 10, got 15"}`).
/// - `400`, `415` and a second kind of `422` come from axum's own `Json`
///   extractor before this function runs, as `text/plain`.
#[utoipa::path(
    post,
    path = "/anime",
    request_body = CreateAnime,
    responses(
        (status = 201, description = "created", body = Anime),
        (status = 400, description = "the body is not valid JSON",
            body = String, content_type = "text/plain"),
        (status = 415, description = "Content-Type is not application/json",
            body = String, content_type = "text/plain"),
        (status = 422, description = "rating out of range, or JSON of the wrong shape",
            content((ApiError = "application/json"), (String = "text/plain"))),
    ),
    tag = "anime"
)]
pub async fn create_anime(
    State(catalog): State<Arc<Catalog>>,
    Json(input): Json<CreateAnime>,
) -> ApiResult<(StatusCode, Json<Anime>)> {
    if let Some(r) = input.rating.filter(|r| !(1..=10).contains(r)) {
        let message = format!("rating must be between 1 and 10, got {r}");
        return Err(error(StatusCode::UNPROCESSABLE_ENTITY, message));
    }
    let mut inner = catalog.inner.lock().unwrap();
    inner.last_id += 1;
    let anime = Anime {
        id: inner.last_id,
        title: input.title,
        status: input.status,
        rating: input.rating,
    };
    inner.items.push(anime.clone());
    Ok((StatusCode::CREATED, Json(anime)))
}

/// `GET /anime/{id}`: `200 OK` with the anime, or `404` with
/// `{"error":"anime not found"}`.
#[utoipa::path(
    get,
    path = "/anime/{id}",
    params(("id" = u64, Path, description = "the anime's id")),
    responses(
        (status = 200, description = "found", body = Anime),
        (status = 404, description = "no anime has this id", body = ApiError),
    ),
    tag = "anime"
)]
pub async fn get_anime(
    State(catalog): State<Arc<Catalog>>,
    Path(id): Path<u64>,
) -> ApiResult<Json<Anime>> {
    let inner = catalog.inner.lock().unwrap();
    match inner.items.iter().find(|a| a.id == id) {
        Some(anime) => Ok(Json(anime.clone())),
        None => Err(error(StatusCode::NOT_FOUND, "anime not found".into())),
    }
}

/// `DELETE /anime/{id}`: `204 No Content` with an empty body, or `404` with
/// `{"error":"anime not found"}`.
#[utoipa::path(
    delete,
    path = "/anime/{id}",
    params(("id" = u64, Path, description = "the anime's id")),
    responses(
        (status = 204, description = "deleted"),
        (status = 404, description = "no anime has this id", body = ApiError),
    ),
    tag = "anime"
)]
pub async fn delete_anime(
    State(catalog): State<Arc<Catalog>>,
    Path(id): Path<u64>,
) -> ApiResult<StatusCode> {
    let mut inner = catalog.inner.lock().unwrap();
    match inner.items.iter().position(|a| a.id == id) {
        Some(index) => {
            inner.items.remove(index);
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err(error(StatusCode::NOT_FOUND, "anime not found".into())),
    }
}

/// The contract, generated from the types and handlers listed here.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Anime catalog",
        version = "1.0.0",
        description = "A tiny catalog API, documented from its own code.",
        license(name = "MIT")
    ),
    paths(list_anime, create_anime, get_anime, delete_anime),
    tags((name = "anime", description = "the catalog itself"))
)]
pub struct ApiDoc;

/// Every `(METHOD, path)` the router below answers, except the spec route.
/// Written by hand, which is the point: `undocumented` compares it with the
/// generated document.
pub const ROUTES: &[(&str, &str)] = &[
    ("GET", "/anime"),
    ("POST", "/anime"),
    ("GET", "/anime/{id}"),
    ("DELETE", "/anime/{id}"),
];

/// `GET /api-docs/openapi.json`: the generated document as JSON.
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

/// The API plus the route that serves its own contract.
pub fn app(catalog: Arc<Catalog>) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .route("/anime/{id}", get(get_anime).delete(delete_anime))
        .route("/api-docs/openapi.json", get(openapi_json))
        .with_state(catalog)
}

/// The names under `components.schemas`, sorted (plain string order).
/// A document with no `components` or no `schemas` gives an empty `Vec`.
pub fn schema_names(doc: &Value) -> Vec<String> {
    todo!("return the sorted names of the schemas the document defines, or nothing when it defines none")
}

/// The response status codes one operation promises, as the strings the
/// document uses (`"200"`, `"404"`), sorted. `method` is any case (`"GET"`
/// and `"get"` mean the same); `path` is the document's spelling
/// (`"/anime/{id}"`). An operation the document does not contain gives an
/// empty `Vec`.
pub fn operation_statuses(doc: &Value, method: &str, path: &str) -> Vec<String> {
    todo!("return the sorted status codes the document lists for this operation, or nothing if it has no such operation")
}

/// The media types one response lists under `content`
/// (`"application/json"`, `"text/plain"`), sorted. Same `method` and `path`
/// rules as [`operation_statuses`]; `status` is the string key (`"422"`).
/// A response with no `content` (a `204`, say), a status the operation does
/// not have, or an operation that does not exist: an empty `Vec`.
pub fn response_content_types(doc: &Value, method: &str, path: &str, status: &str) -> Vec<String> {
    todo!("return the sorted media types this response lists, or nothing when there are none")
}

/// The routes the document does not describe: every `(method, path)` of
/// `routes` for which the document has no operation, formatted as
/// `"METHOD path"` (the method as given, then one space, then the path:
/// `"DELETE /anime/{id}"`), in the order `routes` lists them. Method case
/// does not matter when looking an operation up. `routes` all documented:
/// an empty `Vec`.
pub fn undocumented(doc: &Value, routes: &[(&str, &str)]) -> Vec<String> {
    todo!("list, in the given order, the routes that have no operation in the document")
}
