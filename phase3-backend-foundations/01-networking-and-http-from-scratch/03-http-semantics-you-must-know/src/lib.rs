//! Exercises for 3.1.3 — HTTP semantics you must know.
//!
//! Everything here is plain HTTP-spec logic: no socket, no framework — see
//! the README for what each function needs to do and why.

/// The nine methods HTTP/1.1 actually defines (RFC 9110 §9). A curriculum
/// crate, not a wire-format parser — construct these directly instead of
/// parsing them from text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

/// Whether `method` is *safe*: the spec's promise that calling it is not
/// expected to change server state (RFC 9110 §9.2.1). Safety says nothing
/// about how many times you can call it — that's `is_idempotent`, a
/// separate property.
pub fn is_safe(method: &Method) -> bool {
    todo!(
        "true for exactly Method::Get, Method::Head, Method::Options, and Method::Trace; false \
         for every other method"
    )
}

/// Whether calling `method` any number of times leaves the server in the
/// same state as calling it once (RFC 9110 §9.2.2). `POST` and `PATCH` are
/// the two methods the spec does not make this promise for — even though a
/// real API's `PATCH` often happens to behave idempotently, the spec itself
/// does not require it.
pub fn is_idempotent(method: &Method) -> bool {
    todo!(
        "true for exactly Method::Get, Method::Head, Method::Put, Method::Delete, \
         Method::Options, and Method::Trace; false for Method::Post, Method::Patch, and \
         Method::Connect"
    )
}

/// The five status-code classes HTTP defines by a code's leading digit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
}

/// Classifies `status` by its leading digit. `None` for anything outside
/// `100..=599` — `u16` can hold those values, but the HTTP spec has nothing
/// to say about a "700", so there is no class to return.
pub fn status_class(status: u16) -> Option<StatusClass> {
    todo!(
        "100..=199 is Informational, 200..=299 is Success, 300..=399 is Redirection, 400..=499 \
         is ClientError, 500..=599 is ServerError; anything outside 100..=599 is not a valid \
         HTTP status code, so return None"
    )
}

/// Picks the best media type from `available` for a client's `Accept`
/// header.
///
/// `accept` is a comma-separated list of entries, each an HTTP media type
/// optionally followed by `;q=WEIGHT` (a number from `0` to `1`; a missing
/// `q` means `1.0`, and an unparseable `q` also defaults to `1.0` rather
/// than rejecting the whole entry). A `TYPE/SUBTYPE` in `available` matches
/// an entry when the entry is exactly that type, or is `TYPE/*`, or is
/// `*/*`; matching is case-insensitive. Each `available` type's weight is
/// its *most specific* matching entry's `q` — an exact match always wins
/// over `TYPE/*`, which always wins over `*/*`, regardless of which one's
/// `q` is numerically higher. A type with no matching entry, or whose most
/// specific match has `q` of exactly `0`, is not a candidate. Return the
/// surviving candidate with the highest weight, breaking a tie by
/// `available`'s own order (the server's own preference); return `None` if
/// no candidate survives, including when `available` is empty.
pub fn best_content_type(accept: &str, available: &[&str]) -> Option<String> {
    todo!(
        "return the type from available that the client's Accept header weights highest, per \
         the matching and tie-breaking rules in the doc comment above"
    )
}

/// What can go wrong decoding a chunked body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkedDecodeError {
    /// A chunk-size line was not a valid hexadecimal number.
    InvalidLength(String),
    /// The body ended before a chunk's declared length was satisfied, or
    /// before the terminating zero-length chunk.
    UnexpectedEnd,
}

/// Decodes an HTTP/1.1 chunked-transfer body (RFC 9112 §7.1) back into its
/// original bytes.
///
/// The format: a chunk-size line in hexadecimal, then `\r\n`, then exactly
/// that many bytes of chunk data, then `\r\n` — repeated — ending in a
/// chunk-size line of `0` followed by one final `\r\n` (no trailer headers;
/// out of scope here). Return `Err(InvalidLength(line))` — `line` being the
/// exact text of the offending chunk-size line — if a chunk-size line is
/// not valid hexadecimal; return `Err(UnexpectedEnd)` if the body runs out
/// of bytes before a chunk's declared length, or before the terminating
/// `0\r\n\r\n`.
pub fn decode_chunked(body: &[u8]) -> Result<Vec<u8>, ChunkedDecodeError> {
    todo!(
        "return the original body bytes with the chunk framing stripped away, or the \
         ChunkedDecodeError naming what was wrong with the framing"
    )
}

#[cfg(test)]
mod safe_and_idempotent_tests {
    use super::*;

    #[test]
    fn get_head_options_trace_are_safe() {
        assert!(is_safe(&Method::Get));
        assert!(is_safe(&Method::Head));
        assert!(is_safe(&Method::Options));
        assert!(is_safe(&Method::Trace));
    }

    #[test]
    fn put_delete_post_patch_connect_are_not_safe() {
        assert!(!is_safe(&Method::Put));
        assert!(!is_safe(&Method::Delete));
        assert!(!is_safe(&Method::Post));
        assert!(!is_safe(&Method::Patch));
        assert!(!is_safe(&Method::Connect));
    }

    #[test]
    fn get_head_put_delete_options_trace_are_idempotent() {
        assert!(is_idempotent(&Method::Get));
        assert!(is_idempotent(&Method::Head));
        assert!(is_idempotent(&Method::Put));
        assert!(is_idempotent(&Method::Delete));
        assert!(is_idempotent(&Method::Options));
        assert!(is_idempotent(&Method::Trace));
    }

    #[test]
    fn post_patch_connect_are_not_idempotent() {
        assert!(!is_idempotent(&Method::Post));
        assert!(!is_idempotent(&Method::Patch));
        assert!(!is_idempotent(&Method::Connect));
    }
}

#[cfg(test)]
mod status_class_tests {
    use super::*;

    #[test]
    fn classifies_one_code_from_each_class() {
        assert_eq!(status_class(101), Some(StatusClass::Informational));
        assert_eq!(status_class(201), Some(StatusClass::Success));
        assert_eq!(status_class(308), Some(StatusClass::Redirection));
        assert_eq!(status_class(422), Some(StatusClass::ClientError));
        assert_eq!(status_class(503), Some(StatusClass::ServerError));
    }

    #[test]
    fn class_boundaries_are_exact() {
        assert_eq!(status_class(299), Some(StatusClass::Success));
        assert_eq!(status_class(300), Some(StatusClass::Redirection));
        assert_eq!(status_class(399), Some(StatusClass::Redirection));
        assert_eq!(status_class(400), Some(StatusClass::ClientError));
    }

    #[test]
    fn out_of_range_codes_have_no_class() {
        assert_eq!(status_class(0), None);
        assert_eq!(status_class(99), None);
        assert_eq!(status_class(600), None);
        assert_eq!(status_class(65535), None);
    }
}

#[cfg(test)]
mod content_negotiation_tests {
    use super::*;

    #[test]
    fn exact_match_beats_the_wildcard_that_also_matches() {
        let accept = "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";
        let available = ["application/json", "text/html"];
        assert_eq!(
            best_content_type(accept, &available),
            Some("text/html".to_string())
        );
    }

    #[test]
    fn q_zero_on_the_specific_match_excludes_it_even_though_the_wildcard_would_accept_it() {
        let accept = "text/html;q=0,*/*";
        let available = ["text/html", "application/json"];
        assert_eq!(
            best_content_type(accept, &available),
            Some("application/json".to_string())
        );
    }

    #[test]
    fn no_matching_entry_at_all_returns_none() {
        let accept = "application/json";
        let available = ["text/html", "text/plain"];
        assert_eq!(best_content_type(accept, &available), None);
    }

    #[test]
    fn ties_break_by_availables_own_order() {
        let accept = "*/*";
        let available = ["text/plain", "application/json"];
        assert_eq!(
            best_content_type(accept, &available),
            Some("text/plain".to_string())
        );
    }

    #[test]
    fn subtype_wildcard_outranks_the_full_wildcard() {
        let accept = "image/*;q=0.7,*/*;q=0.1";
        let available = ["image/png", "text/html"];
        assert_eq!(
            best_content_type(accept, &available),
            Some("image/png".to_string())
        );
    }

    #[test]
    fn unparseable_q_defaults_to_one_instead_of_rejecting_the_entry() {
        let accept = "text/html;q=abc";
        let available = ["text/html"];
        assert_eq!(
            best_content_type(accept, &available),
            Some("text/html".to_string())
        );
    }

    #[test]
    fn empty_available_returns_none() {
        assert_eq!(best_content_type("*/*", &[]), None);
    }
}

#[cfg(test)]
mod chunked_decode_tests {
    use super::*;

    #[test]
    fn decodes_the_classic_three_chunk_body() {
        let body = b"4\r\nWiki\r\n5\r\npedia\r\nE\r\n in\r\n\r\nchunks.\r\n0\r\n\r\n";
        assert_eq!(
            decode_chunked(body),
            Ok(b"Wikipedia in\r\n\r\nchunks.".to_vec())
        );
    }

    #[test]
    fn a_body_with_only_the_terminating_chunk_decodes_to_empty() {
        assert_eq!(decode_chunked(b"0\r\n\r\n"), Ok(Vec::new()));
    }

    #[test]
    fn a_non_hex_length_line_is_reported_by_name() {
        assert_eq!(
            decode_chunked(b"ZZ\r\nsomething\r\n0\r\n\r\n"),
            Err(ChunkedDecodeError::InvalidLength("ZZ".to_string()))
        );
    }

    #[test]
    fn a_body_cut_off_mid_chunk_is_unexpected_end() {
        assert_eq!(
            decode_chunked(b"5\r\nabc"),
            Err(ChunkedDecodeError::UnexpectedEnd)
        );
    }

    #[test]
    fn a_body_missing_the_terminating_chunk_is_unexpected_end() {
        assert_eq!(
            decode_chunked(b"4\r\nWiki\r\n"),
            Err(ChunkedDecodeError::UnexpectedEnd)
        );
    }
}
