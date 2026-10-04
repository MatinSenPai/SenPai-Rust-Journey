//! Exercises for 3.7.5 — Modelling RBAC and permissions.
//!
//! Three layers, each a plain function or type before it is HTTP:
//!
//! 1. `Policy` — which role grants which permission. No users, no requests.
//! 2. `can` — one pure yes/no question: may this user do this action to
//!    this review? Ownership lives here.
//! 3. The `axum` edge: `Authenticated` answers "who are you?" (`401`),
//!    `RequirePermission<P>` answers "may your roles do this at all?"
//!    (`403`), and each handler asks `can` once it has loaded the review.
//!
//! Storage is in memory. `AppState::seeded()` gives five users and one
//! review so the tests and the `curl` transcripts have something to talk to.

use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use axum::extract::{FromRequestParts, Path, State};
use axum::http::{header, request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

/// A user's id. Reviews remember their author by it.
pub type UserId = u64;

/// One thing a role may allow. `Own` permissions apply only to a review the
/// user wrote; `Any` permissions apply to every review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    ReviewCreate,
    ReviewEditOwn,
    ReviewEditAny,
    ReviewDeleteOwn,
    ReviewDeleteAny,
    ManageUsers,
}

/// A named bundle of permissions. On the wire it is snake_case: `"member"`,
/// `"moderator"`, `"admin"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Member,
    Moderator,
    Admin,
}

impl Role {
    /// The wire name: `"member"`, `"moderator"` or `"admin"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Member => "member",
            Role::Moderator => "moderator",
            Role::Admin => "admin",
        }
    }
}

/// What a user is trying to do to a review.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Create,
    Edit,
    Delete,
}

/// An authenticated user and the roles they hold. A user with no roles is
/// legal: they are known, and allowed nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub roles: HashSet<Role>,
}

impl User {
    /// A user holding exactly the given roles (duplicates collapse).
    pub fn new(id: UserId, name: &str, roles: &[Role]) -> User {
        User {
            id,
            name: name.to_string(),
            roles: roles.iter().copied().collect(),
        }
    }
}

/// The role table: for each role, the set of permissions it grants.
#[derive(Debug, Clone, Default)]
pub struct Policy {
    grants: HashMap<Role, HashSet<Permission>>,
}

impl Policy {
    /// An empty policy: no role grants anything.
    pub fn new() -> Policy {
        Policy::default()
    }

    /// Adds `permissions` to what `role` grants. Calling it twice for the
    /// same role adds to the set; it never replaces it. Given to you.
    pub fn grant(mut self, role: Role, permissions: &[Permission]) -> Policy {
        self.grants
            .entry(role)
            .or_default()
            .extend(permissions.iter().copied());
        self
    }

    /// The policy the lesson's server runs: a member writes and manages
    /// their own reviews, a moderator may edit and delete anyone's, an admin
    /// manages users. The roles are additive, so a moderator who should also
    /// write reviews holds `Member` too. Given to you.
    pub fn standard() -> Policy {
        use Permission::*;
        Policy::new()
            .grant(
                Role::Member,
                &[ReviewCreate, ReviewEditOwn, ReviewDeleteOwn],
            )
            .grant(Role::Moderator, &[ReviewEditAny, ReviewDeleteAny])
            .grant(Role::Admin, &[ManageUsers])
    }

    /// `true` when at least one of the user's roles grants `permission`.
    ///
    /// The user's permissions are the union over all their roles. A role the
    /// policy has no entry for grants nothing, and a user with no roles is
    /// allowed nothing. Ownership is not considered here; that is [`can`].
    pub fn allows(&self, user: &User, permission: Permission) -> bool {
        user.roles.iter().any(|role| {
            self.grants
                .get(role)
                .is_some_and(|set| set.contains(&permission))
        })
    }
}

/// May `user` perform `action`? `owner` is the author of the review being
/// acted on, or `None` when there is no review yet (a create) or the caller
/// does not know it.
///
/// - `Create`: allowed when the user holds `ReviewCreate`. `owner` is ignored.
/// - `Edit`: allowed when the user holds `ReviewEditAny`, or when `owner` is
///   `Some` of the user's own id and the user holds `ReviewEditOwn`.
/// - `Delete`: the same rule with `ReviewDeleteAny` and `ReviewDeleteOwn`.
///
/// Holding an `Own` permission never helps with someone else's review, and
/// `owner: None` never matches anybody. Pure: no I/O, no state.
pub fn can(policy: &Policy, user: &User, action: Action, owner: Option<UserId>) -> bool {
    let is_owner = owner == Some(user.id);
    match action {
        Action::Create => policy.allows(user, Permission::ReviewCreate),
        Action::Edit => {
            policy.allows(user, Permission::ReviewEditAny)
                || (is_owner && policy.allows(user, Permission::ReviewEditOwn))
        }
        Action::Delete => {
            policy.allows(user, Permission::ReviewDeleteAny)
                || (is_owner && policy.allows(user, Permission::ReviewDeleteOwn))
        }
    }
}

/// One review. As JSON: `{"id":1,"author":1,"anime":"Frieren","text":"..."}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Review {
    pub id: u64,
    pub author: UserId,
    pub anime: String,
    pub text: String,
}

/// The body of `POST /reviews`.
#[derive(Debug, Deserialize)]
pub struct NewReview {
    pub anime: String,
    pub text: String,
}

/// The body of `PATCH /reviews/{id}`.
#[derive(Debug, Deserialize)]
pub struct EditReview {
    pub text: String,
}

/// The body of `POST /users/{id}/roles`: `{"role":"moderator"}`.
#[derive(Debug, Deserialize)]
pub struct GrantRole {
    pub role: Role,
}

/// A user as the API shows them: `{"id":3,"name":"carol","roles":["member","moderator"]}`,
/// roles sorted by name.
#[derive(Debug, Serialize)]
pub struct UserView {
    pub id: UserId,
    pub name: String,
    pub roles: Vec<&'static str>,
}

impl From<&User> for UserView {
    fn from(user: &User) -> UserView {
        let mut roles: Vec<&'static str> = user.roles.iter().map(|r| r.as_str()).collect();
        roles.sort();
        UserView {
            id: user.id,
            name: user.name.clone(),
            roles,
        }
    }
}

/// Everything that can go wrong at the HTTP edge. Given to you; the table is
/// the contract the tests check.
///
/// | Variant | Status | Extra header | JSON body |
/// |---|---|---|---|
/// | `Unauthenticated` | `401` | `www-authenticate: Bearer` | `{"error":"authentication required"}` |
/// | `Forbidden` | `403` | none | `{"error":"you are not allowed to do that"}` |
/// | `NotFound` | `404` | none | `{"error":"not found"}` |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiError {
    Unauthenticated,
    Forbidden,
    NotFound,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                [(header::WWW_AUTHENTICATE, "Bearer")],
                Json(serde_json::json!({"error": "authentication required"})),
            )
                .into_response(),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "you are not allowed to do that"})),
            )
                .into_response(),
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "not found"})),
            )
                .into_response(),
        }
    }
}

#[derive(Default)]
struct Db {
    users: HashMap<UserId, User>,
    tokens: HashMap<String, UserId>,
    reviews: HashMap<u64, Review>,
    next_review_id: u64,
}

/// The shared application state: the policy plus an in-memory database of
/// users, bearer tokens and reviews. Cloning is cheap (two `Arc`s).
#[derive(Clone)]
pub struct AppState {
    policy: Arc<Policy>,
    db: Arc<Mutex<Db>>,
}

impl AppState {
    /// The lesson's world, with `Policy::standard()`:
    ///
    /// | id | name | roles | bearer token |
    /// |---|---|---|---|
    /// | 1 | alice | member | `alice-token` |
    /// | 2 | bob | member | `bob-token` |
    /// | 3 | carol | member, moderator | `carol-token` |
    /// | 4 | dave | member, moderator, admin | `dave-token` |
    /// | 5 | mallory | none | `mallory-token` |
    ///
    /// and one review: id 1, written by alice, anime `Frieren`, text
    /// `Slow, quiet, perfect.` The next review created gets id 2.
    pub fn seeded() -> AppState {
        use Role::*;
        let people = [
            (1, "alice", vec![Member]),
            (2, "bob", vec![Member]),
            (3, "carol", vec![Member, Moderator]),
            (4, "dave", vec![Member, Moderator, Admin]),
            (5, "mallory", vec![]),
        ];
        let mut db = Db::default();
        for (id, name, roles) in people {
            db.users.insert(id, User::new(id, name, &roles));
            db.tokens.insert(format!("{name}-token"), id);
        }
        db.reviews.insert(
            1,
            Review {
                id: 1,
                author: 1,
                anime: "Frieren".to_string(),
                text: "Slow, quiet, perfect.".to_string(),
            },
        );
        db.next_review_id = 2;
        AppState {
            policy: Arc::new(Policy::standard()),
            db: Arc::new(Mutex::new(db)),
        }
    }

    /// The policy this state enforces.
    pub fn policy(&self) -> &Policy {
        &self.policy
    }

    /// The user a bearer token belongs to, with their roles as they are
    /// right now, or `None` for an unknown token.
    pub fn user_for_token(&self, token: &str) -> Option<User> {
        let db = self.db.lock().unwrap();
        let id = db.tokens.get(token)?;
        db.users.get(id).cloned()
    }
}

/// The caller, identified from `Authorization: Bearer <token>`. Given to you.
///
/// Rejects with [`ApiError::Unauthenticated`] (`401`) when the header is
/// missing, is not `Bearer <something>`, or names a token nobody holds. In a
/// real service this is where a verified JWT would be turned into a user; the
/// token table stands in for that so this lesson runs on its own.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authenticated(pub User);

impl FromRequestParts<AppState> for Authenticated {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or(ApiError::Unauthenticated)?;
        state
            .user_for_token(token.trim())
            .map(Authenticated)
            .ok_or(ApiError::Unauthenticated)
    }
}

/// A type-level name for one [`Permission`], so a handler's signature can say
/// which one it needs: `RequirePermission<CanManageUsers>`.
pub trait RequiredPermission: Send + Sync + 'static {
    const PERMISSION: Permission;
}

/// Marker for [`Permission::ManageUsers`].
pub struct CanManageUsers;

impl RequiredPermission for CanManageUsers {
    const PERMISSION: Permission = Permission::ManageUsers;
}

/// The caller, proven to hold permission `P` through at least one of their
/// roles. This is the coarse, role-level gate: it cannot know which review
/// the request is about, so ownership is checked later with [`can`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequirePermission<P> {
    pub user: User,
    _marker: PhantomData<P>,
}

/// Extracts the caller and checks one permission.
///
/// - No usable credentials (everything [`Authenticated`] rejects): the same
///   rejection, [`ApiError::Unauthenticated`], so `401`.
/// - Known user whose roles do not grant `P::PERMISSION` (checked with
///   [`Policy::allows`] on `state.policy()`): [`ApiError::Forbidden`], so `403`.
/// - Otherwise `Ok`, carrying the user.
impl<P: RequiredPermission> FromRequestParts<AppState> for RequirePermission<P> {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Authenticated(user) = Authenticated::from_request_parts(parts, state).await?;
        if state.policy().allows(&user, P::PERMISSION) {
            Ok(RequirePermission {
                user,
                _marker: PhantomData,
            })
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

/// `GET /me` — the caller as a [`UserView`]. `401` without a valid token.
pub async fn me(Authenticated(user): Authenticated) -> Json<UserView> {
    Json(UserView::from(&user))
}

/// `GET /reviews` — every review, smallest id first. Public.
pub async fn list_reviews(State(state): State<AppState>) -> Json<Vec<Review>> {
    let db = state.db.lock().unwrap();
    let mut reviews: Vec<Review> = db.reviews.values().cloned().collect();
    reviews.sort_by_key(|review| review.id);
    Json(reviews)
}

/// `GET /reviews/{id}` — one review, or `404`. Public.
pub async fn get_review(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Review>, ApiError> {
    let db = state.db.lock().unwrap();
    db.reviews
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or(ApiError::NotFound)
}

/// `POST /reviews` — `201` and the new review, written by the caller.
/// `401` without a token, `403` when the caller may not create.
pub async fn create_review(
    Authenticated(user): Authenticated,
    State(state): State<AppState>,
    Json(input): Json<NewReview>,
) -> Result<(StatusCode, Json<Review>), ApiError> {
    if !can(state.policy(), &user, Action::Create, None) {
        return Err(ApiError::Forbidden);
    }
    let mut db = state.db.lock().unwrap();
    let id = db.next_review_id;
    db.next_review_id += 1;
    let review = Review {
        id,
        author: user.id,
        anime: input.anime,
        text: input.text,
    };
    db.reviews.insert(id, review.clone());
    Ok((StatusCode::CREATED, Json(review)))
}

/// `PATCH /reviews/{id}` — replaces the review's text. The order of the
/// checks is part of the contract: `401` (who?), then `404` (what?), then
/// `403` (may you?). `403` unless `can(.., Edit, Some(author))`.
pub async fn update_review(
    Authenticated(user): Authenticated,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<EditReview>,
) -> Result<Json<Review>, ApiError> {
    let mut db = state.db.lock().unwrap();
    let review = db.reviews.get_mut(&id).ok_or(ApiError::NotFound)?;
    if !can(state.policy(), &user, Action::Edit, Some(review.author)) {
        return Err(ApiError::Forbidden);
    }
    review.text = input.text;
    Ok(Json(review.clone()))
}

/// `DELETE /reviews/{id}` — `204`. Same check order as `update_review`, with
/// `Action::Delete`.
pub async fn delete_review(
    Authenticated(user): Authenticated,
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    let mut db = state.db.lock().unwrap();
    let review = db.reviews.get(&id).ok_or(ApiError::NotFound)?;
    if !can(state.policy(), &user, Action::Delete, Some(review.author)) {
        return Err(ApiError::Forbidden);
    }
    db.reviews.remove(&id);
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /users/{id}/roles` — gives a user one more role and returns them as
/// a [`UserView`]. The `RequirePermission<CanManageUsers>` argument is the
/// whole authorization story: `401` without a token, `403` for anyone whose
/// roles lack `ManageUsers`, `404` for an unknown user.
pub async fn grant_role(
    _admin: RequirePermission<CanManageUsers>,
    State(state): State<AppState>,
    Path(id): Path<UserId>,
    Json(input): Json<GrantRole>,
) -> Result<Json<UserView>, ApiError> {
    let mut db = state.db.lock().unwrap();
    let user = db.users.get_mut(&id).ok_or(ApiError::NotFound)?;
    user.roles.insert(input.role);
    Ok(Json(UserView::from(&*user)))
}

/// The router, with `state` attached.
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/me", get(me))
        .route("/reviews", get(list_reviews).post(create_review))
        .route(
            "/reviews/{id}",
            get(get_review).patch(update_review).delete(delete_review),
        )
        .route("/users/{id}/roles", post(grant_role))
        .with_state(state)
}

/// The Challenge: roles that inherit from other roles.
pub mod challenge {
    use super::{Permission, Role};
    use std::collections::{HashMap, HashSet};

    /// Every permission `role` has, counting what its parents grant, what
    /// their parents grant, and so on.
    ///
    /// `grants` says what each role grants directly. `parents` says which
    /// roles each role inherits from (a role missing from either map has
    /// nothing there). The parent graph may contain a cycle, for example two
    /// roles that inherit from each other by mistake; the function must still
    /// return, with the permissions of every role reachable from `role`.
    pub fn effective_permissions(
        grants: &HashMap<Role, HashSet<Permission>>,
        parents: &HashMap<Role, Vec<Role>>,
        role: Role,
    ) -> HashSet<Permission> {
        let mut seen = HashSet::new();
        let mut pending = vec![role];
        let mut result = HashSet::new();
        while let Some(next) = pending.pop() {
            if !seen.insert(next) {
                continue;
            }
            if let Some(direct) = grants.get(&next) {
                result.extend(direct.iter().copied());
            }
            if let Some(inherited) = parents.get(&next) {
                pending.extend(inherited.iter().copied());
            }
        }
        result
    }
}

#[cfg(test)]
mod challenge_tests {
    use super::challenge::effective_permissions;
    use super::*;

    fn grants() -> HashMap<Role, HashSet<Permission>> {
        HashMap::from([
            (Role::Member, HashSet::from([Permission::ReviewCreate])),
            (Role::Moderator, HashSet::from([Permission::ReviewEditAny])),
            (Role::Admin, HashSet::from([Permission::ManageUsers])),
        ])
    }

    #[test]
    fn a_role_with_no_parents_has_only_its_own() {
        let result = effective_permissions(&grants(), &HashMap::new(), Role::Moderator);
        assert_eq!(result, HashSet::from([Permission::ReviewEditAny]));
    }

    #[test]
    fn permissions_come_down_a_chain_of_parents() {
        let parents = HashMap::from([
            (Role::Admin, vec![Role::Moderator]),
            (Role::Moderator, vec![Role::Member]),
        ]);
        let result = effective_permissions(&grants(), &parents, Role::Admin);
        assert_eq!(result.len(), 3);
        assert!(result.contains(&Permission::ReviewCreate));
        let member = effective_permissions(&grants(), &parents, Role::Member);
        assert_eq!(member, HashSet::from([Permission::ReviewCreate]));
    }

    #[test]
    fn a_cycle_does_not_loop_forever() {
        let parents = HashMap::from([
            (Role::Member, vec![Role::Moderator]),
            (Role::Moderator, vec![Role::Member]),
        ]);
        let result = effective_permissions(&grants(), &parents, Role::Member);
        assert_eq!(result.len(), 2);
    }
}
