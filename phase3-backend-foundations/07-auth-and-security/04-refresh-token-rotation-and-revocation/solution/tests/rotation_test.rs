use p3_07_04_refresh_token_rotation_and_revocation_solution::{
    hash_token, ManualClock, RefreshError, RefreshService, ACCESS_TTL, REFRESH_TTL,
};

const T0: u64 = 1_700_000_000;

fn service() -> (RefreshService<ManualClock>, ManualClock) {
    let clock = ManualClock::new(T0);
    (RefreshService::new(clock.clone()), clock)
}

// --- hash_token -----------------------------------------------------------

#[test]
fn hash_matches_the_known_sha256_vector() {
    assert_eq!(
        hash_token("abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn hash_is_64_lowercase_hex_chars_and_deterministic() {
    let h = hash_token("some-refresh-token");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f')));
    assert_eq!(h, hash_token("some-refresh-token"));
    assert_ne!(h, hash_token("some-refresh-token "));
}

// --- storing only hashes --------------------------------------------------

#[test]
fn login_stores_the_hash_not_the_token() {
    let (mut svc, _) = service();
    let pair = svc.login("matin");
    assert_eq!(pair.refresh_token.len(), 64);
    assert_eq!(svc.stored_hashes(), vec![hash_token(&pair.refresh_token)]);
    assert!(!svc.stored_hashes().contains(&pair.refresh_token));
}

// --- rotate: the happy path ----------------------------------------------

#[test]
fn rotate_returns_a_new_pair_for_the_same_user() {
    let (mut svc, clock) = service();
    let first = svc.login("matin");
    clock.advance(60);
    let second = svc.rotate(&first.refresh_token).unwrap();
    assert_ne!(second.refresh_token, first.refresh_token);
    assert_eq!(
        second.access_token,
        format!("access.matin.{}", T0 + 60 + ACCESS_TTL)
    );
    assert_eq!(svc.record_count(), 2);
}

#[test]
fn rotation_can_be_chained() {
    let (mut svc, _) = service();
    let mut token = svc.login("matin").refresh_token;
    for _ in 0..5 {
        token = svc.rotate(&token).unwrap().refresh_token;
    }
    assert!(svc.rotate(&token).is_ok());
}

// --- rotate: the failure outcomes ----------------------------------------

#[test]
fn unknown_token_is_unknown_and_changes_nothing() {
    let (mut svc, _) = service();
    svc.login("matin");
    assert_eq!(svc.rotate("not-a-real-token"), Err(RefreshError::Unknown));
    assert_eq!(svc.record_count(), 1);
}

#[test]
fn reusing_a_rotated_token_revokes_the_whole_family() {
    let (mut svc, _) = service();
    let first = svc.login("matin");
    let second = svc.rotate(&first.refresh_token).unwrap();
    assert_eq!(
        svc.rotate(&first.refresh_token),
        Err(RefreshError::ReuseDetected)
    );
    assert_eq!(svc.record_count(), 0);
    // The legitimate holder of the newest token is cut off too.
    assert_eq!(
        svc.rotate(&second.refresh_token),
        Err(RefreshError::Unknown)
    );
}

#[test]
fn reuse_in_one_family_leaves_other_families_alone() {
    let (mut svc, _) = service();
    let phone = svc.login("matin");
    let laptop = svc.login("matin");
    let other = svc.login("sara");
    svc.rotate(&phone.refresh_token).unwrap();
    assert_eq!(
        svc.rotate(&phone.refresh_token),
        Err(RefreshError::ReuseDetected)
    );
    assert_eq!(svc.record_count(), 2);
    assert!(svc.rotate(&laptop.refresh_token).is_ok());
    assert!(svc.rotate(&other.refresh_token).is_ok());
}

#[test]
fn token_expires_exactly_at_expires_at() {
    let (mut svc, clock) = service();
    let pair = svc.login("matin");
    clock.advance(REFRESH_TTL - 1);
    let still_good = svc.rotate(&pair.refresh_token).unwrap();
    clock.advance(REFRESH_TTL);
    assert_eq!(
        svc.rotate(&still_good.refresh_token),
        Err(RefreshError::Expired)
    );
}

#[test]
fn expired_does_not_revoke_the_family() {
    let (mut svc, clock) = service();
    let pair = svc.login("matin");
    clock.advance(REFRESH_TTL);
    assert_eq!(svc.rotate(&pair.refresh_token), Err(RefreshError::Expired));
    assert_eq!(svc.record_count(), 1);
    // Asking again gives the same answer: Expired is not "used".
    assert_eq!(svc.rotate(&pair.refresh_token), Err(RefreshError::Expired));
}

#[test]
fn used_is_checked_before_expiry() {
    let (mut svc, clock) = service();
    let first = svc.login("matin");
    svc.rotate(&first.refresh_token).unwrap();
    clock.advance(REFRESH_TTL + 1);
    assert_eq!(
        svc.rotate(&first.refresh_token),
        Err(RefreshError::ReuseDetected)
    );
}

// --- logout ---------------------------------------------------------------

#[test]
fn logout_revokes_the_family_and_reports_whether_it_existed() {
    let (mut svc, _) = service();
    let first = svc.login("matin");
    let second = svc.rotate(&first.refresh_token).unwrap();
    assert!(svc.logout(&second.refresh_token));
    assert_eq!(svc.record_count(), 0);
    assert_eq!(
        svc.rotate(&second.refresh_token),
        Err(RefreshError::Unknown)
    );
    assert!(!svc.logout(&second.refresh_token));
    assert!(!svc.logout("never-issued"));
}

#[test]
fn logout_only_touches_its_own_family() {
    let (mut svc, _) = service();
    let phone = svc.login("matin");
    let laptop = svc.login("matin");
    assert!(svc.logout(&phone.refresh_token));
    assert_eq!(svc.record_count(), 1);
    assert!(svc.rotate(&laptop.refresh_token).is_ok());
}

#[test]
fn logout_works_on_an_already_used_token() {
    let (mut svc, _) = service();
    let first = svc.login("matin");
    svc.rotate(&first.refresh_token).unwrap();
    assert!(svc.logout(&first.refresh_token));
    assert_eq!(svc.record_count(), 0);
}

// --- logout_all (the Build rung) -----------------------------------------

#[test]
fn logout_all_removes_every_family_of_one_user() {
    let (mut svc, _) = service();
    let phone = svc.login("matin");
    let laptop = svc.login("matin");
    let sara = svc.login("sara");
    svc.rotate(&phone.refresh_token).unwrap();
    assert_eq!(svc.logout_all("matin"), 2);
    assert_eq!(svc.record_count(), 1);
    assert_eq!(
        svc.rotate(&laptop.refresh_token),
        Err(RefreshError::Unknown)
    );
    assert!(svc.rotate(&sara.refresh_token).is_ok());
    assert_eq!(svc.logout_all("matin"), 0);
}
