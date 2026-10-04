//! The pure layer: `Policy::allows` and `can`. No router, no requests.

use p3_07_05_modelling_rbac_and_permissions_solution::{
    can, Action, Permission, Policy, Role, User,
};

fn member(id: u64) -> User {
    User::new(id, "member", &[Role::Member])
}

mod allows {
    use super::*;

    #[test]
    fn a_user_gets_what_their_role_grants() {
        let policy = Policy::standard();
        assert!(policy.allows(&member(1), Permission::ReviewCreate));
        assert!(policy.allows(&member(1), Permission::ReviewEditOwn));
    }

    #[test]
    fn a_user_does_not_get_what_no_role_grants() {
        let policy = Policy::standard();
        assert!(!policy.allows(&member(1), Permission::ReviewEditAny));
        assert!(!policy.allows(&member(1), Permission::ManageUsers));
    }

    #[test]
    fn roles_add_up() {
        let policy = Policy::standard();
        let both = User::new(3, "carol", &[Role::Member, Role::Moderator]);
        assert!(policy.allows(&both, Permission::ReviewCreate));
        assert!(policy.allows(&both, Permission::ReviewEditAny));
        let moderator_only = User::new(3, "carol", &[Role::Moderator]);
        assert!(!policy.allows(&moderator_only, Permission::ReviewCreate));
    }

    #[test]
    fn a_user_with_no_roles_gets_nothing() {
        let policy = Policy::standard();
        let nobody = User::new(5, "mallory", &[]);
        assert!(!policy.allows(&nobody, Permission::ReviewCreate));
    }

    #[test]
    fn a_role_the_policy_never_heard_of_grants_nothing() {
        let policy = Policy::new().grant(Role::Member, &[Permission::ReviewCreate]);
        let admin = User::new(4, "dave", &[Role::Admin]);
        assert!(!policy.allows(&admin, Permission::ManageUsers));
        assert!(!policy.allows(&admin, Permission::ReviewCreate));
    }

    #[test]
    fn granting_twice_adds_to_the_role() {
        let policy = Policy::new()
            .grant(Role::Member, &[Permission::ReviewCreate])
            .grant(Role::Member, &[Permission::ReviewEditOwn]);
        assert!(policy.allows(&member(1), Permission::ReviewCreate));
        assert!(policy.allows(&member(1), Permission::ReviewEditOwn));
    }
}

mod ownership {
    use super::*;

    #[test]
    fn create_needs_the_create_permission_and_ignores_the_owner() {
        let policy = Policy::standard();
        assert!(can(&policy, &member(1), Action::Create, None));
        assert!(can(&policy, &member(1), Action::Create, Some(99)));
        let nobody = User::new(5, "mallory", &[]);
        assert!(!can(&policy, &nobody, Action::Create, None));
    }

    #[test]
    fn a_member_edits_and_deletes_their_own_review() {
        let policy = Policy::standard();
        assert!(can(&policy, &member(1), Action::Edit, Some(1)));
        assert!(can(&policy, &member(1), Action::Delete, Some(1)));
    }

    #[test]
    fn a_member_cannot_touch_someone_elses_review() {
        let policy = Policy::standard();
        assert!(!can(&policy, &member(2), Action::Edit, Some(1)));
        assert!(!can(&policy, &member(2), Action::Delete, Some(1)));
    }

    #[test]
    fn an_unknown_owner_matches_nobody() {
        let policy = Policy::standard();
        assert!(!can(&policy, &member(1), Action::Edit, None));
        assert!(!can(&policy, &member(1), Action::Delete, None));
    }

    #[test]
    fn a_moderator_touches_anyones_review() {
        let policy = Policy::standard();
        let carol = User::new(3, "carol", &[Role::Member, Role::Moderator]);
        assert!(can(&policy, &carol, Action::Edit, Some(1)));
        assert!(can(&policy, &carol, Action::Delete, Some(1)));
        assert!(can(&policy, &carol, Action::Edit, None));
    }

    #[test]
    fn edit_and_delete_are_decided_separately() {
        let policy = Policy::new()
            .grant(Role::Member, &[Permission::ReviewEditOwn])
            .grant(Role::Moderator, &[Permission::ReviewDeleteAny]);
        let both = User::new(3, "carol", &[Role::Member, Role::Moderator]);
        assert!(can(&policy, &both, Action::Edit, Some(3)));
        assert!(!can(&policy, &both, Action::Edit, Some(1)));
        assert!(can(&policy, &both, Action::Delete, Some(1)));
    }

    #[test]
    fn owning_a_review_is_not_enough_without_the_own_permission() {
        let policy = Policy::new().grant(Role::Member, &[Permission::ReviewCreate]);
        assert!(!can(&policy, &member(1), Action::Edit, Some(1)));
        assert!(!can(&policy, &member(1), Action::Delete, Some(1)));
    }
}
