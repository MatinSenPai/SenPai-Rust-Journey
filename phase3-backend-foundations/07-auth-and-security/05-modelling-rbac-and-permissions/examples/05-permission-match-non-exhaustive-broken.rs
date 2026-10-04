//! DELIBERATELY BROKEN — expected: E0004
//! Run `cargo build -p p3-07-05-modelling-rbac-and-permissions --example 05-permission-match-non-exhaustive-broken --features broken` and read the error.

#[allow(dead_code)]
enum Permission {
    ReviewCreate,
    ReviewEditOwn,
    ReviewEditAny,
    ManageUsers,
}

fn needs_moderator(permission: &Permission) -> bool {
    match permission {
        Permission::ReviewEditAny => true,
        Permission::ReviewCreate => false,
        Permission::ReviewEditOwn => false,
    }
}

fn main() {
    println!("{}", needs_moderator(&Permission::ReviewEditAny));
}
