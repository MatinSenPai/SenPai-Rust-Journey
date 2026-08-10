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

mod page;
mod progress;
mod render;
mod style;
mod tree;

use std::path::{Path, PathBuf};
use std::process::Command;

use axum::extract::{Path as UrlPath, State};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Form, Router};
use serde::Deserialize;

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

    if tree::build(&root).is_none() {
        eprintln!(
            "course-ui: no markdown found under {} — is that the repo root?",
            root.display()
        );
        std::process::exit(1);
    }

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/style.css", get(stylesheet))
        .route("/mark", post(mark))
        .route("/{*path}", get(node_handler))
        .with_state(AppState { root: root.clone() });

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

// ---------------------------------------------------------------- handlers

async fn root_handler(State(state): State<AppState>) -> Response {
    render_path(&state.root, "")
}

async fn node_handler(State(state): State<AppState>, UrlPath(path): UrlPath<String>) -> Response {
    render_path(&state.root, path.trim_matches('/'))
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        style::CSS,
    )
}

#[derive(Deserialize)]
struct MarkForm {
    path: String,
    complete: String,
}

async fn mark(State(state): State<AppState>, Form(form): Form<MarkForm>) -> Response {
    let Some(tree) = tree::build(&state.root) else {
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

    Redirect::to(&format!("/{}", form.path)).into_response()
}

/// The tree is rebuilt per request — a few milliseconds, and it means an edited
/// README or a new lesson directory shows up on refresh with no cache to bust.
fn render_path(root: &Path, path: &str) -> Response {
    let Some(tree) = tree::build(root) else {
        return (StatusCode::INTERNAL_SERVER_ERROR, "no content").into_response();
    };
    let progress = progress::Progress::load(root);

    match tree.find(path) {
        Some(node) => Html(page::render_node(root, &tree, node, &progress)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Html(page::render_missing(&tree, path, &progress)),
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
