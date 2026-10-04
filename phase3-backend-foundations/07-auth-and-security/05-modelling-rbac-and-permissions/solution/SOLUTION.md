# Solution — 3.7.5 Modelling RBAC and permissions

The full code is `solution/src/lib.rs`. It passes everything in `solution/tests/` (13 policy tests, 19 router tests) and the 3 challenge tests inside `lib.rs`.

## Repair

1. `05`: add `Permission::ManageUsers => false,`. No wildcard, so a future permission breaks this `match` again and the compiler lists it.
2. `06`: add a field `_marker: PhantomData<P>` (with `use std::marker::PhantomData;`) and build the value with `_marker: PhantomData`.
3. `07`: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]` on `Role`.
4. `02`: look at the review before writing. Compare `review.author` with the caller's id and return `403` when they differ. The review text must stay as it was.

## `Policy::allows`

```rust
user.roles.iter().any(|role| {
    self.grants
        .get(role)
        .is_some_and(|set| set.contains(&permission))
})
```

For each of the user's roles, look up that role's set. A role the policy has no entry for gives `None`, and `is_some_and` turns that into `false`. A user with no roles makes `any` run over nothing, which is `false`. "Union over roles" is `any`.

## `can`

```rust
let is_owner = owner == Some(user.id);
match action {
    Action::Create => policy.allows(user, Permission::ReviewCreate),
    Action::Edit => {
        policy.allows(user, Permission::ReviewEditAny)
            || (is_owner && policy.allows(user, Permission::ReviewEditOwn))
    }
    Action::Delete => { /* the same with the Delete permissions */ }
}
```

`owner == Some(user.id)` is `false` for `None`, which gives "an unknown owner matches nobody" with no extra code. `Any` is checked first and does not look at the owner. `Own` needs both the permission and the match. The `match` has no wildcard, so a new `Action` is a compile error here.

## `RequirePermission`

```rust
let Authenticated(user) = Authenticated::from_request_parts(parts, state).await?;
if state.policy().allows(&user, P::PERMISSION) {
    Ok(RequirePermission { user, _marker: PhantomData })
} else {
    Err(ApiError::Forbidden)
}
```

Reusing `Authenticated` means every `401` case is decided in one place, and `?` passes its rejection through unchanged. Only a known user can reach the `403`.

## Handlers

`update_review` and `delete_review` lock the store, look the review up (`NotFound` first), then call `can(.., Some(review.author))` and answer `Forbidden` on `false`. The `Authenticated` argument runs first, so the order is `401`, `404`, `403`. `grant_role` takes `RequirePermission<CanManageUsers>` as its first argument, so `401` and `403` happen before the path or body is touched, and only then does it look up the user (`404`) and insert the role into the set.

Note that `update_review` holds the lock across the check and the write. With the check and the write under one lock, a role change cannot slip in between them.

## Challenge

```rust
let mut seen = HashSet::new();
let mut pending = vec![role];
let mut result = HashSet::new();
while let Some(next) = pending.pop() {
    if !seen.insert(next) { continue; }
    if let Some(direct) = grants.get(&next) { result.extend(direct.iter().copied()); }
    if let Some(inherited) = parents.get(&next) { pending.extend(inherited.iter().copied()); }
}
result
```

An explicit stack and a `seen` set. `seen.insert` returns `false` for a role already visited, which is what stops a cycle: each role is processed once, so the loop ends after at most one visit per role.
