//! DELIBERATELY BROKEN — expected: E0392
//! Run `cargo build -p p3-07-05-modelling-rbac-and-permissions --example 06-unused-type-parameter-broken --features broken` and read the error.

struct User {
    id: u64,
}

/// "Proof that this user holds permission P", with P only in the type.
struct RequirePermission<P> {
    user: User,
}

struct ManageUsers;

fn check(user: User) -> RequirePermission<ManageUsers> {
    RequirePermission { user }
}

fn main() {
    println!("{}", check(User { id: 4 }).user.id);
}
