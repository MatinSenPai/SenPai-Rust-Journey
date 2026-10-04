//! The whole stack (router, extractors, handlers, policy) through
//! `tower::ServiceExt::oneshot`: real requests, no socket.

use axum::body::{to_bytes, Body};
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::Router;
use p3_07_05_modelling_rbac_and_permissions::{app, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap()
    }
}

/// One request to `router`. `token` becomes `Authorization: Bearer <token>`;
/// a `Some` body is sent as JSON.
async fn send(
    router: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let request = match body {
        Some(value) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(value.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        headers,
        body: body.to_vec(),
    }
}

fn fresh() -> Router {
    app(AppState::seeded())
}

async fn text_of_review_1(router: &Router) -> String {
    let reply = send(router, "GET", "/reviews/1", None, None).await;
    reply.json()["text"].as_str().unwrap().to_string()
}

fn edit(text: &str) -> Option<Value> {
    Some(json!({ "text": text }))
}

mod authentication {
    use super::*;

    #[tokio::test]
    async fn reading_is_public() {
        let reply = send(&fresh(), "GET", "/reviews", None, None).await;
        assert_eq!(reply.status, StatusCode::OK);
        let reviews = reply.json();
        assert_eq!(reviews.as_array().unwrap().len(), 1);
        assert_eq!(reviews[0]["author"], 1);
    }

    #[tokio::test]
    async fn no_token_is_401_with_a_challenge() {
        let body = Some(json!({"anime": "Dandadan", "text": "wild"}));
        let reply = send(&fresh(), "POST", "/reviews", None, body).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
        assert_eq!(reply.headers[header::WWW_AUTHENTICATE], "Bearer");
        assert_eq!(reply.json(), json!({"error": "authentication required"}));
    }

    #[tokio::test]
    async fn an_unknown_token_is_401() {
        let reply = send(&fresh(), "GET", "/me", Some("nobody-token"), None).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn a_non_bearer_scheme_is_401() {
        let request = Request::builder()
            .uri("/me")
            .header(header::AUTHORIZATION, "Basic YWxpY2U6cHc=")
            .body(Body::empty())
            .unwrap();
        let reply = fresh().oneshot(request).await.unwrap();
        assert_eq!(reply.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn me_shows_the_caller_and_their_roles() {
        let reply = send(&fresh(), "GET", "/me", Some("carol-token"), None).await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(
            reply.json(),
            json!({"id": 3, "name": "carol", "roles": ["member", "moderator"]})
        );
    }
}

mod creating {
    use super::*;

    #[tokio::test]
    async fn a_member_creates_a_review_under_their_own_id() {
        let router = fresh();
        let body = Some(json!({"anime": "Dandadan", "text": "wild"}));
        let reply = send(&router, "POST", "/reviews", Some("bob-token"), body).await;
        assert_eq!(reply.status, StatusCode::CREATED);
        assert_eq!(
            reply.json(),
            json!({"id": 2, "author": 2, "anime": "Dandadan", "text": "wild"})
        );
    }

    #[tokio::test]
    async fn a_known_user_without_the_role_is_403_not_401() {
        let body = Some(json!({"anime": "Dandadan", "text": "wild"}));
        let reply = send(&fresh(), "POST", "/reviews", Some("mallory-token"), body).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
        assert_eq!(
            reply.json(),
            json!({"error": "you are not allowed to do that"})
        );
        assert!(reply.headers.get(header::WWW_AUTHENTICATE).is_none());
    }
}

mod ownership {
    use super::*;

    #[tokio::test]
    async fn the_author_edits_their_own_review() {
        let router = fresh();
        let reply = send(
            &router,
            "PATCH",
            "/reviews/1",
            Some("alice-token"),
            edit("again"),
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(text_of_review_1(&router).await, "again");
    }

    #[tokio::test]
    async fn another_member_cannot_edit_it_and_nothing_changes() {
        let router = fresh();
        let reply = send(
            &router,
            "PATCH",
            "/reviews/1",
            Some("bob-token"),
            edit("hacked"),
        )
        .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
        assert_eq!(text_of_review_1(&router).await, "Slow, quiet, perfect.");
    }

    #[tokio::test]
    async fn a_moderator_edits_anyones_review() {
        let router = fresh();
        let reply = send(
            &router,
            "PATCH",
            "/reviews/1",
            Some("carol-token"),
            edit("[removed]"),
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(text_of_review_1(&router).await, "[removed]");
    }

    #[tokio::test]
    async fn delete_follows_the_same_rule() {
        let router = fresh();
        let bob = send(&router, "DELETE", "/reviews/1", Some("bob-token"), None).await;
        assert_eq!(bob.status, StatusCode::FORBIDDEN);
        let alice = send(&router, "DELETE", "/reviews/1", Some("alice-token"), None).await;
        assert_eq!(alice.status, StatusCode::NO_CONTENT);
        let gone = send(&router, "GET", "/reviews/1", None, None).await;
        assert_eq!(gone.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn a_moderator_deletes_anyones_review() {
        let router = fresh();
        let reply = send(&router, "DELETE", "/reviews/1", Some("carol-token"), None).await;
        assert_eq!(reply.status, StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn checks_run_in_the_order_who_then_what_then_may_you() {
        let router = fresh();
        // No token: 401, even though review 99 does not exist.
        let anonymous = send(&router, "PATCH", "/reviews/99", None, edit("x")).await;
        assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
        // Token, missing review: 404.
        let missing = send(
            &router,
            "PATCH",
            "/reviews/99",
            Some("bob-token"),
            edit("x"),
        )
        .await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND);
        // Token, existing review that is not theirs: 403.
        let foreign = send(&router, "PATCH", "/reviews/1", Some("bob-token"), edit("x")).await;
        assert_eq!(foreign.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn authentication_is_checked_before_the_body_is_read() {
        let request = Request::builder()
            .method("PATCH")
            .uri("/reviews/1")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{ not json"))
            .unwrap();
        let reply = fresh().oneshot(request).await.unwrap();
        assert_eq!(reply.status(), StatusCode::UNAUTHORIZED);
    }
}

mod managing_users {
    use super::*;

    fn promote() -> Option<Value> {
        Some(json!({"role": "moderator"}))
    }

    #[tokio::test]
    async fn no_token_is_401() {
        let reply = send(&fresh(), "POST", "/users/2/roles", None, promote()).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn a_moderator_is_not_an_admin() {
        let reply = send(
            &fresh(),
            "POST",
            "/users/2/roles",
            Some("carol-token"),
            promote(),
        )
        .await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn the_permission_is_checked_before_the_user_is_looked_up() {
        let router = fresh();
        let member = send(
            &router,
            "POST",
            "/users/99/roles",
            Some("bob-token"),
            promote(),
        )
        .await;
        assert_eq!(member.status, StatusCode::FORBIDDEN);
        let admin = send(
            &router,
            "POST",
            "/users/99/roles",
            Some("dave-token"),
            promote(),
        )
        .await;
        assert_eq!(admin.status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn an_admin_promotes_and_the_new_role_works_at_once() {
        let router = fresh();
        let before = send(&router, "PATCH", "/reviews/1", Some("bob-token"), edit("x")).await;
        assert_eq!(before.status, StatusCode::FORBIDDEN);

        let reply = send(
            &router,
            "POST",
            "/users/2/roles",
            Some("dave-token"),
            promote(),
        )
        .await;
        assert_eq!(reply.status, StatusCode::OK);
        assert_eq!(
            reply.json(),
            json!({"id": 2, "name": "bob", "roles": ["member", "moderator"]})
        );

        let after = send(&router, "PATCH", "/reviews/1", Some("bob-token"), edit("x")).await;
        assert_eq!(after.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn an_unknown_role_name_is_rejected_by_the_body_parser() {
        let body = Some(json!({"role": "superuser"}));
        let reply = send(&fresh(), "POST", "/users/2/roles", Some("dave-token"), body).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    }
}
