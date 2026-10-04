# 3.7.5 — Modelling RBAC and permissions

## At a glance

After this lesson you can:

- Model "who may do what" with three plain types (users hold roles, roles grant permissions) and answer it with one pure function, `can`, that you can test without a server.
- Explain why `401`, `403` and `404` come from three different questions ("who are you?", "does this exist?", "may you?") and put the checks in that order.
- Spot and fix a broken-access-control bug (an IDOR) where a handler checks *that* you are logged in but never *whose* review you are editing.
- Write a `RequirePermission<P>` extractor in `axum` 0.8 that turns a role check into one argument in a handler's signature.

**Time:** ~80 minutes · **Prerequisites:**
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md),
[3.2.2 — Writing your own extractor (`FromRequestParts`)](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.md),
[3.7.3 — JWTs and `tower` middleware](../03-jwt-and-tower-middleware/README.md)

---

## Why this matters

The rest of this module answered *who is calling*: a password hash you can check, a session or a token that carries the proof, middleware that verifies it on every request. This lesson answers the question that comes next, and the one that goes wrong most often in real services: **what is this caller allowed to do?**

In Django you got an answer without writing one. `Group` and `Permission` are tables, `user.has_perm("reviews.change_review")` asks them, and `@permission_required` or a DRF `permission_classes = [...]` line guards a view. In `axum` nothing is there until you build it. That is a chance to see how small the idea is. Roles and permissions are a lookup table, and "may this user edit this review?" is one boolean function. The hard part is not the code. It is remembering to ask the question in every handler.

That is why OWASP's Top 10 puts **A01, Broken Access Control**, at number one. The usual form is not a clever exploit. It is a handler that checks you are logged in and then does what the URL says. Change `/reviews/1` to `/reviews/2` and you are editing someone else's data. This lesson builds the model, the extractor, and the habit that stops it.

---

## The concept

### Authentication asks "who", authorization asks "may"

Two different questions run on every protected request, and they fail with different status codes. You met the pair in 3.1.3:

- **Authentication**: who is this? No usable credentials gives `401 Unauthorized`. The name is misleading, because it really means *unauthenticated*.
- **Authorization**: this is a known user. May they do *this*? "No" gives `403 Forbidden`.

The rest of the module built the first question. This lesson is the second. In Django terms, `request.user` is the answer to the first and `has_perm` is the second. DRF's `permission_classes` run after its authentication classes for the same reason.

There is a third question that sits between them, and it decides the order of the checks. "Does review 99 exist?" gives `404`. This lesson's handlers ask the three in this order: who, then what, then may you. Section "Where the checks sit" says why.

### Three plain types: permission, role, user

Start with data and no HTTP. A **permission** is one thing a caller may do. A **role** is a named bundle of permissions. A user holds a set of roles. Their effective permissions are the union of what each role grants:

```rust
enum Role { Member, Moderator, Admin }
enum Permission { ReviewCreate, ReviewEditAny, ManageUsers }

// the role table: which permissions each role grants
let table: HashMap<Role, Vec<Permission>> = /* Member -> [ReviewCreate], ... */;
// a user's effective permissions: the union over their roles
roles.iter().flat_map(|role| table[role].iter().copied()).collect()
```

`examples/01-role-table.rs` builds the table and prints three users:

```text
alice    roles=[Member]  can={ReviewCreate}
carol    roles=[Member, Moderator]  can={ReviewCreate, ReviewEditAny}
mallory  roles=[]  can={}
```

Look at carol. She is a moderator *and* a member, so she holds both roles and gets both sets. Roles here only add; none takes anything away. A moderator who should also write reviews is given `Member` too, and the lesson's server does exactly that. Mallory has no roles. She is a real, known user and is allowed nothing, which is a legal state and the safe default.

```senpai-visual
{"kind":"concept","labels":["user holds roles: member, moderator","role grants permissions: member -> create, moderator -> edit any","effective permissions = union over the user's roles","a user with no roles can do nothing","permissions are what code checks, roles are how people get them"]}
```

The permission is what your code checks. The role is how people *receive* permissions. Handlers should ask "may this user edit any review?", never "is this user a moderator?", because the first still reads correctly the day you add a `Support` role that can also edit reviews.

Django's version of this is `Permission` (auto-created per model: `add_review`, `change_review`, `delete_review`, `view_review`), `Group` (a role) and `user.groups`. Two differences matter. Django's permissions are **per model**, not per object. And Django's default backend answers `False` to any object-level `has_perm(perm, obj)` check. The line "a user may edit *their own* review" is therefore always something you write yourself. The next section writes it.

### `can`: one pure function that decides

Ownership does not fit the role table. "Moderators may edit any review" is a role fact. "Members may edit *their own* review" depends on a *particular* review. So the model splits each edit permission in two: `ReviewEditOwn` and `ReviewEditAny`. Then one function combines the table with the review's author:

```rust
fn can_edit(roles: &[Role], user_id: u64, owner: Option<u64>) -> bool {
    roles.contains(&Role::Moderator)
        || (roles.contains(&Role::Member) && owner == Some(user_id))
}
```

`examples/03-can-matrix.rs` asks each of three users two questions:

```text
user                      own review      alice's review
alice (member, id 1)      true            true
bob (member, id 2)        true            false
carol (moderator, id 3)   true            true
```

The `false` in bob's row is the whole point. He holds a member role, and that role does let members edit reviews, but only their own. The lesson's real `can(policy, user, action, owner)` has the same shape. It takes the policy, the user, an `Action` (`Create`, `Edit` or `Delete`) and `owner: Option<UserId>`, the author of the review in question or `None`. `None` never matches anybody, so a caller that does not know the owner gets `false`, not an accidental `true`.

The function is **pure**: no request, no database, no clock. That is the design decision worth copying. Every rule in your authorization policy can be tested with plain values, in microseconds, in a table. `axum` is only a way to deliver the question and the answer.

### Broken access control: the IDOR

Now see the bug that function exists to prevent. `examples/02-idor-missing-owner-check.rs` is an `edit_review` that checks the caller is logged in and nothing else. It compiles and it runs:

```rust
fn edit_review(reviews: &mut HashMap<u64, Review>, logged_in: Option<u64>, id: u64, text: &str) -> u16 {
    if logged_in.is_none() {
        return 401;
    }
    match reviews.get_mut(&id) {
        Some(review) => { review.text = text.to_string(); 200 }
        None => 404,
    }
}
```

```text
bob (id 2) edits review 1: 200
review 1 now says "this anime is trash", author still 1
```

This is an **IDOR**, an *insecure direct object reference*: the request names the object directly (`/reviews/1`), and the server never asks whether this caller may touch *that* object. It is the standard shape of OWASP A01. No error appears anywhere. The code is a complete, working "authenticated users can edit reviews" feature, which is not the feature anyone meant. Only a test with the wrong user catches it, which is why the tests for this lesson always include one.

The fix is not a new framework. It is passing the review's author to `can`. A handler that touches an object has to load the object, take its owner, and ask.

### An extractor for the role-level check

Some routes need no object at all. `POST /users/{id}/roles` is for admins, full stop. For those, repeating `if !can(...) { return Err(Forbidden) }` at the top of every handler is the copy-paste problem 3.2.2 solved with extractors, and the same tool fits. `examples/04-require-permission-extractor.rs` is a self-contained version:

```rust
struct Require<P>(PhantomData<P>);

impl<P: Needs, S: Send + Sync> FromRequestParts<S> for Require<P> {
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, StatusCode> {
        let header = parts.headers.get("x-permissions").ok_or(StatusCode::UNAUTHORIZED)?;
        let held: HashSet<&str> = header.to_str().unwrap_or("").split(',').collect();
        if held.contains(P::PERMISSION) { Ok(Require(PhantomData)) } else { Err(StatusCode::FORBIDDEN) }
    }
}

async fn admin_page(_: Require<ManageUsers>) -> &'static str { "welcome, admin" }
```

```text
x-permissions: (absent)             -> 401 Unauthorized
x-permissions: create               -> 403 Forbidden
x-permissions: create,manage_users  -> 200 OK
```

Three things to see. `Require<ManageUsers>` in the signature *is* the authorization rule: a reader sees it on the function, and `axum` runs the check before the handler body. `P` appears only in the type, and `PhantomData<P>` is what lets the struct carry a type parameter without a field of that type. The rejection splits exactly as 3.1.3 taught: no credentials is `401`, credentials that are not enough is `403`. (The example reads permissions straight from a header, which would be a security hole in real life. The lesson's crate looks the user up from a bearer token, which stands in for the verified JWT from 3.7.3.)

The crate's `RequirePermission<P>` is the same idea against the real model: it extracts `Authenticated`, then asks `Policy::allows(user, P::PERMISSION)`, and it returns the `User` so the handler can use it.

### Where the checks sit

An extractor runs before the handler, so it knows the request's headers and nothing else. It cannot know which review `/reviews/1` is. That sets the division of labour:

```senpai-visual
{"kind":"network","labels":["request","Authenticated: no usable token -> 401","RequirePermission: no role grants it -> 403 (role-level routes)","handler loads the object: missing -> 404","can(user, action, owner): not allowed -> 403","do the work -> 200 / 204"]}
```

The extractor is the **coarse gate** ("may your roles do this *kind* of thing at all?"). `can` inside the handler is the **fine check** ("may you do it to *this* object?"). The order the lesson picks for object routes is 401, then 404, then 403. The reasoning: with a missing review there is no owner to compare against, so "may you?" has no answer yet. A real design question lives here. For *public* data like reviews, a `403` that confirms "review 1 exists but is not yours" leaks nothing. For *private* data, many services answer `404` to anyone who may not see the object, so that a `403` never confirms it exists. Pick one on purpose and write it down.

### Roles in storage, and roles in a token

Everything so far is in memory. In a database, the same three ideas become three tables. This is a sketch to show the shape, and the schema work itself belongs to [3.5.2 — Migrations](../../05-postgres-and-sqlx/02-migrations/README.md) and [3.6.3 — Schema design for a real service](../../06-database-design-and-query-performance/03-schema-design-for-a-real-service/README.md):

```sql
-- illustrative sketch, not run in this lesson
CREATE TABLE roles (id SERIAL PRIMARY KEY, name TEXT NOT NULL UNIQUE);
CREATE TABLE role_permissions (
    role_id    INT  NOT NULL REFERENCES roles (id),
    permission TEXT NOT NULL,
    PRIMARY KEY (role_id, permission)
);
CREATE TABLE user_roles (
    user_id BIGINT NOT NULL REFERENCES users (id),
    role_id INT    NOT NULL REFERENCES roles (id),
    PRIMARY KEY (user_id, role_id)
);
```

Notice that the permission names stay text. The Rust `enum Permission` is what makes the compiler list every `match` you forgot to update when you add one (the first error below), so the usual shape is: roles and their grants in the database, permissions as an enum in code.

The second question is *where the roles are read from at request time*. If they ride inside a signed token, a promotion or a demotion does not take effect until that token is replaced. In this lesson `AppState::user_for_token` reads the user's roles from the store on every request, so `POST /users/{id}/roles` is effective on the very next call. You will see it happen below. That freshness is the exact trade-off the token lessons earlier in this module were about, and why short-lived access tokens with refresh-token rotation (3.7.4) exist.

---

## Hands on

The lesson's server is `src/main.rs`, on port `3210`, seeded with five users: alice (member), bob (member), carol (member and moderator), dave (member, moderator and admin) and mallory (no roles). Their bearer tokens are `alice-token`, `bob-token` and so on. The skeleton's `todo!()` functions panic until you do the Implement and Build rungs, so run the examples first:

```sh
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 01-role-table
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 02-idor-missing-owner-check
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 03-can-matrix
cargo run -p p3-07-05-modelling-rbac-and-permissions --example 04-require-permission-extractor
```

Then the three that do not compile:

```sh
cargo build -p p3-07-05-modelling-rbac-and-permissions --example 05-permission-match-non-exhaustive-broken --features broken
cargo build -p p3-07-05-modelling-rbac-and-permissions --example 06-unused-type-parameter-broken --features broken
cargo build -p p3-07-05-modelling-rbac-and-permissions --example 07-role-not-hash-broken --features broken
```

Once Implement and Build are done, start the server (`cargo run -p p3-07-05-modelling-rbac-and-permissions`) and talk to it. The transcripts below come from the finished `solution/` crate, which is the same router. Each header block ends with a `date` line that changes every run. First, no token:

```sh
curl -si http://127.0.0.1:3210/me
```

```text
HTTP/1.1 401 Unauthorized
content-type: application/json
www-authenticate: Bearer
content-length: 35
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"authentication required"}
```

`401` with a `www-authenticate: Bearer` challenge, so the client knows what kind of credential to send. Now bob, a member, edits alice's review, and then carol, a moderator, does:

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/1 -H 'authorization: Bearer bob-token' -H 'content-type: application/json' -d '{"text":"hacked"}'
```

```text
HTTP/1.1 403 Forbidden
content-type: application/json
content-length: 42
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"you are not allowed to do that"}
```

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/1 -H 'authorization: Bearer carol-token' -H 'content-type: application/json' -d '{"text":"[removed by moderator]"}'
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 69
date: Sun, 04 Oct 2026 11:16:58 GMT

{"id":1,"author":1,"anime":"Frieren","text":"[removed by moderator]"}
```

The same URL and the same body gave `403` and `200`. Only the caller's roles differ. Now the role-level route. Carol is a moderator but not an admin, so she cannot grant roles. Dave can:

```sh
curl -si -X POST http://127.0.0.1:3210/users/2/roles -H 'authorization: Bearer carol-token' -H 'content-type: application/json' -d '{"role":"moderator"}'
```

```text
HTTP/1.1 403 Forbidden
content-type: application/json
content-length: 42
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"you are not allowed to do that"}
```

```sh
curl -si -X POST http://127.0.0.1:3210/users/2/roles -H 'authorization: Bearer dave-token' -H 'content-type: application/json' -d '{"role":"moderator"}'
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 52
date: Sun, 04 Oct 2026 11:16:58 GMT

{"id":2,"name":"bob","roles":["member","moderator"]}
```

Bob is a moderator now. The same request that failed a moment ago goes through at once, because roles are looked up on every request, not frozen into a token:

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/1 -H 'authorization: Bearer bob-token' -H 'content-type: application/json' -d '{"text":"bob is a moderator now"}'
```

```text
HTTP/1.1 200 OK
content-type: application/json
content-length: 69
date: Sun, 04 Oct 2026 11:16:58 GMT

{"id":1,"author":1,"anime":"Frieren","text":"bob is a moderator now"}
```

Last, the order of the checks. Alice is allowed to edit her own review, but review 99 does not exist, so the answer is `404`, not `403`:

```sh
curl -si -X PATCH http://127.0.0.1:3210/reviews/99 -H 'authorization: Bearer alice-token' -H 'content-type: application/json' -d '{"text":"x"}'
```

```text
HTTP/1.1 404 Not Found
content-type: application/json
content-length: 21
date: Sun, 04 Oct 2026 11:16:58 GMT

{"error":"not found"}
```

Then try these:

1. In `01-role-table`, add a `Support` role that grants `ReviewEditAny` and give a user `[Member, Support]`. What does their `can=` set look like? What would you have had to change if handlers had checked `Role::Moderator` instead of the permission?
2. In `03-can-matrix`, add a user with no roles. What happens in the `own review` column, and is that what the real policy should do?
3. In `04-require-permission-extractor`, send `x-permissions: manage_users_please`. Which status do you get, and why isn't it a `200`?

---

## Errors you will meet

### `E0004`: a `match` over permissions that misses one

```text
error[E0004]: non-exhaustive patterns: `&Permission::ManageUsers` not covered
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\05-permission-match-non-exhaustive-broken.rs:13:11
   |
13 |     match permission {
   |           ^^^^^^^^^^ pattern `&Permission::ManageUsers` not covered
   |
note: `Permission` defined here
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\05-permission-match-non-exhaustive-broken.rs:5:6
   |
 5 | enum Permission {
   |      ^^^^^^^^^^
...
 9 |     ManageUsers,
   |     ----------- not covered
   = note: the matched value is of type `&Permission`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
16 ~         Permission::ReviewEditOwn => false,
17 ~         &Permission::ManageUsers => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p3-07-05-modelling-rbac-and-permissions` (example "05-permission-match-non-exhaustive-broken") due to 1 previous error
```

**What the compiler is objecting to:** the function lists three of the four permissions and has no arm for `ManageUsers`. `rustc` knows every variant of an enum, so a `match` without a wildcard must name them all.

**The fix:** add the arm, and decide on purpose what the answer is (`ManageUsers` does not need a moderator, so `false`). Do not paste the suggested `todo!()`, and avoid a `_ => false` wildcard:

```rust
Permission::ManageUsers => false,
```

**Why this is the fix:** the error is the feature. When you add a permission next month, this error lists *every* function that must now say what it does with it. A `_` arm would silence that list, and the day someone adds `ReviewDeleteAny`, the wildcard quietly decides for you. In an authorization table that is the wrong place to default.

### `E0392`: a type parameter that appears in no field

```text
error[E0392]: type parameter `P` is never used
 --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\06-unused-type-parameter-broken.rs:9:26
  |
9 | struct RequirePermission<P> {
  |                          ^ unused type parameter
  |
  = help: consider removing `P`, referring to it in a field, or using a marker such as `PhantomData`
  = help: if you intended `P` to be a const parameter, use `const P: /* Type */` instead

error[E0282]: type annotations needed
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\06-unused-type-parameter-broken.rs:16:5
   |
16 |     RequirePermission { user }
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^ cannot infer type of the type parameter `P` declared on the struct `RequirePermission`
   |
help: consider specifying a concrete type for the type parameter `P`
   |
16 |     RequirePermission::</* Type */> { user }
   |                      ++++++++++++++

Some errors have detailed explanations: E0282, E0392.
For more information about an error, try `rustc --explain E0282`.
error: could not compile `p3-07-05-modelling-rbac-and-permissions` (example "06-unused-type-parameter-broken") due to 2 previous errors
```

**What the compiler is objecting to:** `P` is only there to name a permission in the type. No field has that type, so `rustc` cannot work out what the struct means for different `P`s, and it reports the declaration. The second error is a follow-on: with `P` unused, nothing in the constructor can tell `rustc` which `P` is meant.

**The fix:** give the struct a field that mentions `P` without storing one:

```rust
struct RequirePermission<P> { user: User, _marker: PhantomData<P> }
```

**Why this is the fix:** `PhantomData<P>` takes no memory and says "this struct behaves as if it were tied to `P`". Construct it with `_marker: PhantomData`. Fix the first error and the second goes away.

### `E0599`: using a `Role` as a `HashMap` key without `Hash`

```text
error[E0599]: the method `insert` exists for struct `HashMap<Role, HashSet<&str>>`, but its trait bounds were not satisfied
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\07-role-not-hash-broken.rs:14:12
   |
 7 | enum Role {
   | --------- doesn't satisfy `Role: Eq` or `Role: Hash`
...
14 |     grants.insert(Role::Member, HashSet::from(["review:create"]));
   |            ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Role: Eq`
           `Role: Hash`
help: consider annotating `Role` with `#[derive(Eq, Hash, PartialEq)]`
   |
 7 + #[derive(Eq, Hash, PartialEq)]
 8 | enum Role {
   |

error[E0599]: the method `insert` exists for struct `HashMap<Role, HashSet<&str>>`, but its trait bounds were not satisfied
  --> phase3-backend-foundations\07-auth-and-security\05-modelling-rbac-and-permissions\examples\07-role-not-hash-broken.rs:15:12
   |
 7 | enum Role {
   | --------- doesn't satisfy `Role: Eq` or `Role: Hash`
...
15 |     grants.insert(Role::Moderator, HashSet::from(["review:edit_any"]));
   |            ^^^^^^
   |
   = note: the following trait bounds were not satisfied:
           `Role: Eq`
           `Role: Hash`
help: consider annotating `Role` with `#[derive(Eq, Hash, PartialEq)]`
   |
 7 + #[derive(Eq, Hash, PartialEq)]
 8 | enum Role {
   |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p3-07-05-modelling-rbac-and-permissions` (example "07-role-not-hash-broken") due to 2 previous errors
```

**What the compiler is objecting to:** the declaration `HashMap<Role, ...>` is allowed, but `insert` needs the key type to implement `Eq` and `Hash`, and this `Role` derives neither. The error is reported once per call, so you see it twice.

**The fix:** do what the help line says:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
```

**Why this is the fix:** a map finds a key by hashing it and then comparing, so a key type has to be hashable and comparable for equality. A fieldless enum like `Role` can derive all of it for free.

### No error at all: the missing owner check

```text
bob (id 2) edits review 1: 200
review 1 now says "this anime is trash", author still 1
```

**What's actually broken:** `examples/02-idor-missing-owner-check.rs` compiles, runs, and returns `200` every time. It checks that someone is logged in and never compares the caller with the review's author. No type can express "this caller owns this row", so no compiler will catch it.

**The fix:** look at who wrote the review before changing it. Return `403` when the caller is not the author:

```rust
if review.author != caller { return 403; }
```

**Why this is the fix:** the decision needs *two* inputs, the caller and the object. Authentication has only the first. In the lesson's crate that comparison lives in `can`, and the only way to find the bug is a test where the wrong user tries it. Every ownership rule you write should come with that test.

---

## Exercises

### Warm up

<details>
<summary>A user whose only role is <code>Member</code> calls <code>POST /users/2/roles</code>, with a valid token. <code>401</code> or <code>403</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`403 Forbidden`. The token is fine, so the server knows who the caller is. Their roles do not include `ManageUsers`. `401` is for "I could not identify you".

</details>

<details>
<summary>Carol holds only <code>Moderator</code>, not <code>Member</code>. With the lesson's standard policy, may she create a review?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. `ReviewCreate` is granted by `Member`, and roles only add. That is why the seeded carol holds both `Member` and `Moderator`. Neither role includes the other's permissions.

</details>

<details>
<summary>What does <code>can(&amp;policy, &amp;member, Action::Edit, None)</code> return for a plain member, and what does it return for a moderator?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`false` for the member and `true` for the moderator. `owner: None` never matches anybody's id, so the "own" permission cannot help, but a moderator holds `ReviewEditAny`, which does not look at the owner at all.

</details>

<details>
<summary>Bob (a plain member) sends <code>PATCH /reviews/99</code>, and review 99 does not exist. Which status comes back, and why not <code>403</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`404 Not Found`. This lesson checks who (`401`), then what (`404`), then may you (`403`). With no review there is no owner to compare against, so "may you?" is never asked.

</details>

### Repair

Fix all four:

1. `examples/05-permission-match-non-exhaustive-broken.rs` compiles, with a real arm for `ManageUsers` and no wildcard.
2. `examples/06-unused-type-parameter-broken.rs` compiles and prints `4`.
3. `examples/07-role-not-hash-broken.rs` compiles and prints `2`.
4. `examples/02-idor-missing-owner-check.rs` prints `403` for bob and leaves the review's text unchanged. (This one already compiles. Its bug is in the logic.)

### Implement

Two functions in `src/lib.rs`, with no HTTP involved:

```sh
cargo test -p p3-07-05-modelling-rbac-and-permissions --test policy_test
```

Each doc comment is its full specification:

- `Policy::allows`: does any of the user's roles grant this permission?
- `can`: may this user perform this action on a review written by `owner`? Create, edit and delete, with the "own" and "any" permissions.

### Build

Make the server work end to end. Four more `todo!()`s in `src/lib.rs`: the `FromRequestParts` impl for `RequirePermission<P>`, and the handlers `update_review`, `delete_review` and `grant_role`. The check order and status codes are in their doc comments. The 19 tests in `tests/api_test.rs` drive the whole router with `oneshot`:

```sh
cargo test -p p3-07-05-modelling-rbac-and-permissions --test api_test
```

### Challenge (optional)

Real systems often let a role *inherit* from another: `Admin` gets everything `Moderator` has, which gets everything `Member` has. Implement `challenge::effective_permissions`, which takes what each role grants directly and a map of parents, and returns every permission a role has, parents included. Be ready for a mistaken config where two roles inherit from each other. The function must still return. This reaches ahead to graph traversal, but only a visited set and a loop are needed, and the three tests in `src/lib.rs` check it.

---

## Wrapping up

This lesson closes module 3.7. The module followed one request's trust from end to end. [3.7.1 — Password hashing with `argon2`](../01-password-hashing-argon2/README.md) stored the thing that proves who you are. [3.7.2 — Sessions vs. JWT: the real trade-off](../02-sessions-vs-jwt/README.md) weighed the two ways that proof travels with each request. [3.7.3 — JWTs and `tower` middleware](../03-jwt-and-tower-middleware/README.md) verified it on every call. [3.7.4 — Refresh-token rotation and revocation](../04-refresh-token-rotation-and-revocation/README.md) handled the proof having to stop working early. This lesson answers what a proven identity is allowed to do. Next, module 3.8 turns to what your API says when it refuses, and how to test all of it at scale.

| Term | What it means | Where you'll use it |
|---|---|---|
| Authentication | establishing who the caller is; failure is `401` | every protected route |
| Authorization | deciding what a known caller may do; failure is `403` | every protected route |
| Permission | one thing a caller may do, such as `ReviewEditAny` | what handlers and `can` check |
| Role | a named bundle of permissions that users hold | how permissions are assigned and audited |
| RBAC (role-based access control) | permissions come from roles, not from individual users | most real access-control systems |
| `can` | one pure yes/no function of policy, user, action and owner | tests, handlers, anywhere |
| Ownership check | comparing the caller with an object's owner | edit and delete by id |
| IDOR / broken access control | acting on an object by id without a check; OWASP A01 | reviewing any handler that takes an id |
| `RequirePermission<P>` | an extractor that rejects callers lacking `P` | role-level routes |

### What you now know

- Authentication and authorization are two questions with two status codes. A known user who is not allowed gets `403`, not `401`.
- Users hold roles, roles grant permissions, and effective permissions are the union. Handlers check permissions, not role names.
- Ownership does not fit a role table, so permissions come in "own" and "any" forms, and one pure function, `can`, combines them with the object's owner.
- A handler that takes an id and checks only that you are logged in has an IDOR, and no compiler will warn you.
- A `FromRequestParts` extractor is the right home for the coarse, role-level check. The object-level check cannot live there, because the extractor does not know the object.
- Where roles are read from decides how fast a change takes effect: from the store on each request, at once. From a signed token, when it is replaced.

### What comes back later

- **One error body shape for `401`, `403`, `404` and the rest, in every handler** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **Testing an authorization policy against a real database** — [3.8.3 — Integration tests with `testcontainers`](../../08-error-handling-and-testing-at-scale/03-integration-tests-with-testcontainers/README.md)
- **Building users, roles and reviews quickly for tests** — [3.8.4 — Test data factories and fixtures](../../08-error-handling-and-testing-at-scale/04-test-data-factories-and-fixtures/README.md)
- **Storing roles and grants in tables, with migrations** — [3.5.2 — Migrations](../../05-postgres-and-sqlx/02-migrations/README.md)
- **Designing real tables, where a sketch like the one above gets its constraints and indexes** — [3.6.3 — Schema design for a real service](../../06-database-design-and-query-performance/03-schema-design-for-a-real-service/README.md)

### Can you explain?

- Why does a known user without the right role get `403` while a request with no token gets `401`?
- Why is "may this user edit this review?" a function of the review, and not only of the user's roles?
- What is an IDOR, and why can't the compiler find one?
- Why does `RequirePermission<P>` need `PhantomData`, and why can't the extractor do the ownership check?
- Why would a `_ => false` arm in a `match` over permissions be a mistake, even though it compiles?
- What changes about a promotion's timing if roles are read from a stored table on every request, instead of from a token's claims?

---

## Going further

- [OWASP Top 10: A01 Broken Access Control](https://owasp.org/Top10/): the category this lesson's IDOR belongs to, with real-world failure patterns.
- [OWASP Authorization Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authorization_Cheat_Sheet.html): deny by default, enforce on every request, and test your authorization logic.
- [Django: permissions and authorization](https://docs.djangoproject.com/en/stable/topics/auth/default/#permissions-and-authorization): `Permission`, `Group` and `has_perm`, the model this lesson rebuilt by hand.
- [DRF: permissions](https://www.django-rest-framework.org/api-guide/permissions/): `has_permission` versus `has_object_permission`, the same coarse and fine split as the extractor and `can`.
- [axum: `FromRequestParts`](https://docs.rs/axum/0.8/axum/extract/trait.FromRequestParts.html): the trait behind `RequirePermission`.
