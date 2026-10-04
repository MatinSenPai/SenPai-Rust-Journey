//! Exercises for 3.7.2 — Sessions vs. JWT: the real trade-off.
//!
//! Server-side sessions, in memory: a `SessionStore` with an injectable
//! clock, a pure `Set-Cookie` builder, a `Cookie` header parser, an `Origin`
//! check, and a small `axum` app that wires them together. No database and
//! no JWT (3.7.3 owns that). `tests/store_test.rs` and `tests/cookie_test.rs`
//! check the plain functions; `tests/api_test.rs` checks the whole app
//! through `tower::ServiceExt::oneshot`.

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

/// The real clock: seconds since the Unix epoch. Given to you.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// A clock a test moves by hand. Given to you.
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

/// 16 random bytes from the operating system, as 32 lowercase hex
/// characters. Given to you.
pub fn new_session_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the OS random source failed");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

struct Session {
    user: String,
    /// The first second at which the session is dead: it is live while
    /// `now < expires_at`.
    expires_at: u64,
}

/// Server-side sessions: the id lives in the cookie, everything else here.
///
/// Throughout, a session is **live** at time `now` when `now < expires_at`,
/// where `expires_at` is the creation time plus the store's lifetime
/// (`ttl_secs`). Read the clock through `self.clock`.
pub struct SessionStore {
    sessions: Mutex<HashMap<String, Session>>,
    clock: Arc<dyn Clock>,
    ttl_secs: u64,
}

impl SessionStore {
    /// An empty store whose sessions live `ttl_secs` seconds. Given to you.
    pub fn new(clock: Arc<dyn Clock>, ttl_secs: u64) -> Self {
        SessionStore {
            sessions: Mutex::new(HashMap::new()),
            clock,
            ttl_secs,
        }
    }

    /// The session lifetime this store was built with. Given to you.
    pub fn ttl_secs(&self) -> u64 {
        self.ttl_secs
    }

    /// How many entries the store holds, expired-but-not-yet-purged
    /// included. Given to you.
    pub fn len(&self) -> usize {
        self.sessions.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Starts a session for `user` and returns its id.
    ///
    /// - The id is a fresh `new_session_id()`: every call gives a different
    ///   id, even for the same user (one user, many devices).
    /// - The session expires `ttl_secs` seconds after the clock's current
    ///   time (use a saturating add so a huge lifetime cannot overflow).
    pub fn create(&self, user: &str) -> String {
        todo!("store a new live session for this user and return its fresh id")
    }

    /// The user the session belongs to, or `None`.
    ///
    /// - An unknown id is `None`.
    /// - A live session gives `Some(user)`.
    /// - An expired session is `None` **and is removed from the store**, so
    ///   `len()` drops by one. Nothing else purges expired entries.
    pub fn lookup(&self, id: &str) -> Option<String> {
        todo!("the user of a live session, forgetting the session if it has expired")
    }

    /// Ends one session on the server. Returns `true` only if the session
    /// was live when it was removed.
    ///
    /// - An unknown id, or an id already revoked: `false`.
    /// - An expired session: it is still removed from the store, but the
    ///   answer is `false`.
    pub fn revoke(&self, id: &str) -> bool {
        todo!("remove this session; say whether it was still live")
    }

    /// Ends every session of `user` ("log out everywhere") and returns how
    /// many of them were live. Expired sessions of that user are removed
    /// too but not counted. Other users' sessions are untouched. A user
    /// with no sessions gives `0`.
    pub fn revoke_all_for(&self, user: &str) -> usize {
        todo!("remove every session of this user; count the ones that were live")
    }

    /// Build rung: sliding expiry. If `id` names a live session, give it a
    /// fresh full lifetime counted from now (`ttl_secs` seconds after the
    /// clock's current time) and return `true`. An unknown or expired id
    /// returns `false` and is not resurrected.
    pub fn touch(&self, id: &str) -> bool {
        todo!("push a live session's expiry a full lifetime from now; false if not live")
    }

    /// Challenge rung: "log out every other device". Removes every session
    /// of `user` except the one named `keep`, and returns how many of the
    /// removed ones were live. Other users and the `keep` session are
    /// untouched.
    pub fn revoke_others(&self, user: &str, keep: &str) -> usize {
        todo!("remove all of this user's sessions except the kept one; count the live ones")
    }
}

// -------------------------------------------------------------- cookies

/// The `SameSite` attribute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// The knobs `set_cookie` needs. `HttpOnly` and `Path=/` are not knobs:
/// a session cookie always has both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CookieOpts {
    pub max_age: Option<u64>,
    pub secure: bool,
    pub same_site: SameSite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CookieError {
    /// A name or value contains a character that is not allowed.
    Invalid,
    /// `SameSite=None` without `Secure`: browsers reject that cookie.
    SameSiteNoneNeedsSecure,
}

/// Builds the value of a `Set-Cookie` header.
///
/// The result is `name=value` followed by these attributes, each preceded
/// by `"; "`, in exactly this order:
///
/// 1. `Path=/` (always)
/// 2. `HttpOnly` (always)
/// 3. `Secure` (only if `opts.secure`)
/// 4. `SameSite=Strict`, `SameSite=Lax` or `SameSite=None`
/// 5. `Max-Age=<n>` (only if `opts.max_age` is `Some(n)`; `Some(0)` is
///    written out as `Max-Age=0`)
///
/// For example, `set_cookie("sid", "abc", ..)` with `Lax`, not secure, no
/// max age gives `sid=abc; Path=/; HttpOnly; SameSite=Lax`.
///
/// Errors, checked in this order:
///
/// - `Err(CookieError::Invalid)` if `name` is empty, contains `=`, or if
///   `name` or `value` contains any character outside printable ASCII
///   `!`..=`~` (so no space, no control characters, no non-ASCII), or any
///   of `;` `,` `"` `\`. An empty `value` is allowed. This blocks a value
///   from smuggling in extra attributes or a second header.
/// - `Err(CookieError::SameSiteNoneNeedsSecure)` if `same_site` is `None`
///   and `secure` is `false`.
pub fn set_cookie(name: &str, value: &str, opts: &CookieOpts) -> Result<String, CookieError> {
    todo!("the Set-Cookie value described above, or the first error that applies")
}

/// A `Set-Cookie` value that tells the browser to delete the cookie: the
/// same as `set_cookie(name, "", ..)` with `opts.max_age` replaced by
/// `Some(0)`. Same errors as `set_cookie`.
pub fn expire_cookie(name: &str, opts: &CookieOpts) -> Result<String, CookieError> {
    todo!("a Set-Cookie value with an empty value and Max-Age=0")
}

/// Finds the cookie called `name` in a `Cookie` request header.
///
/// - The header is `a=1; sid=abc; b=2`: pairs separated by `;`, with
///   optional spaces around each pair (trim them).
/// - Split each pair at its **first** `=`; the value keeps any later `=`
///   (`sid=a=b` gives `a=b`). A piece with no `=` is skipped.
/// - The name must match exactly and case-sensitively (`xsid` and `SID` do
///   not match `sid`).
/// - If several pairs match, the first wins.
/// - An empty value counts as absent: `None`.
pub fn session_id_from_cookie_header<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    todo!("the value of the first cookie with this exact name, if it is non-empty")
}

/// The CSRF defence used by the app: does this request's `Origin` header
/// match the one origin we serve?
///
/// - No `Origin` header (`None`) passes: not a browser form or fetch.
/// - `Some(o)` passes only if `o` equals `expected` exactly. No trailing
///   slash is tolerated, and `"null"` fails.
pub fn origin_ok(origin: Option<&str>, expected: &str) -> bool {
    todo!("true if there is no Origin or it is exactly the expected one")
}

// ----------------------------------------------------------------- app
// Everything below is given to you: read it, then make it work by
// finishing the functions above.

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
