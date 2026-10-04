//! Tests for `UserStore`. Cheap params keep every test fast.

use p3_07_01_password_hashing_argon2::*;

fn store() -> UserStore {
    UserStore::new(cheap_params())
}

#[test]
fn register_then_login_works() {
    let mut s = store();
    assert_eq!(s.register("aria", "correct horse"), Ok(()));
    assert_eq!(s.login("aria", "correct horse"), Ok(()));
}

#[test]
fn the_store_never_keeps_the_plaintext() {
    let mut s = store();
    s.register("aria", "correct horse").unwrap();
    let stored = s.stored_hash("aria").unwrap();
    assert!(stored.starts_with("$argon2id$"));
    assert!(!stored.contains("correct horse"));
    assert_eq!(s.stored_hash("nobody"), None);
}

#[test]
fn unknown_user_and_wrong_password_give_the_same_error() {
    let mut s = store();
    s.register("aria", "correct horse").unwrap();
    let wrong_password = s.login("aria", "wrong horse");
    let unknown_user = s.login("ghost", "correct horse");
    assert_eq!(wrong_password, Err(AuthError::InvalidCredentials));
    assert_eq!(unknown_user, wrong_password);
}

#[test]
fn register_rejects_short_passwords_and_duplicates() {
    let mut s = store();
    assert_eq!(
        s.register("aria", "short"),
        Err(AuthError::PasswordTooShort)
    );
    assert_eq!(s.stored_hash("aria"), None);
    s.register("aria", "correct horse").unwrap();
    assert_eq!(
        s.register("aria", "another long one"),
        Err(AuthError::UsernameTaken)
    );
    assert_eq!(s.login("aria", "correct horse"), Ok(()));
}

#[test]
fn eight_characters_counts_chars_not_bytes() {
    let mut s = store();
    // 4 Persian letters are 8 bytes but only 4 characters.
    assert_eq!(s.register("a", "سلام"), Err(AuthError::PasswordTooShort));
    assert_eq!(s.register("b", "سلامسلام"), Ok(()));
}

#[test]
fn two_users_with_the_same_password_get_different_rows() {
    let mut s = store();
    s.register("aria", "correct horse").unwrap();
    s.register("ben", "correct horse").unwrap();
    assert_ne!(s.stored_hash("aria"), s.stored_hash("ben"));
}
