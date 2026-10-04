//! Solution for 3.7.2 — Sessions vs. JWT: the real trade-off.
//!
//! A server-side session store with an injectable clock, a `Set-Cookie`
//! builder, and a small `axum` app that logs in, looks up, and revokes
//! sessions. Everything is in memory: no database, no JWT (3.7.3 owns that).

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::header::{COOKIE, ORIGIN, SET_COOKIE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;

// ---------------------------------------------------------------- clock

/// A source of "now", in whole seconds. Injected so tests can move time.
pub trait Clock: Send + Sync {
    fn now(&self) -> u64;
}

/// The real clock: seconds since the Unix epoch.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// A clock a test moves by hand.
pub struct ManualClock(AtomicU64);

impl ManualClock {
    pub fn new(start: u64) -> Self {
        ManualClock(AtomicU64::new(start))
    }
    pub fn advance(&self, secs: u64) {
        self.0.fetch_add(secs, Ordering::SeqCst);
    }
    pub fn set(&self, t: u64) {
        self.0.store(t, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

// ---------------------------------------------------------------- store

/// 16 random bytes from the operating system, as 32 lowercase hex characters.
pub fn new_session_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the OS random source failed");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Session {
    user: String,
    expires_at: u64,
}

/// Server-side sessions: the id lives in the cookie, everything else here.
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Session>>,
    clock: Arc<dyn Clock>,
    ttl_secs: u64,
}

impl SessionStore {
    pub fn new(clock: Arc<dyn Clock>, ttl_secs: u64) -> Self {
        SessionStore {
            sessions: Mutex::new(HashMap::new()),
            clock,
            ttl_secs,
        }
    }

    pub fn ttl_secs(&self) -> u64 {
        self.ttl_secs
    }

    /// How many entries the store holds, expired-but-not-yet-purged included.
    pub fn len(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn create(&self, user: &str) -> String {
        let id = new_session_id();
        let expires_at = self.clock.now().saturating_add(self.ttl_secs);
        let session = Session {
            user: user.to_string(),
            expires_at,
        };
        self.sessions.lock().unwrap().insert(id.clone(), session);
        id
    }

    pub fn lookup(&self, id: &str) -> Option<String> {
        let now = self.clock.now();
        let mut sessions = self.sessions.lock().unwrap();
        match sessions.get(id) {
            Some(s) if now < s.expires_at => Some(s.user.clone()),
            Some(_) => {
                sessions.remove(id);
                None
            }
            None => None,
        }
    }

    pub fn revoke(&self, id: &str) -> bool {
        let now = self.clock.now();
        let removed = self.sessions.lock().unwrap().remove(id);
        matches!(removed, Some(s) if now < s.expires_at)
    }

    pub fn revoke_all_for(&self, user: &str) -> usize {
        let now = self.clock.now();
        let mut sessions = self.sessions.lock().unwrap();
        let mut live = 0;
        sessions.retain(|_, s| {
            if s.user != user {
                return true;
            }
            if now < s.expires_at {
                live += 1;
            }
            false
        });
        live
    }

    /// Build rung: sliding expiry. A live session gets a fresh full lifetime.
    pub fn touch(&self, id: &str) -> bool {
        let now = self.clock.now();
        let mut sessions = self.sessions.lock().unwrap();
        match sessions.get_mut(id) {
            Some(s) if now < s.expires_at => {
                s.expires_at = now.saturating_add(self.ttl_secs);
                true
            }
            _ => false,
        }
    }

    /// Challenge rung: log out every device except `keep`.
    pub fn revoke_others(&self, user: &str, keep: &str) -> usize {
        let now = self.clock.now();
        let mut sessions = self.sessions.lock().unwrap();
        let mut live = 0;
        sessions.retain(|id, s| {
            if s.user != user || id == keep {
                return true;
            }
            if now < s.expires_at {
                live += 1;
            }
            false
        });
        live
    }
}

// -------------------------------------------------------------- cookies

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CookieOpts {
    pub max_age: Option<u64>,
    pub secure: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieError {
    Invalid,
    SameSiteNoneNeedsSecure,
}

fn bad_char(c: char) -> bool {
    !('\u{21}'..='\u{7e}').contains(&c) || matches!(c, ';' | ',' | '"' | '\\')
}

pub fn set_cookie(name: &str, value: &str, opts: &CookieOpts) -> Result<String, CookieError> {
    if name.is_empty() || name.contains('=') || name.chars().any(bad_char) {
        return Err(CookieError::Invalid);
    }
    if value.chars().any(bad_char) {
        return Err(CookieError::Invalid);
    }
    if opts.same_site == SameSite::None && !opts.secure {
        return Err(CookieError::SameSiteNoneNeedsSecure);
    }
    let mut out = format!("{name}={value}; Path=/; HttpOnly");
    if opts.secure {
        out.push_str("; Secure");
    }
    out.push_str(match opts.same_site {
        SameSite::Strict => "; SameSite=Strict",
        SameSite::Lax => "; SameSite=Lax",
        SameSite::None => "; SameSite=None",
    });
    if let Some(n) = opts.max_age {
        out.push_str(&format!("; Max-Age={n}"));
    }
    Ok(out)
}

pub fn expire_cookie(name: &str, opts: &CookieOpts) -> Result<String, CookieError> {
    let opts = CookieOpts {
        max_age: Some(0),
        ..*opts
    };
    set_cookie(name, "", &opts)
}

pub fn session_id_from_cookie_header<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(n, _)| *n == name)
        .map(|(_, v)| v)
        .filter(|v| !v.is_empty())
}

pub fn origin_ok(origin: Option<&str>, expected: &str) -> bool {
    match origin {
        None => true,
        Some(o) => o == expected,
    }
}

// ----------------------------------------------------------------- app

/// The name of the session cookie.
pub const COOKIE_NAME: &str = "sid";

/// Shared by every handler.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<SessionStore>,
    /// Whether to add `Secure` to cookies (true behind HTTPS).
    pub secure: bool,
    /// The one `Origin` allowed to make state-changing requests.
    pub origin: String,
}

fn cookie_opts(s: &AppState, max_age: Option<u64>) -> CookieOpts {
    CookieOpts {
        max_age,
        secure: s.secure,
        same_site: SameSite::Lax,
    }
}

fn sid_of(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(COOKIE)?.to_str().ok()?;
    session_id_from_cookie_header(raw, COOKIE_NAME).map(str::to_string)
}

fn origin_allowed(s: &AppState, headers: &HeaderMap) -> bool {
    let origin = headers.get(ORIGIN).and_then(|v| v.to_str().ok());
    origin_ok(origin, &s.origin)
}

fn forbidden() -> Response {
    (StatusCode::FORBIDDEN, "forbidden origin").into_response()
}

fn not_logged_in() -> Response {
    (StatusCode::UNAUTHORIZED, "not logged in").into_response()
}

async fn login(State(s): State<AppState>, headers: HeaderMap, body: String) -> Response {
    if !origin_allowed(&s, &headers) {
        return forbidden();
    }
    let user = body.trim();
    if user.is_empty() {
        return (StatusCode::BAD_REQUEST, "username required").into_response();
    }
    // Session fixation: never keep an id across a login.
    if let Some(old) = sid_of(&headers) {
        s.store.revoke(&old);
    }
    let id = s.store.create(user);
    let cookie = set_cookie(COOKIE_NAME, &id, &cookie_opts(&s, Some(s.store.ttl_secs()))).unwrap();
    ([(SET_COOKIE, cookie)], format!("welcome {user}")).into_response()
}

async fn me(State(s): State<AppState>, headers: HeaderMap) -> Response {
    match sid_of(&headers).and_then(|id| s.store.lookup(&id)) {
        Some(user) => user.into_response(),
        None => not_logged_in(),
    }
}

async fn logout(State(s): State<AppState>, headers: HeaderMap) -> Response {
    if !origin_allowed(&s, &headers) {
        return forbidden();
    }
    if let Some(id) = sid_of(&headers) {
        s.store.revoke(&id);
    }
    let cookie = expire_cookie(COOKIE_NAME, &cookie_opts(&s, None)).unwrap();
    (StatusCode::NO_CONTENT, [(SET_COOKIE, cookie)]).into_response()
}

async fn logout_all(State(s): State<AppState>, headers: HeaderMap) -> Response {
    if !origin_allowed(&s, &headers) {
        return forbidden();
    }
    let Some(user) = sid_of(&headers).and_then(|id| s.store.lookup(&id)) else {
        return not_logged_in();
    };
    s.store.revoke_all_for(&user);
    let cookie = expire_cookie(COOKIE_NAME, &cookie_opts(&s, None)).unwrap();
    (StatusCode::NO_CONTENT, [(SET_COOKIE, cookie)]).into_response()
}

/// The router: `POST /login`, `GET /me`, `POST /logout`, `POST /logout-all`.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/me", get(me))
        .route("/logout", post(logout))
        .route("/logout-all", post(logout_all))
        .with_state(state)
}
