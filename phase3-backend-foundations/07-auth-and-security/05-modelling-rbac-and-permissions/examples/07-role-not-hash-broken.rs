//! DELIBERATELY BROKEN — expected: E0599
//! Run `cargo build -p p3-07-05-modelling-rbac-and-permissions --example 07-role-not-hash-broken --features broken` and read the error.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy)]
enum Role {
    Member,
    Moderator,
}

fn main() {
    let mut grants: HashMap<Role, HashSet<&str>> = HashMap::new();
    grants.insert(Role::Member, HashSet::from(["review:create"]));
    grants.insert(Role::Moderator, HashSet::from(["review:edit_any"]));
    println!("{}", grants.len());
}
