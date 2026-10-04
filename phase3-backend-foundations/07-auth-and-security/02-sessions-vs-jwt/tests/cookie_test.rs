//! The pure functions: `Set-Cookie` builder, `Cookie` header parser, and the
//! `Origin` check.

use p3_07_02_sessions_vs_jwt::{
    expire_cookie, origin_ok, session_id_from_cookie_header, set_cookie, CookieError, CookieOpts,
    SameSite,
};

fn opts(max_age: Option<u64>, secure: bool, same_site: SameSite) -> CookieOpts {
    CookieOpts {
        max_age,
        secure,
        same_site,
    }
}

#[test]
fn the_default_shape_is_path_httponly_samesite() {
    let c = set_cookie("sid", "abc", &opts(None, false, SameSite::Lax)).unwrap();
    assert_eq!(c, "sid=abc; Path=/; HttpOnly; SameSite=Lax");
}

#[test]
fn secure_comes_before_samesite_and_max_age_comes_last() {
    let c = set_cookie("sid", "abc", &opts(Some(3600), true, SameSite::Strict)).unwrap();
    assert_eq!(
        c,
        "sid=abc; Path=/; HttpOnly; Secure; SameSite=Strict; Max-Age=3600"
    );
}

#[test]
fn max_age_zero_is_written_out_not_dropped() {
    let c = set_cookie("sid", "abc", &opts(Some(0), false, SameSite::Lax)).unwrap();
    assert!(c.ends_with("; Max-Age=0"));
}

#[test]
fn samesite_none_with_secure_is_allowed() {
    let c = set_cookie("sid", "abc", &opts(None, true, SameSite::None)).unwrap();
    assert_eq!(c, "sid=abc; Path=/; HttpOnly; Secure; SameSite=None");
}

#[test]
fn samesite_none_without_secure_is_an_error() {
    let r = set_cookie("sid", "abc", &opts(None, false, SameSite::None));
    assert_eq!(r, Err(CookieError::SameSiteNoneNeedsSecure));
}

#[test]
fn an_empty_value_is_allowed() {
    let c = set_cookie("sid", "", &opts(None, false, SameSite::Lax)).unwrap();
    assert!(c.starts_with("sid=; Path=/"));
}

#[test]
fn an_attribute_smuggled_through_the_value_is_rejected() {
    let r = set_cookie(
        "sid",
        "a; Domain=evil.example",
        &opts(None, false, SameSite::Lax),
    );
    assert_eq!(r, Err(CookieError::Invalid));
}

#[test]
fn names_and_values_with_forbidden_characters_are_rejected() {
    let o = opts(None, false, SameSite::Lax);
    for bad_value in [
        "a,b",
        "a b",
        "a\"b",
        "a\\b",
        "a\r\nSet-Cookie: x=y",
        "caf\u{e9}",
    ] {
        assert_eq!(
            set_cookie("sid", bad_value, &o),
            Err(CookieError::Invalid),
            "{bad_value:?}"
        );
    }
    for bad_name in ["", "a=b", "a;b", "a b"] {
        assert_eq!(
            set_cookie(bad_name, "v", &o),
            Err(CookieError::Invalid),
            "{bad_name:?}"
        );
    }
}

#[test]
fn invalid_input_is_reported_before_the_samesite_rule() {
    let r = set_cookie("sid", "a;b", &opts(None, false, SameSite::None));
    assert_eq!(r, Err(CookieError::Invalid));
}

#[test]
fn expire_cookie_blanks_the_value_and_sets_max_age_zero() {
    let c = expire_cookie("sid", &opts(None, true, SameSite::Lax)).unwrap();
    assert_eq!(c, "sid=; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=0");
}

#[test]
fn expire_cookie_ignores_the_max_age_in_opts() {
    let c = expire_cookie("sid", &opts(Some(999), false, SameSite::Strict)).unwrap();
    assert_eq!(c, "sid=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0");
}

#[test]
fn expire_cookie_keeps_the_samesite_secure_rule() {
    let r = expire_cookie("sid", &opts(None, false, SameSite::None));
    assert_eq!(r, Err(CookieError::SameSiteNoneNeedsSecure));
}

#[test]
fn the_cookie_header_parser_finds_a_pair_among_others() {
    assert_eq!(
        session_id_from_cookie_header("a=1; sid=abc; b=2", "sid"),
        Some("abc")
    );
    assert_eq!(session_id_from_cookie_header("sid=abc", "sid"), Some("abc"));
}

#[test]
fn the_cookie_header_parser_trims_whitespace_around_pairs() {
    assert_eq!(
        session_id_from_cookie_header("a=1;   sid=abc  ;b=2", "sid"),
        Some("abc")
    );
}

#[test]
fn the_cookie_header_parser_matches_the_whole_name_case_sensitively() {
    assert_eq!(session_id_from_cookie_header("xsid=1; SID=2", "sid"), None);
    assert_eq!(session_id_from_cookie_header("", "sid"), None);
}

#[test]
fn the_cookie_header_parser_takes_the_first_match_and_keeps_equals_signs_in_the_value() {
    assert_eq!(
        session_id_from_cookie_header("sid=one; sid=two", "sid"),
        Some("one")
    );
    assert_eq!(session_id_from_cookie_header("sid=a=b", "sid"), Some("a=b"));
}

#[test]
fn the_cookie_header_parser_treats_an_empty_value_as_absent() {
    assert_eq!(session_id_from_cookie_header("sid=; a=1", "sid"), None);
}

#[test]
fn a_request_with_no_origin_passes_the_origin_check() {
    assert!(origin_ok(None, "http://localhost:3000"));
}

#[test]
fn only_the_exact_expected_origin_passes() {
    let me = "http://localhost:3000";
    assert!(origin_ok(Some(me), me));
    assert!(!origin_ok(Some("http://evil.example"), me));
    assert!(!origin_ok(Some("http://localhost:3000/"), me));
    assert!(!origin_ok(Some("null"), me));
}
