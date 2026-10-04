//! One pure function answers "may this user do this to this review?".
//! Run: cargo run -p p3-07-05-modelling-rbac-and-permissions --example 03-can-matrix

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Member,
    Moderator,
}

/// Edit rule: a moderator may edit anything, a member only their own review.
fn can_edit(roles: &[Role], user_id: u64, owner: Option<u64>) -> bool {
    roles.contains(&Role::Moderator) || (roles.contains(&Role::Member) && owner == Some(user_id))
}

fn main() {
    let cases = [
        ("alice (member, id 1)", vec![Role::Member], 1),
        ("bob (member, id 2)", vec![Role::Member], 2),
        ("carol (moderator, id 3)", vec![Role::Moderator], 3),
    ];
    println!("{:<26}{:<16}{}", "user", "own review", "alice's review");
    for (name, roles, id) in cases {
        let own = can_edit(&roles, id, Some(id));
        let alices = can_edit(&roles, id, Some(1));
        println!("{name:<26}{own:<16}{alices}");
    }
}
