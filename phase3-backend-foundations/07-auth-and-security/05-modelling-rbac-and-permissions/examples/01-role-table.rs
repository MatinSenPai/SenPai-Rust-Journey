//! A role table and a user's effective permissions, with plain std types.
//! Run: cargo run -p p3-07-05-modelling-rbac-and-permissions --example 01-role-table

use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Role {
    Member,
    Moderator,
    Admin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Permission {
    ReviewCreate,
    ReviewEditAny,
    ManageUsers,
}

fn main() {
    use Permission::*;
    let table: HashMap<Role, Vec<Permission>> = HashMap::from([
        (Role::Member, vec![ReviewCreate]),
        (Role::Moderator, vec![ReviewEditAny]),
        (Role::Admin, vec![ManageUsers]),
    ]);
    let users = [
        ("alice", vec![Role::Member]),
        ("carol", vec![Role::Member, Role::Moderator]),
        ("mallory", vec![]),
    ];
    for (name, roles) in users {
        let effective: BTreeSet<Permission> = roles
            .iter()
            .flat_map(|role| table[role].iter().copied())
            .collect();
        println!("{name:<8} roles={roles:?}  can={effective:?}");
    }
}
