//! Solution for 3.1.3 — HTTP semantics you must know. See `../README.md` for
//! the walkthrough and `SOLUTION.md` for commentary on this exact code.

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

pub fn is_safe(method: &Method) -> bool {
    matches!(
        method,
        Method::Get | Method::Head | Method::Options | Method::Trace
    )
}

pub fn is_idempotent(method: &Method) -> bool {
    matches!(
        method,
        Method::Get | Method::Head | Method::Put | Method::Delete | Method::Options | Method::Trace
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Informational,
    Success,
    Redirection,
    ClientError,
    ServerError,
}

pub fn status_class(status: u16) -> Option<StatusClass> {
    match status {
        100..=199 => Some(StatusClass::Informational),
        200..=299 => Some(StatusClass::Success),
        300..=399 => Some(StatusClass::Redirection),
        400..=499 => Some(StatusClass::ClientError),
        500..=599 => Some(StatusClass::ServerError),
        _ => None,
    }
}

/// Splits `"type/subtype"` into its two halves. A malformed entry with no
/// `/` at all (never produced by a well-formed `Accept` header, but nothing
/// stops a client from sending one) becomes `(entry, "")`, which simply
/// matches nothing and is dropped.
fn media_parts(media: &str) -> (&str, &str) {
    media.split_once('/').unwrap_or((media, ""))
}

pub fn best_content_type(accept: &str, available: &[&str]) -> Option<String> {
    struct AcceptEntry<'a> {
        kind: &'a str,
        subkind: &'a str,
        q: f64,
    }

    let entries: Vec<AcceptEntry> = accept
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (media, q) = match entry.split_once(";q=") {
                Some((media, weight)) => (media.trim(), weight.trim().parse().unwrap_or(1.0)),
                None => (entry, 1.0),
            };
            let (kind, subkind) = media_parts(media);
            AcceptEntry { kind, subkind, q }
        })
        .collect();

    let mut best: Option<(usize, f64)> = None;

    for (index, candidate) in available.iter().enumerate() {
        let (want_kind, want_subkind) = media_parts(candidate);

        // Highest-specificity match wins: 2 = exact, 1 = `TYPE/*`, 0 = `*/*`.
        let mut chosen: Option<(u8, f64)> = None;
        for entry in &entries {
            let specificity = if entry.kind.eq_ignore_ascii_case("*") {
                0u8
            } else if !entry.kind.eq_ignore_ascii_case(want_kind) {
                continue;
            } else if entry.subkind.eq_ignore_ascii_case("*") {
                1
            } else if entry.subkind.eq_ignore_ascii_case(want_subkind) {
                2
            } else {
                continue;
            };

            let better = match chosen {
                None => true,
                Some((cur_specificity, cur_q)) => {
                    specificity > cur_specificity || (specificity == cur_specificity && entry.q > cur_q)
                }
            };
            if better {
                chosen = Some((specificity, entry.q));
            }
        }

        let Some((_, q)) = chosen else { continue };
        if q <= 0.0 {
            continue;
        }

        let better = match best {
            None => true,
            Some((_, best_q)) => q > best_q,
        };
        if better {
            best = Some((index, q));
        }
    }

    best.map(|(index, _)| available[index].to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkedDecodeError {
    InvalidLength(String),
    UnexpectedEnd,
}

fn find_crlf(data: &[u8]) -> Option<usize> {
    data.windows(2).position(|pair| pair == b"\r\n")
}

pub fn decode_chunked(body: &[u8]) -> Result<Vec<u8>, ChunkedDecodeError> {
    let mut out = Vec::new();
    let mut rest = body;

    loop {
        let line_end = find_crlf(rest).ok_or(ChunkedDecodeError::UnexpectedEnd)?;
        let size_line = &rest[..line_end];
        rest = &rest[line_end + 2..];

        let size_text = std::str::from_utf8(size_line)
            .map_err(|_| ChunkedDecodeError::InvalidLength(String::from_utf8_lossy(size_line).into_owned()))?;
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| ChunkedDecodeError::InvalidLength(size_text.to_string()))?;

        if size == 0 {
            return if rest.len() >= 2 && &rest[..2] == b"\r\n" {
                Ok(out)
            } else {
                Err(ChunkedDecodeError::UnexpectedEnd)
            };
        }

        if rest.len() < size + 2 || &rest[size..size + 2] != b"\r\n" {
            return Err(ChunkedDecodeError::UnexpectedEnd);
        }
        out.extend_from_slice(&rest[..size]);
        rest = &rest[size + 2..];
    }
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
