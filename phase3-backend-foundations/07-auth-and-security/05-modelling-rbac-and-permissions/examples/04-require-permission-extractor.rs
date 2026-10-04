//! A `RequirePermission` extractor: 401 for "who are you?", 403 for "not you".
//! Run: cargo run -p p3-07-05-modelling-rbac-and-permissions --example 04-require-permission-extractor

use std::collections::HashSet;
use std::marker::PhantomData;

use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use tower::ServiceExt;

/// A type-level permission name, so a handler signature can say what it needs.
trait Needs: Send + Sync + 'static {
    const PERMISSION: &'static str;
}

struct ManageUsers;
impl Needs for ManageUsers {
    const PERMISSION: &'static str = "manage_users";
}

struct Require<P>(PhantomData<P>);

impl<P: Needs, S: Send + Sync> FromRequestParts<S> for Require<P> {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, StatusCode> {
        // Stand-in for a real token lookup: the header lists the caller's permissions.
        let header = parts
            .headers
            .get("x-permissions")
            .ok_or(StatusCode::UNAUTHORIZED)?;
        let held: HashSet<&str> = header.to_str().unwrap_or("").split(',').collect();
        if held.contains(P::PERMISSION) {
            Ok(Require(PhantomData))
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

async fn admin_page(_: Require<ManageUsers>) -> &'static str {
    "welcome, admin"
}

async fn call(app: &Router, permissions: Option<&str>) -> StatusCode {
    let mut request = Request::builder().uri("/admin");
    if let Some(value) = permissions {
        request = request.header("x-permissions", value);
    }
    let request = request.body(Body::empty()).unwrap();
    app.clone().oneshot(request).await.unwrap().status()
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/admin", get(admin_page));
    let tries = [None, Some("create"), Some("create,manage_users")];
    for permissions in tries {
        let status = call(&app, permissions).await;
        println!(
            "x-permissions: {:<20} -> {status}",
            permissions.unwrap_or("(absent)")
        );
    }
}
