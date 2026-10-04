//! Tests for the hashing functions. Every test uses `cheap_params()`, so the
//! whole file runs in a fraction of a second.

use argon2::Params;
use p3_07_01_password_hashing_argon2::*;

const PHC: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";

#[test]
fn hash_then_verify_round_trips() {
    let phc = hash_password("correct horse battery staple", &cheap_params());
    assert!(verify_password("correct horse battery staple", &phc));
    assert!(!verify_password("wrong password", &phc));
}

#[test]
fn hash_is_an_argon2id_phc_string_with_the_requested_cost() {
    let phc = hash_password("x", &cheap_params());
    assert!(phc.starts_with("$argon2id$v=19$m=8,t=1,p=1$"), "{phc}");
    assert!(!phc.contains('x') || phc.starts_with('$'));
}

#[test]
fn same_password_hashes_differently_each_time() {
    let a = hash_password("same", &cheap_params());
    let b = hash_password("same", &cheap_params());
    assert_ne!(a, b);
    assert!(verify_password("same", &a) && verify_password("same", &b));
}

#[test]
fn empty_password_round_trips() {
    let phc = hash_password("", &cheap_params());
    assert!(verify_password("", &phc));
    assert!(!verify_password("not empty", &phc));
}

#[test]
fn verify_rejects_malformed_hashes_without_panicking() {
    assert!(!verify_password("anything", "not-a-real-phc-hash-string"));
    assert!(!verify_password("anything", ""));
}

#[test]
fn verify_uses_the_cost_stored_in_the_hash() {
    let weak = hash_password("pw", &cheap_params());
    let stronger = Params::new(16, 2, 1, None).unwrap();
    let strong = hash_password("pw", &stronger);
    assert!(verify_password("pw", &weak));
    assert!(verify_password("pw", &strong));
}

#[test]
fn describe_hash_reads_every_field() {
    let info = describe_hash(PHC).unwrap();
    assert_eq!(info.algorithm, "argon2id");
    assert_eq!(info.version, 19);
    assert_eq!(
        (info.memory_kib, info.iterations, info.lanes),
        (19456, 2, 1)
    );
    assert_eq!(info.salt, "c29tZXNhbHQ");
    assert_eq!(info.hash_len, 24);
}

#[test]
fn describe_hash_of_a_fresh_hash_has_a_32_byte_output() {
    let info = describe_hash(&hash_password("pw", &cheap_params())).unwrap();
    assert_eq!((info.memory_kib, info.iterations, info.lanes), (8, 1, 1));
    assert_eq!(info.hash_len, 32);
}

#[test]
fn describe_hash_rejects_garbage_and_incomplete_strings() {
    assert_eq!(describe_hash("nope"), None);
    assert_eq!(describe_hash(""), None);
    assert_eq!(describe_hash("$argon2id$v=19$m=8,t=1,p=1"), None);
}

#[test]
fn needs_rehash_compares_each_setting() {
    let target = Params::new(16, 2, 1, None).unwrap();
    let weak = hash_password("pw", &cheap_params());
    assert!(needs_rehash(&weak, &target));
    assert!(!needs_rehash(&weak, &cheap_params()));
    let strong = hash_password("pw", &Params::new(32, 3, 2, None).unwrap());
    assert!(!needs_rehash(&strong, &target));
    let low_t = hash_password("pw", &Params::new(32, 1, 1, None).unwrap());
    assert!(needs_rehash(&low_t, &target));
}

#[test]
fn needs_rehash_is_true_for_unparsable_or_other_algorithms() {
    assert!(needs_rehash("garbage", &cheap_params()));
    let argon2i = "$argon2i$v=19$m=8,t=1,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG";
    assert!(needs_rehash(argon2i, &cheap_params()));
}

#[test]
fn constant_time_eq_matches_ordinary_equality() {
    assert!(constant_time_eq(b"", b""));
    assert!(constant_time_eq(b"secret", b"secret"));
    assert!(!constant_time_eq(b"secret", b"secreT"));
    assert!(!constant_time_eq(b"Secret", b"secret"));
    assert!(!constant_time_eq(b"secret", b"secre"));
    assert!(!constant_time_eq(b"", b"x"));
}
