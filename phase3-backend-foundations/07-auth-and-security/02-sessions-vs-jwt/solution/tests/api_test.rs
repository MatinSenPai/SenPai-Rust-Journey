//! The whole app through `tower::ServiceExt::oneshot`: real requests, no socket.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{header, HeaderMap, Request, StatusCode};
use axum::Router;
use tower::ServiceExt;

use p3_07_02_sessions_vs_jwt_solution::{app, AppState, ManualClock, SessionStore};

const ORIGIN: &str = "http://localhost:3000";

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl Reply {
    /// The value of the `sid` cookie this reply set (without attributes).
    fn sid(&self) -> Option<String> {
        let c = self.headers.get(header::SET_COOKIE)?.to_str().ok()?;
        let first = c.split(';').next()?;
        first.strip_prefix("sid=").map(str::to_string)
    }
    fn set_cookie(&self) -> String {
        self.headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .to_string()
    }
}

struct World {
    clock: Arc<ManualClock>,
    router: Router,
}

fn world() -> World {
    let clock = Arc::new(ManualClock::new(5_000));
    let store = Arc::new(SessionStore::new(clock.clone(), 3_600));
    let state = AppState {
        store,
        secure: false,
        origin: ORIGIN.to_string(),
    };
    World {
        clock,
        router: app(state),
    }
}

async fn send(
    w: &World,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    origin: Option<&str>,
    body: &str,
) -> Reply {
    let mut b = Request::builder().method(method).uri(uri);
    if let Some(c) = cookie {
        b = b.header(header::COOKIE, c);
    }
    if let Some(o) = origin {
        b = b.header(header::ORIGIN, o);
    }
    let req = b.body(Body::from(body.to_string())).unwrap();
    let res = w.router.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    Reply {
        status,
        headers,
        body: String::from_utf8(bytes.to_vec()).unwrap(),
    }
}

async fn login(w: &World, user: &str) -> String {
    let r = send(w, "POST", "/login", None, None, user).await;
    assert_eq!(r.status, StatusCode::OK);
    format!("sid={}", r.sid().unwrap())
}

#[tokio::test]
async fn login_answers_200_with_a_hardened_session_cookie() {
    let w = world();
    let r = send(&w, "POST", "/login", None, None, "matin").await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body, "welcome matin");
    let c = r.set_cookie();
    let id = r.sid().unwrap();
    assert_eq!(id.len(), 32);
    assert_eq!(
        c,
        format!("sid={id}; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600")
    );
}

#[tokio::test]
async fn login_trims_the_body_and_rejects_an_empty_username() {
    let w = world();
    let r = send(&w, "POST", "/login", None, None, "  matin\n").await;
    assert_eq!(r.body, "welcome matin");
    let r = send(&w, "POST", "/login", None, None, "   ").await;
    assert_eq!(r.status, StatusCode::BAD_REQUEST);
    assert_eq!(r.body, "username required");
    assert!(r.headers.get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn me_returns_the_user_for_a_live_session() {
    let w = world();
    let cookie = login(&w, "matin").await;
    let r = send(&w, "GET", "/me", Some(&cookie), None, "").await;
    assert_eq!(r.status, StatusCode::OK);
    assert_eq!(r.body, "matin");
}

#[tokio::test]
async fn me_without_or_with_a_forged_cookie_is_401() {
    let w = world();
    let r = send(&w, "GET", "/me", None, None, "").await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(r.body, "not logged in");
    let r = send(
        &w,
        "GET",
        "/me",
        Some("sid=0123456789abcdef0123456789abcdef"),
        None,
        "",
    )
    .await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_finds_the_session_cookie_among_other_cookies() {
    let w = world();
    let cookie = login(&w, "matin").await;
    let header = format!("theme=dark; {cookie}; lang=fa");
    let r = send(&w, "GET", "/me", Some(&header), None, "").await;
    assert_eq!(r.body, "matin");
}

#[tokio::test]
async fn a_session_stops_working_when_its_lifetime_is_over() {
    let w = world();
    let cookie = login(&w, "matin").await;
    w.clock.advance(3_599);
    assert_eq!(
        send(&w, "GET", "/me", Some(&cookie), None, "").await.status,
        StatusCode::OK
    );
    w.clock.advance(1);
    assert_eq!(
        send(&w, "GET", "/me", Some(&cookie), None, "").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn logout_revokes_on_the_server_so_a_copied_cookie_dies_too() {
    let w = world();
    let cookie = login(&w, "matin").await;
    let r = send(&w, "POST", "/logout", Some(&cookie), None, "").await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        r.set_cookie(),
        "sid=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"
    );
    // The attacker kept a copy of the old cookie: still refused.
    let r = send(&w, "GET", "/me", Some(&cookie), None, "").await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_without_a_session_is_still_204() {
    let w = world();
    let r = send(&w, "POST", "/logout", None, None, "").await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn logging_in_with_an_attacker_chosen_id_never_authenticates_that_id() {
    let w = world();
    let planted = "sid=attackerchosenid";
    let r = send(&w, "POST", "/login", Some(planted), None, "matin").await;
    let fresh = r.sid().unwrap();
    assert_ne!(fresh, "attackerchosenid");
    let r = send(&w, "GET", "/me", Some(planted), None, "").await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logging_in_again_revokes_the_session_the_request_arrived_with() {
    let w = world();
    let first = login(&w, "matin").await;
    let r = send(&w, "POST", "/login", Some(&first), None, "matin").await;
    let second = format!("sid={}", r.sid().unwrap());
    assert_ne!(first, second);
    assert_eq!(
        send(&w, "GET", "/me", Some(&first), None, "").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&w, "GET", "/me", Some(&second), None, "").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn logout_all_ends_every_device_of_the_user_and_nobody_elses() {
    let w = world();
    let laptop = login(&w, "matin").await;
    let phone = login(&w, "matin").await;
    let sara = login(&w, "sara").await;
    let r = send(&w, "POST", "/logout-all", Some(&laptop), None, "").await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
    assert_eq!(
        r.set_cookie(),
        "sid=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"
    );
    assert_eq!(
        send(&w, "GET", "/me", Some(&phone), None, "").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        send(&w, "GET", "/me", Some(&sara), None, "").await.body,
        "sara"
    );
}

#[tokio::test]
async fn logout_all_needs_a_login() {
    let w = world();
    let r = send(&w, "POST", "/logout-all", None, None, "").await;
    assert_eq!(r.status, StatusCode::UNAUTHORIZED);
    assert_eq!(r.body, "not logged in");
}

#[tokio::test]
async fn a_cross_origin_post_is_refused_even_with_a_valid_cookie() {
    let w = world();
    let cookie = login(&w, "matin").await;
    for path in ["/logout", "/logout-all"] {
        let r = send(
            &w,
            "POST",
            path,
            Some(&cookie),
            Some("http://evil.example"),
            "",
        )
        .await;
        assert_eq!(r.status, StatusCode::FORBIDDEN, "{path}");
        assert_eq!(r.body, "forbidden origin");
    }
    let r = send(
        &w,
        "POST",
        "/login",
        None,
        Some("http://evil.example"),
        "matin",
    )
    .await;
    assert_eq!(r.status, StatusCode::FORBIDDEN);
    // The session survived the forged logouts.
    assert_eq!(
        send(&w, "GET", "/me", Some(&cookie), None, "").await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn the_expected_origin_is_accepted() {
    let w = world();
    let cookie = login(&w, "matin").await;
    let r = send(&w, "POST", "/logout", Some(&cookie), Some(ORIGIN), "").await;
    assert_eq!(r.status, StatusCode::NO_CONTENT);
}
