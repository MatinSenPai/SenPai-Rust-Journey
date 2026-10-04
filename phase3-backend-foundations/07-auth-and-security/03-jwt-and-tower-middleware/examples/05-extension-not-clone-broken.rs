//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo build --example 05-extension-not-clone-broken --features broken`
//! and read the error.

use axum::extract::Extension;
use axum::routing::get;
use axum::Router;

// Forgot `#[derive(Clone)]`: an `Extension<T>` extractor needs `T: Clone`.
struct AuthUser(String);

async fn whoami(Extension(user): Extension<AuthUser>) -> String {
    user.0
}

fn main() {
    let _router: Router = Router::new().route("/whoami", get(whoami));
}
