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

mod concepts;
mod dashboard;
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

/// Every write to `.course-progress.json` is a load-modify-save, so two
/// requests arriving together could each load, each modify their own copy, and
/// the second save could drop the first's change. A browser opening several
/// tabs (or a prefetch) is enough to trigger it. One process-wide lock makes
/// the sequence atomic; `Progress::save` makes the file write itself atomic.
type WriteLock = std::sync::Arc<tokio::sync::Mutex<()>>;

#[derive(Clone)]
struct AppState {
    root: PathBuf,
    /// When false, serving a page records nothing. Set by `--no-track`, and by
    /// the tests — which run against this very repo and must not touch the
    /// reader's real progress.
    tracking: bool,
    write_lock: WriteLock,
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

    let app = app(root.clone(), !args.no_track);

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

fn app(root: PathBuf, tracking: bool) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/assets/style.css", get(stylesheet))
        .route("/assets/app.js", get(script))
        .route("/assets/vazirmatn.woff2", get(vazirmatn))
        .route("/assets/jetbrains-mono.woff2", get(jetbrains_mono))
        .route("/{locale}/search", get(search_handler))
        .route("/{locale}/progress", get(progress_handler))
        .route("/{locale}/mark", post(mark))
        .route("/{locale}/self-check", post(self_check))
        .route("/{locale}/", get(locale_root_handler))
        .route("/{locale}/{*path}", get(node_handler))
        .fallback(legacy_handler)
        .with_state(AppState {
            root,
            tracking,
            write_lock: WriteLock::default(),
        })
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
    let path = path.trim_matches('/');
    record_first_view(&state, path).await;
    render_path(&state.root, &locale, path)
}

/// Opening a lesson for the first time starts its clock, which is what makes
/// the time estimate on the progress page possible. Only the *first* view
/// writes: re-reading a finished lesson never rewrites its history, so the
/// overwhelmingly common case is a pure read.
async fn record_first_view(state: &AppState, path: &str) {
    if !state.tracking || path.is_empty() {
        return;
    }
    let Some(tree) = tree::build(&state.root, Locale::En) else {
        return;
    };
    if !tree.find(path).is_some_and(tree::Node::is_lesson) {
        return;
    }

    let _guard = state.write_lock.lock().await;
    let mut progress = progress::Progress::load(&state.root);
    if !progress.touch(path) {
        return;
    }
    if let Err(err) = progress.save(&state.root) {
        eprintln!("course-ui: could not record first view: {err}");
    }
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

async fn script() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        style::JS,
    )
}

/// Vazirmatn carries both Persian and Latin, so one face covers all prose.
async fn vazirmatn() -> impl IntoResponse {
    font(include_bytes!("../assets/Vazirmatn-Variable.woff2").as_slice())
}

/// Code only. Kept off `bdi`/`[dir=ltr]` so inline English inside Persian
/// prose does not change typeface mid-sentence.
async fn jetbrains_mono() -> impl IntoResponse {
    font(include_bytes!("../assets/JetBrainsMono-Variable.woff2").as_slice())
}

fn font(bytes: &'static [u8]) -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, "font/woff2"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        bytes,
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

async fn progress_handler(
    State(state): State<AppState>,
    UrlPath(locale): UrlPath<String>,
) -> Response {
    let Some(locale) = Locale::parse(&locale) else {
        return (StatusCode::NOT_FOUND, "unsupported locale").into_response();
    };
    let Some(tree) = tree::build(&state.root, locale) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no content").into_response();
    };
    let progress = progress::Progress::load(&state.root);
    let body = dashboard::render(&state.root, &tree, &progress, locale);
    Html(page::render_shell(
        locale.progress_title(),
        &tree,
        &progress,
        locale,
        &body,
    ))
    .into_response()
}

/// The exercise ticks, the confidence rating and the note all post here. They
/// are separate from `/mark` on purpose: none of them means "finished", and
/// conflating them would let a stray checkbox tick a lesson complete.
#[derive(Deserialize)]
struct SelfCheckForm {
    path: String,
    /// Rung slugs currently ticked. Absent means "none" — an unchecked box
    /// sends nothing, so the whole set is replaced rather than merged.
    #[serde(default)]
    exercise: Vec<String>,
    #[serde(default)]
    rungs: String,
    confidence: Option<String>,
    note: Option<String>,
}

async fn self_check(
    State(state): State<AppState>,
    UrlPath(locale): UrlPath<String>,
    Form(form): Form<SelfCheckForm>,
) -> Response {
    let Some(locale) = Locale::parse(&locale) else {
        return (StatusCode::NOT_FOUND, "unsupported locale").into_response();
    };
    let Some(tree) = tree::build(&state.root, locale) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no content").into_response();
    };
    if !tree.find(&form.path).is_some_and(tree::Node::is_lesson) {
        return (StatusCode::BAD_REQUEST, "not a lesson").into_response();
    }

    let _guard = state.write_lock.lock().await;
    let mut progress = progress::Progress::load(&state.root);

    // `rungs` lists every checkbox the form rendered, so an unticked one is
    // cleared rather than left behind from a previous save.
    for slug in form.rungs.split(',').filter(|slug| !slug.is_empty()) {
        progress.set_exercise(&form.path, slug, form.exercise.iter().any(|e| e == slug));
    }
    if let Some(confidence) = form.confidence.as_deref() {
        progress.set_confidence(&form.path, confidence.parse().ok());
    }
    if let Some(note) = form.note.as_deref() {
        progress.set_note(&form.path, note);
    }

    if let Err(err) = progress.save(&state.root) {
        eprintln!("course-ui: could not write progress: {err}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "could not save progress").into_response();
    }
    Redirect::to(&format!("/{}/{}#self-check", locale.code(), form.path)).into_response()
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

    let _guard = state.write_lock.lock().await;
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
    /// Browse without recording anything — useful when you are looking
    /// something up in a checkout whose progress file is not yours.
    no_track: bool,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let mut parsed = Args {
            root: None,
            no_open: false,
            no_track: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--no-open" => parsed.no_open = true,
                "--no-track" => parsed.no_track = true,
                "--root" => parsed.root = args.next().map(PathBuf::from),
                other => {
                    eprintln!("course-ui: unknown argument `{other}`");
                    eprintln!(
                        "usage: cargo run -p course-ui -- [--root <path>] [--no-open] [--no-track]"
                    );
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

    /// Tracking off: these tests run against this very repository, and a GET
    /// that recorded a first view would edit the reader's real progress file.
    async fn response(path: &str) -> Response {
        app(default_root(), false)
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
        // A rebuilt lesson, so this test doesn't break every time the
        // curriculum is reordered.
        let path = "/fa/phase1-fundamentals/01-foundations/01-variables-mutability-shadowing";
        let html = body(path).await;
        assert!(html.contains("<html lang=\"fa\" dir=\"rtl\""));
        assert!(html.contains("سایه‌زنی"));
        assert!(html.contains(
            "href=\"/en/phase1-fundamentals/01-foundations/01-variables-mutability-shadowing\""
        ));
    }

    /// A lesson shows the diagrams its markdown authors, and no others. The
    /// dashboard's roadmap figure is generated; lesson figures never are.
    #[tokio::test]
    async fn lesson_only_renders_visuals_authored_for_its_content() {
        let authored = body(
            "/fa/phase3-backend-foundations/05-database-design-and-query-performance/02-pagination",
        )
        .await;
        assert_eq!(authored.matches("class=\"concept-visual").count(), 1);
        assert!(authored.contains("concept-database"));
        assert!(authored.contains(">cursor آخرین ردیف دیده‌شده</text>"));

        let unauthored = body("/fa/phase0-setup/04-cargo-basics").await;
        assert_eq!(
            unauthored.matches("class=\"concept-visual").count(),
            0,
            "a lesson with no senpai-visual fence must not be given a decorative one"
        );
    }

    #[tokio::test]
    async fn persian_search_escapes_the_query() {
        let html = body("/fa/search?q=%D9%85%D8%A7%D9%84%DA%A9%DB%8C%D8%AA%3Cscript%3E").await;
        assert!(html.contains("جست‌وجو"));
        assert!(
            html.contains("&lt;script&gt;"),
            "the query is reflected back, escaped"
        );
        assert!(!html.contains("مالکیت<script>"), "and never reflected raw");
        // The only scripts on any page are ours: the inline theme bootstrap
        // and the deferred app.js. Anything else came from user input.
        assert_eq!(html.matches("<script").count(), 2);
    }

    #[tokio::test]
    async fn unsupported_locale_is_not_found() {
        let response = response("/de/").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn every_page_ships_both_faces_and_the_theme_control() {
        let html = body("/fa/").await;
        assert!(html.contains("/assets/vazirmatn.woff2"));
        assert!(html.contains("/assets/jetbrains-mono.woff2"));
        assert!(html.contains("class=\"theme-toggle\""));
        assert!(
            html.contains("course-ui-theme"),
            "the stored theme is applied before first paint"
        );
    }

    #[tokio::test]
    async fn assets_are_served_with_the_right_content_type() {
        for (path, content_type) in [
            ("/assets/style.css", "text/css; charset=utf-8"),
            ("/assets/app.js", "text/javascript; charset=utf-8"),
            ("/assets/vazirmatn.woff2", "font/woff2"),
            ("/assets/jetbrains-mono.woff2", "font/woff2"),
        ] {
            let response = response(path).await;
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(
                response.headers()[header::CONTENT_TYPE],
                content_type,
                "{path}"
            );
        }
    }

    #[tokio::test]
    async fn the_progress_page_renders_in_both_languages() {
        let fa = body("/fa/progress").await;
        assert!(fa.contains("class=\"stat-row\""));
        assert!(fa.contains(&format!("<h1>{}</h1>", Locale::Fa.progress_title())));
        assert!(
            fa.contains("class=\"mastery\""),
            "the concept grid renders from docs/concept-map.toml"
        );

        let en = body("/en/progress").await;
        assert!(en.contains("How far you've come"));
    }

    #[tokio::test]
    async fn a_lesson_offers_its_self_check_but_a_phase_index_does_not() {
        let lesson = body("/fa/phase0-setup/04-cargo-basics").await;
        assert!(lesson.contains("id=\"self-check\""));
        assert!(lesson.contains("name=\"confidence\""));

        let phase = body("/fa/phase0-setup").await;
        assert!(
            !phase.contains("id=\"self-check\""),
            "only a lesson is something you can assess yourself on"
        );
    }

    /// A miniature repo so the tracking path can be exercised without touching
    /// the reader's real progress file. This is the path that, before the
    /// write lock and the atomic save, could lose completed lessons.
    fn tracking_fixture(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("course-ui-track-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let lesson = root.join("phase0-setup").join("01-intro");
        std::fs::create_dir_all(&lesson).unwrap();
        std::fs::write(root.join("README.md"), "# Fixture\n").unwrap();
        std::fs::write(root.join("phase0-setup").join("README.md"), "# Phase 0\n").unwrap();
        std::fs::write(lesson.join("README.md"), "# 01 - Intro\n").unwrap();
        root
    }

    async fn get(root: &Path, tracking: bool, path: &str) -> StatusCode {
        app(root.to_path_buf(), tracking)
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn a_first_view_starts_the_clock_and_a_second_leaves_it_alone() {
        let root = tracking_fixture("first-view");
        assert_eq!(
            get(&root, true, "/en/phase0-setup/01-intro").await,
            StatusCode::OK
        );

        let after_first = std::fs::read_to_string(root.join(progress::FILE_NAME)).unwrap();
        assert!(after_first.contains("\"first_seen_at\""));
        assert!(after_first.contains("in-progress"));

        assert_eq!(
            get(&root, true, "/en/phase0-setup/01-intro").await,
            StatusCode::OK
        );
        assert_eq!(
            std::fs::read_to_string(root.join(progress::FILE_NAME)).unwrap(),
            after_first,
            "re-reading a lesson must not rewrite its history"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn no_track_serves_the_same_page_and_records_nothing() {
        let root = tracking_fixture("no-track");
        assert_eq!(
            get(&root, false, "/en/phase0-setup/01-intro").await,
            StatusCode::OK
        );
        assert!(
            !root.join(progress::FILE_NAME).exists(),
            "--no-track must not create a progress file"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn viewing_a_lesson_never_drops_existing_completions() {
        // The regression this guards: a torn read during a concurrent write let
        // an empty state be saved over a file holding real completions.
        let root = tracking_fixture("keeps-completions");
        std::fs::write(
            root.join(progress::FILE_NAME),
            r#"{"version":1,"completed":["phase0-setup/01-intro","phase9/gone"]}"#,
        )
        .unwrap();

        assert_eq!(
            get(&root, true, "/en/phase0-setup/01-intro").await,
            StatusCode::OK
        );

        let progress = progress::Progress::load(&root);
        assert!(progress.is_complete("phase0-setup/01-intro"));
        assert!(
            progress.is_complete("phase9/gone"),
            "an orphaned key survives the v1 migration too"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test]
    async fn self_check_refuses_a_path_that_is_not_a_lesson() {
        let response = app(default_root(), false)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/fa/self-check")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("path=docs&confidence=2"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
