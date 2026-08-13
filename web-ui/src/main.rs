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
//! [`progress`] and `docs/adr/0001-web-ui-progress-state.md`), and the answers
//! you type into each checkpoint live beside it in a gitignored
//! `.checkpoint-answers/` directory (see [`answers`]).

mod answers;
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
        .route("/{locale}/checkpoint", post(checkpoint))
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
    let (locale, tree) = match resolve_lesson(&state.root, &locale, &form.path) {
        Ok(resolved) => resolved,
        Err(rejection) => return rejection.into_response(),
    };

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

#[derive(Deserialize)]
struct AnswerForm {
    path: String,
    answer: String,
}

/// Saves what you typed into a lesson's checkpoint editor, then sends you back
/// to it — a plain POST-redirect-GET, so a refresh can't resubmit your answer.
async fn checkpoint(
    State(state): State<AppState>,
    UrlPath(locale): UrlPath<String>,
    Form(form): Form<AnswerForm>,
) -> Response {
    let (locale, _) = match resolve_lesson(&state.root, &locale, &form.path) {
        Ok(resolved) => resolved,
        Err(rejection) => return rejection.into_response(),
    };

    if let Err(err) = answers::save(&state.root, &form.path, &form.answer) {
        eprintln!("course-ui: could not write checkpoint answer: {err}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "could not save answer").into_response();
    }

    Redirect::to(&format!(
        "/{}/{}#{}",
        locale.code(),
        form.path,
        page::ANSWER_ANCHOR
    ))
    .into_response()
}

/// Shared front door for the two handlers that write: parse the locale and
/// confirm `path` names a real lesson. That check is also what keeps a crafted
/// `path` from touching anything, since it has to match a node found on disk.
fn resolve_lesson(
    root: &Path,
    locale: &str,
    path: &str,
) -> Result<(Locale, tree::Node), Rejection> {
    let Some(locale) = Locale::parse(locale) else {
        return Err((StatusCode::NOT_FOUND, "unsupported locale"));
    };
    let Some(tree) = tree::build(root, locale) else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "no content"));
    };
    let is_lesson = tree.find(path).map(|n| n.is_lesson()).unwrap_or(false);
    if !is_lesson {
        return Err((StatusCode::BAD_REQUEST, "not a lesson"));
    }
    Ok((locale, tree))
}

/// Why a write was refused. Kept as the parts rather than a built `Response`,
/// which is a large type to carry around in a `Result`.
type Rejection = (StatusCode, &'static str);

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
    use axum::http::{
        header::{CONTENT_TYPE, LOCATION},
        Request,
    };
    use tower::ServiceExt;

    async fn response_at(root: PathBuf, path: &str) -> Response {
        app(root)
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn response(path: &str) -> Response {
        response_at(default_root(), path).await
    }

    async fn text(response: Response) -> String {
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
            .await
            .unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    async fn body(path: &str) -> String {
        text(response(path).await).await
    }

    /// Form-encoded POST, the only kind this zero-JavaScript UI ever receives.
    async fn post(root: PathBuf, path: &str, form: &str) -> Response {
        app(root)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(path)
                    .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(form.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    /// A throwaway repo, so writing tests don't leave answers in the real one.
    fn temp_repo(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("course-ui-route-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let write = |rel: &str, body: &str| {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        };
        write("README.md", "# Fixture Repo\n");
        write("phase0/README.md", "# Phase 0\n");
        write("phase0/01-intro/README.md", "# 01 - Intro\n");
        write("phase0/01-intro/CHECKPOINT.md", "# Checkpoint\n\n1. Why?\n");
        write("phase0/02-plain/README.md", "# 02 - Plain\n");
        root
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
    async fn lesson_only_renders_visuals_authored_for_its_content() {
        let html = body("/fa/phase0-setup/03-cargo-basics").await;
        assert_eq!(html.matches("class=\"concept-visual").count(), 1);
        assert!(!html.contains("مسیر مفهوم: متین"));
        assert!(html.contains(">check</text>"));
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

    #[tokio::test]
    async fn a_checkpoint_answer_is_written_to_disk_and_read_back_into_the_editor() {
        let root = temp_repo("answers");

        let saved = post(
            root.clone(),
            "/fa/checkpoint",
            // `۱. rustup <them>` — the tag proves the reply is escaped on the
            // way back out, not stored escaped.
            "path=phase0%2F01-intro&answer=%DB%B1.+rustup+%3Cthem%3E",
        )
        .await;
        assert_eq!(saved.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            saved.headers()[LOCATION],
            "/fa/phase0/01-intro#checkpoint-answer",
            "saving lands you back on your own words"
        );
        assert_eq!(
            std::fs::read_to_string(root.join(".checkpoint-answers/phase0/01-intro.md")).unwrap(),
            "۱. rustup <them>\n",
            "the file mirrors the lesson path and holds exactly what was typed"
        );

        let html = text(response_at(root.clone(), "/fa/phase0/01-intro").await).await;
        assert!(html.contains("۱. rustup &lt;them&gt;</textarea>"));
        assert!(html.contains(".checkpoint-answers/phase0/01-intro.md"));

        // Clearing the box removes the file rather than leaving an empty one.
        let cleared = post(
            root.clone(),
            "/fa/checkpoint",
            "path=phase0%2F01-intro&answer=",
        )
        .await;
        assert_eq!(cleared.status(), StatusCode::SEE_OTHER);
        assert!(!root.join(".checkpoint-answers/phase0/01-intro.md").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn the_editor_only_appears_where_there_are_questions_to_answer() {
        let root = temp_repo("editor-placement");

        let with_checkpoint = text(response_at(root.clone(), "/fa/phase0/01-intro").await).await;
        assert!(with_checkpoint.contains("id=\"checkpoint-answer\""));
        assert!(
            with_checkpoint.find("id=\"checkpoint-answer\"")
                > with_checkpoint.find("file-checkpoint-md"),
            "the editor sits under the questions, not above them"
        );

        let without = text(response_at(root.clone(), "/fa/phase0/02-plain").await).await;
        assert!(!without.contains("id=\"checkpoint-answer\""));

        // A phase is not a lesson, so it has nowhere to put an answer.
        let phase = text(response_at(root.clone(), "/fa/phase0").await).await;
        assert!(!phase.contains("id=\"checkpoint-answer\""));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn only_a_real_lesson_can_be_answered() {
        let root = temp_repo("answers-guard");

        for form in [
            "path=phase0&answer=not+a+lesson",
            "path=..%2F..%2Fetc%2Fpasswd&answer=nope",
            "path=&answer=nope",
        ] {
            let response = post(root.clone(), "/fa/checkpoint", form).await;
            assert_eq!(
                response.status(),
                StatusCode::BAD_REQUEST,
                "rejects `{form}`"
            );
        }
        assert!(!root.join(".checkpoint-answers").exists());

        let _ = std::fs::remove_dir_all(&root);
    }
}
