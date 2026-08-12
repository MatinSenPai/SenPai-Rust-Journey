//! `course-ui` — a local web UI for reading this curriculum and tracking which
//! lessons you've finished.
//!
//! ```sh
//! cargo run -p course-ui            # serves http://127.0.0.1:5000 and opens it
//! cargo run -p course-ui -- --no-open
//! ```
//!
//! The navigation tree is derived from the repo's directory layout (see
//! [`tree`]); completion lives in a gitignored `.course-progress.json` (see
//! [`progress`] and `docs/adr/0001-web-ui-progress-state.md`).

mod locale;
mod page;
mod progress;
mod render;
mod search;
mod style;
mod tree;
mod visual;

use std::path::{Path, PathBuf};
use std::process::Command;

use axum::extract::{Path as UrlPath, Query, State};
use axum::http::{header, StatusCode, Uri};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

use locale::Locale;

/// Fixed so the URL in your history and bookmarks keeps working. If it's taken
/// we fail loudly rather than silently landing on a different port.
const PORT: u16 = 5000;

#[derive(Clone)]
struct AppState {
    root: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(default_root);

    if tree::build(&root, Locale::En).is_none() {
        eprintln!(
            "course-ui: no markdown found under {} — is that the repo root?",
            root.display()
        );
        std::process::exit(1);
    }

    let app = app(root.clone());

    let addr = format!("127.0.0.1:{PORT}");
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("course-ui: could not bind {addr}: {err}");
            eprintln!("         something else is already using port {PORT}.");
            std::process::exit(1);
        }
    };

    let url = format!("http://{addr}");
    println!("course-ui: serving {} on {url}", root.display());
    if !args.no_open {
        open_browser(&url);
    }

    if let Err(err) = axum::serve(listener, app).await {
        eprintln!("course-ui: server error: {err}");
        std::process::exit(1);
    }
}

fn app(root: PathBuf) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/assets/style.css", get(stylesheet))
        .route("/assets/vazirmatn.woff2", get(font))
        .route("/{locale}/search", get(search_handler))
        .route("/{locale}/mark", post(mark))
        .route("/{locale}/", get(locale_root_handler))
        .route("/{locale}/{*path}", get(node_handler))
        .fallback(legacy_handler)
        .with_state(AppState { root })
}

// ---------------------------------------------------------------- handlers

async fn root_handler() -> Response {
    Redirect::permanent("/fa/").into_response()
}

async fn locale_root_handler(
    State(state): State<AppState>,
    UrlPath(locale): UrlPath<String>,
) -> Response {
    render_path(&state.root, &locale, "")
}

async fn node_handler(
    State(state): State<AppState>,
    UrlPath((locale, path)): UrlPath<(String, String)>,
) -> Response {
    if Locale::parse(&locale).is_none() {
        return Redirect::permanent(&format!(
            "/fa/{}/{}",
            locale.trim_matches('/'),
            path.trim_matches('/')
        ))
        .into_response();
    }
    render_path(&state.root, &locale, path.trim_matches('/'))
}

async fn legacy_handler(uri: Uri) -> Response {
    Redirect::permanent(&format!("/fa/{}", uri.path().trim_matches('/'))).into_response()
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        style::CSS,
    )
}

async fn font() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "font/woff2"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        include_bytes!("../assets/Vazirmatn-Variable.woff2").as_slice(),
    )
}

#[derive(Default, Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: String,
}

async fn search_handler(
    State(state): State<AppState>,
    UrlPath(locale): UrlPath<String>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let Some(locale) = Locale::parse(&locale) else {
        return (StatusCode::NOT_FOUND, "unsupported locale").into_response();
    };
    let Some(tree) = tree::build(&state.root, locale) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no content").into_response();
    };
    let progress = progress::Progress::load(&state.root);
    let hits = search::find(&state.root, &tree, locale, &query.q);
    Html(page::render_search(
        &tree, &progress, locale, &query.q, &hits,
    ))
    .into_response()
}

#[derive(Deserialize)]
struct MarkForm {
    path: String,
    complete: String,
    advance: Option<String>,
}

async fn mark(
    State(state): State<AppState>,
    UrlPath(locale): UrlPath<String>,
    Form(form): Form<MarkForm>,
) -> Response {
    let Some(locale) = Locale::parse(&locale) else {
        return (StatusCode::NOT_FOUND, "unsupported locale").into_response();
    };
    let Some(tree) = tree::build(&state.root, locale) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no content").into_response();
    };

    // Only a real lesson can be marked — this is also what keeps a crafted
    // `path` from touching anything, since it must match a node on disk.
    let is_lesson = tree
        .find(&form.path)
        .map(|n| n.is_lesson())
        .unwrap_or(false);
    if !is_lesson {
        return (StatusCode::BAD_REQUEST, "not a lesson").into_response();
    }

    let mut progress = progress::Progress::load(&state.root);
    progress.set(&form.path, form.complete == "true");
    if let Err(err) = progress.save(&state.root) {
        eprintln!("course-ui: could not write progress: {err}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "could not save progress").into_response();
    }

    let destination = if form.advance.as_deref() == Some("true") && form.complete == "true" {
        let lessons = tree.lessons();
        lessons
            .iter()
            .position(|lesson| lesson.path == form.path)
            .and_then(|index| lessons.get(index + 1))
            .map(|lesson| lesson.path.as_str())
            .unwrap_or(&form.path)
    } else {
        &form.path
    };
    Redirect::to(&format!("/{}/{}", locale.code(), destination)).into_response()
}

/// The tree is rebuilt per request — a few milliseconds, and it means an edited
/// README or a new lesson directory shows up on refresh with no cache to bust.
fn render_path(root: &Path, locale: &str, path: &str) -> Response {
    let Some(locale) = Locale::parse(locale) else {
        return (StatusCode::NOT_FOUND, "unsupported locale").into_response();
    };
    let Some(tree) = tree::build(root, locale) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no content").into_response();
    };
    let progress = progress::Progress::load(root);

    match tree.find(path) {
        Some(node) => Html(page::render_node(root, &tree, node, &progress, locale)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Html(page::render_missing(&tree, path, &progress, locale)),
        )
            .into_response(),
    }
}

// ---------------------------------------------------------------- startup

struct Args {
    root: Option<PathBuf>,
    no_open: bool,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut parsed = Args {
            root: None,
            no_open: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--no-open" => parsed.no_open = true,
                "--root" => parsed.root = args.next().map(PathBuf::from),
                other => {
                    eprintln!("course-ui: unknown argument `{other}`");
                    eprintln!("usage: cargo run -p course-ui -- [--root <path>] [--no-open]");
                    std::process::exit(2);
                }
            }
        }
        parsed
    }
}

/// The repo root is this crate's parent directory — `web-ui/` sits directly
/// under it. `--root` overrides for anyone running the binary from elsewhere.
fn default_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn open_browser(url: &str) {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };
    if let Err(err) = result {
        eprintln!("course-ui: could not open a browser ({err}) — visit {url}");
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;
    use axum::body::{to_bytes, Body};
    use axum::http::{header::LOCATION, Request};
    use tower::ServiceExt;

    async fn response(path: &str) -> Response {
        app(default_root())
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn body(path: &str) -> String {
        let response = response(path).await;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    #[tokio::test]
    async fn root_redirects_to_persian_dashboard() {
        let response = response("/").await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(response.headers()[LOCATION], "/fa/");
    }

    #[tokio::test]
    async fn legacy_bookmark_redirects_to_same_persian_path() {
        let response = response("/phase1-fundamentals/02-ownership-and-memory").await;
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(
            response.headers()[LOCATION],
            "/fa/phase1-fundamentals/02-ownership-and-memory"
        );
    }

    #[tokio::test]
    async fn persian_lesson_is_rtl_and_switches_to_the_same_path() {
        let path = "/fa/phase1-fundamentals/02-ownership-and-memory/01-move-semantics";
        let html = body(path).await;
        assert!(html.contains("<html lang=\"fa\" dir=\"rtl\""));
        assert!(html.contains("معنای انتقال"));
        assert!(html.contains(
            "href=\"/en/phase1-fundamentals/02-ownership-and-memory/01-move-semantics\""
        ));
    }

    #[tokio::test]
    async fn persian_search_escapes_the_query() {
        let html = body("/fa/search?q=%D9%85%D8%A7%D9%84%DA%A9%DB%8C%D8%AA%3Cscript%3E").await;
        assert!(!html.contains("<script>"));
        assert!(html.contains("جست‌وجو"));
    }

    #[tokio::test]
    async fn unsupported_locale_is_not_found() {
        let response = response("/de/").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
