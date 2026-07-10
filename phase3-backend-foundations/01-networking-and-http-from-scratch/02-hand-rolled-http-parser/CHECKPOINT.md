# Checkpoint

1. `parse_request` calls `std::str::from_utf8(raw)` and returns
   `HttpParseError::InvalidUtf8` on failure before doing anything else with
   the bytes. What would go wrong later in the function if it skipped that
   check and just assumed the bytes were valid text?
2. `HttpParseError` has four distinct variants instead of one generic
   `ParseFailed` variant. Pick `rejects_malformed_request_line` and
   `rejects_malformed_header_line` in `tests/parser_test.rs` — what does a
   caller (or a test's `assert!(matches!(...))`) get from two separate
   variants that it wouldn't get from a single catch-all error?
3. `HttpRequest::header` does a case-insensitive comparison
   (`eq_ignore_ascii_case`) at lookup time, but `parse_request` stores
   header names exactly as the client sent them (`headers: Vec<(String,
   String)>`, no lowercasing). Why not just lowercase every header name
   once, up front, during parsing, and do a plain `==` at lookup time
   instead?
4. `HttpResponse::to_bytes` computes `Content-Length` from
   `self.body.len()` (byte length) rather than
   `self.body.chars().count()` (character count). Construct a body string
   where those two numbers would differ, and explain why a client reading
   exactly `Content-Length` bytes off the wire would break if this method
   used the character count instead.
5. Django's `request.GET` is already a parsed `QueryDict` by the time your
   view sees it. This lesson's `HttpRequest` only stores
   `query: Option<String>` — the raw, unparsed query string. What would you
   need to add to turn `Some("status=watching&page=2")` into something
   `QueryDict`-like (e.g. `HashMap<String, String>`)? You don't have to
   implement it — just describe the parsing steps.
